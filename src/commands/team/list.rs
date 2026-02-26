use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries;
use crate::api::types::TeamsResponse;
use crate::config;
use crate::output;

pub async fn run() -> Result<()> {
    let client = LinearClient::new(config::api_key()?);
    let resp: TeamsResponse = client.query(queries::TEAMS, json!({})).await?;
    output::team_table(&resp.teams.nodes);
    Ok(())
}
