use enum_dispatch::enum_dispatch;
use reqwest::Client;
use thiserror::Error;
use tokio::sync::OnceCell;
use tracing::{error, info};

use crate::jampf::models::ComputerInventoryResponse;

use super::models::{JamfAuthReponse, JamfComputerDetailedResponse, JamfGetComputersResponse};
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
    async fn get_computers(&self) -> Result<JamfGetComputersResponse, JamfClientError>;
    async fn get_computer_details(
        &self,
        id: usize,
    ) -> Result<JamfComputerDetailedResponse, JamfClientError>;
    async fn get_computer_inventory(
        &self,
        section: Vec<ComputerInventorySection>,
    ) -> Result<ComputerInventoryResponse, JamfClientError>;
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
    #[error("Failed to reach Jampf")]
    ConnectionError(#[from] reqwest::Error),
}

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
        let client = get_client().await;
        let response = client
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

    async fn get_computer_inventory_page(
        &self,
        section: &Vec<ComputerInventorySection>,
        page: usize,
    ) -> Result<ComputerInventoryResponse, JamfClientError> {
        info!("Getting inventory page");
        let mut params = section
            .iter()
            .map(|s| ("section".to_string(), s.to_string()))
            .collect::<Vec<(String, String)>>();
        params.push(("page".to_string(), page.to_string()));
        params.push(("page-size".to_string(), PAGE_SIZE.to_string()));
        let client = get_client().await;

        let response = client
            .get(format!("{}/api/v1/computers-inventory", self.jamf_url))
            .header("accept", "application/json")
            .bearer_auth(self.bearer_token.clone())
            .query(&params)
            .send()
            .await
            .inspect_err(|e| error!("Failed to get computers inventory: {}", e))?;
        Ok(response
            .json::<ComputerInventoryResponse>()
            .await
            .inspect_err(|e| error!("Failed to create ComputerInventoryResponse: {}", e))?)
    }
}

impl JamfClientTrait for JamfClientImpl {
    async fn get_computers(&self) -> Result<JamfGetComputersResponse, JamfClientError> {
        let client = get_client().await;

        let response = client
            .get(format!(
                "{}/JSSResource/computers/subset/basic",
                self.jamf_url
            ))
            .header("accept", "application/json")
            .bearer_auth(self.bearer_token.clone())
            .send()
            .await?;
        Ok(response.json::<JamfGetComputersResponse>().await?)
    }

    async fn get_computer_details(
        &self,
        id: usize,
    ) -> Result<JamfComputerDetailedResponse, JamfClientError> {
        let client = get_client().await;

        let response = client
            .get(format!(
                "{}/JSSResource/computers/id/{}",
                self.jamf_url,
                id.to_string()
            ))
            .header("accept", "application/json")
            .bearer_auth(self.bearer_token.clone())
            .send()
            .await?;
        Ok(response.json::<JamfComputerDetailedResponse>().await?)
    }

    async fn get_computer_inventory(
        &self,
        section: Vec<ComputerInventorySection>,
    ) -> Result<ComputerInventoryResponse, JamfClientError> {
        info!("Getting computer inventory");
        let mut inventory_response = self.get_computer_inventory_page(&section, 0).await?;
        if inventory_response.totalCount <= PAGE_SIZE {
            return Ok(inventory_response);
        }

        let num_pages =
            (inventory_response.totalCount + PAGE_SIZE as usize - 1) / PAGE_SIZE as usize;
        for page in 1..num_pages {
            let pagination_response = self.get_computer_inventory_page(&section, page).await?;
            inventory_response
                .results
                .extend(pagination_response.results);
        }
        Ok(inventory_response)
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
    async fn get_get_computers_success() {
        let provider = init().await;
        let computers = provider.get_computers().await.unwrap();

        assert_eq!(computers.computers.len(), 4);
        println!("{:?}", computers.computers[0]);

        let detailed_metadata = provider.get_computer_details(12).await.unwrap();
        println!("{:?}", detailed_metadata);

        let inventory = provider
            .get_computer_inventory(vec![
                ComputerInventorySection::OperatingSystem,
                ComputerInventorySection::General,
            ])
            .await
            .unwrap();
        println!("{:?}", inventory);
    }
}
