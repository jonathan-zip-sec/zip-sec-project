use super::{
    client::{JamfClient, JamfClientError},
    models::JamfComputer,
};
use crate::jampf::client::ComputerInventorySection;
use crate::jampf::client::JamfClientTrait;
use serde::{Deserialize, Serialize};
use tracing::error;
use version_compare::{compare, Cmp};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct DevicesOutput {
    computers: Vec<Computer>,
}

fn is_os_updated(os_version: String, available_updates: Vec<String>) -> bool {
    // NOTE: There are probably different ways to interpret an OS being out of date
    //  here we just assume anything lower than the highest version in the list of available updates
    //  is out of date
    !available_updates
        .iter()
        .any(|update| compare(os_version.clone(), update).unwrap_or(Cmp::Lt) == Cmp::Lt)
}

fn convert_jamf_computer_details(
    jamf_computer_details: JamfComputer,
    mac_os_versions: Vec<String>,
) -> Computer {
    let name = jamf_computer_details.general.and_then(|g| Some(g.name));
    let model = jamf_computer_details.hardware.and_then(|h| Some(h.model));
    let os = jamf_computer_details
        .operating_system
        .as_ref()
        .and_then(|o| Some(o.name.clone()));
    let os_version = jamf_computer_details
        .operating_system
        .as_ref()
        .and_then(|o| Some(o.version.clone()));
    let os_is_updated = os_version.and_then(|v| Some(is_os_updated(v, mac_os_versions)));

    Computer {
        name,
        model,
        os,
        os_is_latest: os_is_updated,
        device_id: jamf_computer_details.id.clone(),
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Computer {
    device_id: Option<String>,
    name: Option<String>,
    model: Option<String>,
    os: Option<String>,
    os_is_latest: Option<bool>,
}

pub struct ComputerProvider {
    pub(crate) jamf_client: JamfClient,
}

impl ComputerProvider {
    pub async fn fetch_computers(&self) -> Result<DevicesOutput, JamfClientError> {
        let inventory = self
            .jamf_client
            .get_computer_inventory(vec![
                ComputerInventorySection::OperatingSystem,
                ComputerInventorySection::General,
                ComputerInventorySection::Hardware,
            ])
            .await
            .inspect_err(|e| error!("Failed to fetch computers with error: {}", e))?;
        let os_versions = self
            .jamf_client
            .get_os_managed_updates()
            .await
            .inspect_err(|e| error!("Failed to get OS versions with error {}", e))?;
        let computers_output = DevicesOutput {
            computers: inventory
                .results
                .into_iter()
                .map(|i| {
                    convert_jamf_computer_details(i, os_versions.available_updates.mac_os.clone())
                })
                .collect(),
        };
        Ok(computers_output)
    }
}

#[cfg(test)]
mod test {
    use crate::jampf::{
        client::{ComputerInventorySection, JamfClient, MockJamfClientTrait},
        models::{
            AvailableUpdates, JamfAvailableUpdates, JamfComputer, JamfComputerGeneral,
            JamfComputerInventoryResponse, JamfComputerOperatingSystem,
        },
        provider::{ComputerProvider, DevicesOutput},
    };

    use super::Computer;

    #[tokio::test]
    async fn fetch_computers_empty() {
        let mut client_mock = MockJamfClientTrait::new();
        client_mock
            .expect_get_computer_inventory()
            .return_once(|inventory_section| {
                assert_eq!(
                    inventory_section,
                    vec![
                        ComputerInventorySection::OperatingSystem,
                        ComputerInventorySection::General,
                        ComputerInventorySection::Hardware,
                    ]
                );
                Ok(JamfComputerInventoryResponse {
                    total_count: 0,
                    results: vec![],
                })
            });
        client_mock
            .expect_get_os_managed_updates()
            .return_once(|| Ok(test_available_updates()));
        let jamf_client = JamfClient::Mock(client_mock);
        let computer_provider = ComputerProvider { jamf_client };
        let computers = computer_provider
            .fetch_computers()
            .await
            .expect("Should succeed");
        assert_eq!(computers, DevicesOutput { computers: vec![] });
    }

    #[tokio::test]
    async fn fetch_computers_one_computer() {
        let mut client_mock = MockJamfClientTrait::new();
        client_mock
            .expect_get_computer_inventory()
            .return_once(|inventory_section| {
                assert_eq!(
                    inventory_section,
                    vec![
                        ComputerInventorySection::OperatingSystem,
                        ComputerInventorySection::General,
                        ComputerInventorySection::Hardware,
                    ]
                );
                Ok(test_inventory_response())
            });
        client_mock
            .expect_get_os_managed_updates()
            .return_once(|| Ok(test_available_updates()));
        let jamf_client = JamfClient::Mock(client_mock);
        let computer_provider = ComputerProvider { jamf_client };
        let computers = computer_provider
            .fetch_computers()
            .await
            .expect("Should succeed");
        assert_eq!(
            computers,
            DevicesOutput {
                computers: vec![test_computer_output()]
            }
        );
    }

    // TODO: If I had more time, I'd write tests for more cases, mock errors etc... :)

    fn test_computer_output() -> Computer {
        Computer {
            device_id: Some("test_id".to_string()),
            name: Some("test_name".to_string()),
            model: None,
            os: Some("MacOS".to_string()),
            os_is_latest: Some(true),
        }
    }

    fn test_inventory_response() -> JamfComputerInventoryResponse {
        JamfComputerInventoryResponse {
            total_count: 1,
            results: vec![JamfComputer {
                hardware: None,
                software: None,
                operating_system: Some(JamfComputerOperatingSystem {
                    name: "MacOS".to_string(),
                    version: "14.0.0".to_string(),
                    build: "whatever".to_string(),
                    software_updates: None,
                }),
                general: Some(JamfComputerGeneral {
                    name: "test_name".to_string(),
                }),
                id: Some("test_id".to_string()),
                udid: Some("udid_test".to_string()),
            }],
        }
    }

    fn test_available_updates() -> JamfAvailableUpdates {
        JamfAvailableUpdates {
            available_updates: AvailableUpdates {
                mac_os: vec!["14.0.0".to_string()],
                ios: vec![],
            },
        }
    }
}
