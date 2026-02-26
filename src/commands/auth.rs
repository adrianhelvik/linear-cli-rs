use anyhow::{Context, Result};
use std::io::IsTerminal;

use crate::api::client::LinearClient;
use crate::api::queries;
use crate::api::types::ViewerResponse;
use crate::cli::AuthArgs;
use crate::config;

pub async fn run(args: AuthArgs) -> Result<()> {
    let key = if let Some(path) = args.key_file {
        std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read API key file {}", path.display()))?
            .trim()
            .to_string()
    } else if std::io::stdin().is_terminal() {
        rpassword::prompt_password("Enter your Linear API key: ")?
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
