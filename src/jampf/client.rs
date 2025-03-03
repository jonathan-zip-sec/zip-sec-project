use enum_dispatch::enum_dispatch;
use reqwest::Client;
use thiserror::Error;
use tokio::sync::OnceCell;
use tracing::error;

use crate::jampf::models::JamfComputerInventoryResponse;

use super::models::{JamfAuthReponse, JamfAvailableUpdates};

// We should only create a single reqwest::Client and keep it as a singleton to avoid having many open connections
static JAMF_CLIENT: OnceCell<Client> = OnceCell::const_new();

static PAGE_SIZE: usize = 100;

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum ComputerInventorySection {
    General,
    Hardware,
    OperatingSystem,
}

impl ToString for ComputerInventorySection {
    fn to_string(&self) -> String {
        match self {
            ComputerInventorySection::General => "GENERAL".to_string(),
            ComputerInventorySection::Hardware => "HARDWARE".to_string(),
            ComputerInventorySection::OperatingSystem => "OPERATING_SYSTEM".to_string(),
        }
    }
}

#[enum_dispatch]
#[mockall::automock]
pub(crate) trait JamfClientTrait {
    async fn get_computer_inventory(
        &self,
        section: Vec<ComputerInventorySection>,
    ) -> Result<JamfComputerInventoryResponse, JamfClientError>;

    async fn get_os_managed_updates(&self) -> Result<JamfAvailableUpdates, JamfClientError>;

    // TODO: There aren't any mobile devices on the test jamf instance, but if there were I would add a method for fetching them
}

#[enum_dispatch(JamfClientTrait)]
pub(crate) enum JamfClient {
    Impl(JamfClientImpl),
    #[cfg(test)]
    Mock(MockJamfClientTrait),
}

pub(crate) struct JamfClientImpl {
    jamf_url: String,
    bearer_token: String,
}

#[derive(Error, Debug)]
pub enum JamfClientError {
    #[error("Failed to reach Jampf with error {0}")]
    ReqwestError(#[from] reqwest::Error),
}

// We should only create a single reqwest::Client and keep it as a singleton to avoid having many open connections
async fn get_client() -> Client {
    JAMF_CLIENT
        .get_or_init(|| async { Client::new() })
        .await
        .clone()
}

impl JamfClientImpl {
    pub async fn new(
        username: String,
        password: String,
        jamf_url: String,
    ) -> Result<Self, JamfClientError> {
        // NOTE: In a real app the request would probably come with a bearer token
        // just for the sake of this example, we authenticate when creating the client
        // we never have to refresh the token because the JamfClientImpl only lives
        // for a single request
        let response = get_client()
            .await
            .post(format!("{}/api/v1/auth/token", jamf_url))
            .basic_auth(username, Some(password))
            .send()
            .await?;
        let jamf_response = response.json::<JamfAuthReponse>().await?;
        Ok(Self {
            jamf_url,
            bearer_token: jamf_response.token,
        })
    }

    // Helper function to handle pagination
    // Gets a single page of the the computer inventory
    async fn get_computer_inventory_page(
        &self,
        section: &Vec<ComputerInventorySection>,
        page: usize,
    ) -> Result<JamfComputerInventoryResponse, JamfClientError> {
        // Select for the sections being requested
        let mut params = section
            .iter()
            .map(|s| ("section".to_string(), s.to_string()))
            .collect::<Vec<(String, String)>>();
        params.push(("page".to_string(), page.to_string()));
        params.push(("page-size".to_string(), PAGE_SIZE.to_string()));

        let response = get_client()
            .await
            .get(format!("{}/api/v1/computers-inventory", self.jamf_url))
            .header("accept", "application/json")
            .bearer_auth(self.bearer_token.clone())
            .query(&params)
            .send()
            .await
            .inspect_err(|e| error!("Failed to get computers inventory: {}", e))?;
        Ok(response
            .json::<JamfComputerInventoryResponse>()
            .await
            .inspect_err(|e| error!("Failed to create ComputerInventoryResponse: {}", e))?)
    }
}

impl JamfClientTrait for JamfClientImpl {
    /// Fetch all computers from computer inventory - include data from sections provided
    async fn get_computer_inventory(
        &self,
        section: Vec<ComputerInventorySection>,
    ) -> Result<JamfComputerInventoryResponse, JamfClientError> {
        let mut inventory_response = self.get_computer_inventory_page(&section, 0).await?;
        if inventory_response.total_count <= PAGE_SIZE {
            return Ok(inventory_response);
        }

        let num_pages =
            (inventory_response.total_count + PAGE_SIZE as usize - 1) / PAGE_SIZE as usize;
        for page in 1..num_pages {
            let pagination_response = self.get_computer_inventory_page(&section, page).await?;
            inventory_response
                .results
                .extend(pagination_response.results);
        }
        Ok(inventory_response)
    }

    /// Get all OS managed updates from Jamf server to determine if devices are up to date
    async fn get_os_managed_updates(&self) -> Result<JamfAvailableUpdates, JamfClientError> {
        let response = get_client()
            .await
            .get(format!(
                "{}/api/v1/managed-software-updates/available-updates",
                self.jamf_url
            ))
            .header("accept", "application/json")
            .bearer_auth(self.bearer_token.clone())
            .send()
            .await
            .inspect_err(|e| error!("Failed to current Mac OS versions: {}", e))?;

        Ok(response
            .json::<JamfAvailableUpdates>()
            .await
            .inspect_err(|e| error!("Failed to get available updates: {}", e))?)
    }
}

#[cfg(test)]
mod tests {
    use crate::jampf::client::{ComputerInventorySection, JamfClient, JamfClientTrait};
    use dotenv::dotenv;
    use std::env;

    use super::JamfClientImpl;

    async fn init() -> JamfClient {
        dotenv().ok();
        let username = env::var("USERNAME").expect("Please set username env var");
        let password = env::var("PASSWORD").expect("Please set password env var");
        let jamf_url = env::var("JAMF_URL").expect("Please set jamf_url env var");
        JamfClient::Impl(
            JamfClientImpl::new(username, password, jamf_url)
                .await
                .unwrap(),
        )
    }

    #[tokio::test]
    async fn get_computers_success() {
        let provider = init().await;

        let inventory = provider
            .get_computer_inventory(vec![
                ComputerInventorySection::OperatingSystem,
                ComputerInventorySection::General,
            ])
            .await
            .unwrap();
        assert_eq!(inventory.total_count, 4);
        assert_eq!(inventory.results.len(), 4);
    }

    #[tokio::test]
    async fn get_mac_os_updates() {
        let provider = init().await;
        let updates = provider.get_os_managed_updates().await.unwrap();
        // This can obviously change, just a placeholder so I have some kind of test
        assert_eq!(updates.available_updates.mac_os.len(), 14);
    }

    // TODO: If I had more time I would use http-relay to mock jamf server responses: https://crates.io/crates/http-relay
}
