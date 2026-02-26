use anyhow::Result;
use serde_json::json;

use super::client::LinearClient;
use super::queries;
use super::types::*;

pub async fn team_id(client: &LinearClient, key: &str) -> Result<String> {
    let resp: TeamsResponse = client.query(queries::TEAMS, json!({})).await?;
    resp.teams
        .nodes
        .iter()
        .find(|t| {
            t.key
                .as_deref()
                .is_some_and(|k| k.eq_ignore_ascii_case(key))
        })
        .map(|t| t.id.clone())
        .ok_or_else(|| anyhow::anyhow!("Team '{key}' not found"))
}

pub async fn user_id(client: &LinearClient, name: &str) -> Result<String> {
    if name == "me" {
        let resp: ViewerResponse = client.query(queries::VIEWER, json!({})).await?;
        return Ok(resp.viewer.id);
    }
    let resp: UsersResponse = client.query(queries::USERS, json!({})).await?;
    let lower = name.to_lowercase();
    resp.users
        .nodes
        .iter()
        .find(|u| {
            u.display_name
                .as_deref()
                .is_some_and(|n| n.to_lowercase().contains(&lower))
                || u.name
                    .as_deref()
                    .is_some_and(|n| n.to_lowercase().contains(&lower))
                || u.email
                    .as_deref()
                    .is_some_and(|e| e.to_lowercase().contains(&lower))
        })
        .map(|u| u.id.clone())
        .ok_or_else(|| anyhow::anyhow!("User '{name}' not found"))
}

pub async fn state_id(client: &LinearClient, team_id: &str, name: &str) -> Result<String> {
    let resp: WorkflowStatesResponse = client
        .query(
            queries::WORKFLOW_STATES,
            json!({ "filter": { "team": { "id": { "eq": team_id } } } }),
        )
        .await?;
    let lower = name.to_lowercase();
    resp.workflow_states
        .nodes
        .iter()
        .find(|s| s.name.as_deref().is_some_and(|n| n.to_lowercase() == lower))
        .map(|s| s.id.clone())
        .ok_or_else(|| anyhow::anyhow!("State '{name}' not found for this team"))
}

pub async fn label_id(client: &LinearClient, name: &str) -> Result<String> {
    let resp: LabelsResponse = client.query(queries::LABELS, json!({})).await?;
    let lower = name.to_lowercase();
    resp.issue_labels
        .nodes
        .iter()
        .find(|l| l.name.as_deref().is_some_and(|n| n.to_lowercase() == lower))
        .map(|l| l.id.clone())
        .ok_or_else(|| anyhow::anyhow!("Label '{name}' not found"))
}
