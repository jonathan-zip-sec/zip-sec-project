use std::env;

use axum::response::Json;
use http::StatusCode;

use tracing::{info, instrument};

use crate::jampf::{
    client::{JamfClient, JamfClientImpl},
    provider::{ComputerProvider, ComputersOutput},
};
use dotenv::dotenv;

#[instrument]
pub async fn computers() -> Result<Json<ComputersOutput>, StatusCode> {
    dotenv().ok();
    // For now, just get everything from env variables - in the future can use postgres
    let username = env::var("USERNAME").expect("Please set username env var");
    let password = env::var("PASSWORD").expect("Please set password env var");
    let jamf_url = env::var("JAMF_URL").expect("Please set jamf_url env var");
    info!("About to create client!");

    let jamf_client = JamfClient::Impl(
        JamfClientImpl::new(username, password, jamf_url)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    let computer_provider = ComputerProvider { jamf_client };

    let inventory = computer_provider
        .fetch_computers()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let computers_output = ComputersOutput::from(inventory);
    Ok(Json(computers_output))
}
