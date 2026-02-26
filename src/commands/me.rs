use anyhow::Result;

use crate::api::client::LinearClient;
use crate::api::queries;
use crate::api::types::ViewerResponse;
use crate::config;

pub async fn run() -> Result<()> {
    let client = LinearClient::new(config::api_key()?);
    let resp: ViewerResponse = client
        .query(queries::VIEWER, serde_json::json!({}))
        .await?;

    let user = resp.viewer;
    let name = user
        .display_name
        .as_deref()
        .or(user.name.as_deref())
        .unwrap_or("—");
    let email = user.email.as_deref().unwrap_or("—");

    println!("Name:  {name}");
    println!("Email: {email}");
    println!("ID:    {}", user.id);
    Ok(())
}
