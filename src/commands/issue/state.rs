use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::{mutations, queries, resolve};
use crate::api::types::*;
use crate::config;
use crate::interactive;

struct StateOption {
    id: String,
    name: String,
}

impl std::fmt::Display for StateOption {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub async fn run(id: String, state: Option<String>) -> Result<()> {
    let client = LinearClient::new(config::api_key()?);

    // Fetch issue to get team ID
    let issue_resp: IssueResponse =
        client.query(queries::ISSUE, json!({ "id": id })).await?;
    let team_id = issue_resp
        .issue
        .team
        .as_ref()
        .map(|t| t.id.clone())
        .ok_or_else(|| anyhow::anyhow!("Issue has no team"))?;

    // Resolve state
    let (state_id, state_name) = if let Some(name) = &state {
        let sid = resolve::state_id(&client, &team_id, name).await?;
        (sid, name.clone())
    } else {
        let states: WorkflowStatesResponse = client
            .query(
                queries::WORKFLOW_STATES,
                json!({ "filter": { "team": { "id": { "eq": team_id } } } }),
            )
            .await?;

        let options: Vec<StateOption> = states
            .workflow_states
            .nodes
            .into_iter()
            .map(|s| StateOption {
                id: s.id,
                name: s.name.unwrap_or_default(),
            })
            .collect();

        let selected = interactive::select("Select state:", options, "state")?;
        let name = selected.name.clone();
        (selected.id, name)
    };

    let resp: IssueUpdateResponse = client
        .query(
            mutations::ISSUE_UPDATE,
            json!({ "id": id, "input": { "stateId": state_id } }),
        )
        .await?;

    if resp.issue_update.success {
        if let Some(issue) = resp.issue_update.issue {
            let identifier = issue.identifier.as_deref().unwrap_or(&issue.id);
            println!("{identifier}: → {state_name}");
        }
    } else {
        anyhow::bail!("Failed to update issue state");
    }

    Ok(())
}
