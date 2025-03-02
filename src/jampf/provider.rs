use super::{
    client::{JamfClient, JamfClientError},
    models::ComputerInventoryResponse,
};
use crate::jampf::client::ComputerInventorySection;
use crate::jampf::client::JamfClientTrait;
use serde::{Deserialize, Serialize};
use tracing::{error};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct ComputersOutput {
    computers: Vec<Computer>,
}

impl From<ComputerInventoryResponse> for ComputersOutput {
    fn from(value: ComputerInventoryResponse) -> Self {
        let computers = value
            .results
            .into_iter()
            .map(|c| {
                let name = c.general.and_then(|g| Some(g.name));
                let model = c.hardware.and_then(|h| Some(h.model));
                let os = c
                    .operating_system
                    .as_ref()
                    .and_then(|o| Some(o.name.clone()));
                let os_version = c
                    .operating_system
                    .as_ref()
                    .and_then(|o| Some(o.version.clone()));
                Computer {
                    name,
                    model,
                    os,
                    os_version,
                    device_id: c.id.clone(),
                }
            })
            .collect();
        Self { computers }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Computer {
    device_id: Option<String>,
    name: Option<String>,
    model: Option<String>,
    os: Option<String>,
    os_version: Option<String>,
}

pub struct ComputerProvider {
    jamf_client: JamfClient,
}

impl ComputerProvider {
    async fn fetch_computers(&self) -> Result<ComputersOutput, JamfClientError> {
        let inventory = self
            .jamf_client
            .get_computer_inventory(vec![
                ComputerInventorySection::OperatingSystem,
                ComputerInventorySection::General,
                ComputerInventorySection::Hardware,
            ])
            .await
            .inspect_err(|e| error!("Failed to fetch computers with error: {}", e))?;
        let computers_output = ComputersOutput::from(inventory);
        Ok(computers_output)
    }
}

#[cfg(test)]
mod test {
    use crate::jampf::{
        client::{ComputerInventorySection, JamfClient, MockJamfClientTrait},
        models::{ComputerInventoryResponse, GeneralMetadata, JamfComputerDetailedMetadata},
        provider::{ComputerProvider, ComputersOutput},
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
                Ok(ComputerInventoryResponse {
                    totalCount: 0,
                    results: vec![],
                })
            });
        let jamf_client = JamfClient::Mock(client_mock);
        let computer_provider = ComputerProvider { jamf_client };
        let computers = computer_provider
            .fetch_computers()
            .await
            .expect("Should succeed");
        assert_eq!(computers, ComputersOutput { computers: vec![] });
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
        let jamf_client = JamfClient::Mock(client_mock);
        let computer_provider = ComputerProvider { jamf_client };
        let computers = computer_provider
            .fetch_computers()
            .await
            .expect("Should succeed");
        assert_eq!(computers, ComputersOutput { computers: vec![test_computer_output()] });
    }

    // TODO: If I had more time, I'd write tests for more cases :) 

    fn test_computer_output() -> Computer {
        Computer {
            device_id: Some("test_id".to_string()),
            name: Some("test_name".to_string()),
            model: None,
            os: None,
            os_version: None,
        }
    }

    fn test_inventory_response() -> ComputerInventoryResponse {
        ComputerInventoryResponse {
            totalCount: 1,
            results: vec![JamfComputerDetailedMetadata {
                hardware: None,
                security: None,
                software: None,
                configuration_profiles: None,
                operating_system: None,
                general: Some(GeneralMetadata { name: "test_name".to_string() }),
                id: Some("test_id".to_string()),
                udid: Some("udid_test".to_string()),
            }],
        }
    }
}
