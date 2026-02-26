use anyhow::Result;
use std::io::IsTerminal;

use crate::api::client::LinearClient;
use crate::api::queries;
use crate::api::types::ViewerResponse;
use crate::config;

pub async fn run() -> Result<()> {
    let key = if std::io::stdin().is_terminal() {
        inquire::Password::new("Enter your Linear API key:")
            .without_confirmation()
            .prompt()?
    } else {
        let mut key = String::new();
        std::io::stdin().read_line(&mut key)?;
        key.trim().to_string()
    };

    if key.is_empty() {
        anyhow::bail!("API key cannot be empty");
    }

    // Validate by fetching viewer
    let client = LinearClient::new(key.clone());
    let resp: ViewerResponse = client.query(queries::VIEWER, serde_json::json!({})).await?;

    let name = resp
        .viewer
        .display_name
        .as_deref()
        .or(resp.viewer.name.as_deref())
        .unwrap_or("Unknown");

    let mut cfg = config::load()?;
    cfg.api_key = Some(key);
    config::save(&cfg)?;

    println!("Authenticated as {name}");
    Ok(())
}
