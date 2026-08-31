use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries;
use crate::api::types::{Team, TeamsResponse};
use crate::config;
use crate::output;

pub async fn run() -> Result<()> {
    let client = LinearClient::new(config::api_key()?);
    let teams: Vec<Team> = client
        .query_all::<TeamsResponse, Team>(queries::TEAMS, json!({ "first": 250 }), 10_000)
        .await?;
    output::team_table(&teams);
    Ok(())
}
