use std::env;

use axum::response::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use tracing::{info, instrument, span, Level};

use crate::jampf::{
    client::{ComputerInventorySection, JamfClient, JamfClientImpl, JamfClientTrait},
    models::ComputerInventoryResponse,
    provider::ComputersOutput,
};
use dotenv::dotenv;

#[instrument]
pub async fn computers() -> Result<Json<ComputersOutput>, StatusCode> {
    dotenv().ok();
    let username = env::var("USERNAME").expect("Please set username env var");
    let password = env::var("PASSWORD").expect("Please set password env var");
    let jamf_url = env::var("JAMF_URL").expect("Please set jamf_url env var");
    info!("About to create client!");

    let jamf_client = JamfClient::Impl(
        JamfClientImpl::new(username, password, jamf_url)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    info!("About to call inventory!");
    let inventory = jamf_client
        .get_computer_inventory(vec![
            ComputerInventorySection::OperatingSystem,
            ComputerInventorySection::General,
            ComputerInventorySection::Hardware,
        ])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    info!("Called inventory");
    let computers_output = ComputersOutput::from(inventory);
    Ok(Json(computers_output))
}
