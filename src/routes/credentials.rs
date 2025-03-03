use axum::response::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CredentialsInput {
    pub username: String,
    pub password: String,
    pub url: String
}

#[derive(Serialize, Deserialize)]
pub struct CredentialsOutput {
    pub username: String,
    pub password: String,
    pub url: String
}

pub async fn credentials(Json(credentials): Json<CredentialsInput>) -> Json<CredentialsOutput> {
    // NOTE: For the real app, this would probably authenticate with Jamf and return a bearer token that could be included in subsequent requests
    Json(CredentialsOutput {
        username: credentials.username,
        password: credentials.password,
        url: credentials.url
    })
}
