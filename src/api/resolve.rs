use anyhow::{Result, bail};
use serde_json::json;

use super::client::LinearClient;
use super::queries;
use super::types::*;

const LOOKUP_PAGE_SIZE: i32 = 250;
const LOOKUP_MAX: usize = 10_000;
const CANDIDATE_PREVIEW_LIMIT: usize = 5;

pub async fn team_id(client: &LinearClient, key: &str) -> Result<String> {
    let teams: Vec<Team> = client
        .query_all::<TeamsResponse, Team>(
            queries::TEAMS,
            json!({ "first": LOOKUP_PAGE_SIZE }),
            LOOKUP_MAX,
        )
        .await?;
    teams
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
    if name.eq_ignore_ascii_case("me") {
        let resp: ViewerResponse = client.query(queries::VIEWER, json!({})).await?;
        return Ok(resp.viewer.id);
    }

    let name = name.trim();
    if name.is_empty() {
        bail!("User cannot be empty");
    }

    let users: Vec<User> = client
        .query_all::<UsersResponse, User>(
            queries::USERS,
            json!({ "first": LOOKUP_PAGE_SIZE }),
            LOOKUP_MAX,
        )
        .await?;

    let exact: Vec<&User> = users
        .iter()
        .filter(|u| {
            equals_ignore_case(u.display_name.as_deref(), name)
                || equals_ignore_case(u.name.as_deref(), name)
                || equals_ignore_case(u.email.as_deref(), name)
        })
        .collect();
    match exact.len() {
        1 => return Ok(exact[0].id.clone()),
        2.. => {
            bail!(
                "Ambiguous user '{name}'. Matches: {}. Use a full email for an exact target.",
                render_user_candidates(&exact)
            )
        }
        _ => {}
    }

    let lower = name.to_lowercase();
    let fuzzy: Vec<&User> = users
        .iter()
        .filter(|u| {
            contains_ignore_case(u.display_name.as_deref(), &lower)
                || contains_ignore_case(u.name.as_deref(), &lower)
                || contains_ignore_case(u.email.as_deref(), &lower)
        })
        .collect();
    match fuzzy.len() {
        1 => Ok(fuzzy[0].id.clone()),
        2.. => bail!(
            "Ambiguous user '{name}'. Matches: {}. Use a full email for an exact target.",
            render_user_candidates(&fuzzy)
        ),
        _ => bail!("User '{name}' not found"),
    }
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
    let name = name.trim();
    if name.is_empty() {
        bail!("Label cannot be empty");
    }

    let labels: Vec<Label> = client
        .query_all::<LabelsResponse, Label>(
            queries::LABELS,
            json!({ "first": LOOKUP_PAGE_SIZE }),
            LOOKUP_MAX,
        )
        .await?;

    let exact_case: Vec<&Label> = labels
        .iter()
        .filter(|l| l.name.as_deref().is_some_and(|label| label == name))
        .collect();
    match exact_case.len() {
        1 => return Ok(exact_case[0].id.clone()),
        2.. => {
            bail!(
                "Ambiguous label '{name}'. Matches: {}.",
                render_label_candidates(&exact_case)
            )
        }
        _ => {}
    }

    let case_insensitive: Vec<&Label> = labels
        .iter()
        .filter(|l| equals_ignore_case(l.name.as_deref(), name))
        .collect();
    match case_insensitive.len() {
        1 => Ok(case_insensitive[0].id.clone()),
        2.. => bail!(
            "Ambiguous label '{name}'. Multiple case-insensitive matches exist: {}. Use exact case.",
            render_label_candidates(&case_insensitive)
        ),
        _ => bail!("Label '{name}' not found"),
    }
}

fn equals_ignore_case(value: Option<&str>, query: &str) -> bool {
    value.is_some_and(|v| v.eq_ignore_ascii_case(query))
}

fn contains_ignore_case(value: Option<&str>, query_lower: &str) -> bool {
    value.is_some_and(|v| v.to_lowercase().contains(query_lower))
}

fn render_user_candidates(users: &[&User]) -> String {
    let mut formatted: Vec<String> = users
        .iter()
        .map(|u| {
            let display = u
                .display_name
                .as_deref()
                .or(u.name.as_deref())
                .unwrap_or("Unknown");
            match u.email.as_deref() {
                Some(email) if !email.is_empty() => format!("{display} <{email}>"),
                _ => display.to_string(),
            }
        })
        .collect();

    if formatted.len() > CANDIDATE_PREVIEW_LIMIT {
        let extra = formatted.len() - CANDIDATE_PREVIEW_LIMIT;
        formatted.truncate(CANDIDATE_PREVIEW_LIMIT);
        format!("{} (+{} more)", formatted.join(", "), extra)
    } else {
        formatted.join(", ")
    }
}

fn render_label_candidates(labels: &[&Label]) -> String {
    let mut formatted: Vec<String> = labels
        .iter()
        .map(|l| {
            let name = l.name.as_deref().unwrap_or("Unnamed");
            format!("{name} [{}]", l.id)
        })
        .collect();

    if formatted.len() > CANDIDATE_PREVIEW_LIMIT {
        let extra = formatted.len() - CANDIDATE_PREVIEW_LIMIT;
        formatted.truncate(CANDIDATE_PREVIEW_LIMIT);
        format!("{} (+{} more)", formatted.join(", "), extra)
    } else {
        formatted.join(", ")
    }
}
