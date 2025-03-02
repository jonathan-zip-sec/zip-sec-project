use axum::response::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CredentialsInput {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct CredentialsOutput {
    pub username: String,
    pub password: String,
}

pub async fn credentials(Json(credentials): Json<CredentialsInput>) -> Json<CredentialsOutput> {
    Json(CredentialsOutput {
        username: credentials.username,
        password: credentials.password,
    })
}
