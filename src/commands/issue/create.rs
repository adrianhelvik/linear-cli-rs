use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::{mutations, queries, resolve};
use crate::api::types::*;
use crate::cli::CreateArgs;
use crate::config;
use crate::interactive;

struct TeamOption {
    id: String,
    key: String,
    name: String,
}

impl std::fmt::Display for TeamOption {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} — {}", self.key, self.name)
    }
}

struct StateOption {
    id: String,
    name: String,
}

impl std::fmt::Display for StateOption {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub async fn run(args: CreateArgs) -> Result<()> {
    let client = LinearClient::new(config::api_key()?);

    // Resolve team
    let team_id = if let Some(key) = &args.team {
        resolve::team_id(&client, key).await?
    } else {
        let teams: TeamsResponse = client.query(queries::TEAMS, json!({})).await?;
        let options: Vec<TeamOption> = teams
            .teams
            .nodes
            .into_iter()
            .map(|t| TeamOption {
                id: t.id,
                key: t.key.unwrap_or_default(),
                name: t.name.unwrap_or_default(),
            })
            .collect();
        let selected = interactive::select("Select a team:", options)?;
        selected.id
    };

    // Get title
    let title = interactive::require_text("title", "Issue title:", args.title)?;

    // Build mutation input
    let mut input = json!({
        "teamId": team_id,
        "title": title,
    });

    if let Some(desc) = args.description {
        input["description"] = json!(desc);
    }

    if let Some(p) = args.priority {
        input["priority"] = json!(p);
    }

    if let Some(state_name) = &args.state {
        let sid = resolve::state_id(&client, &team_id, state_name).await?;
        input["stateId"] = json!(sid);
    } else if interactive::is_tty() {
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
        if !options.is_empty() {
            let selected = interactive::select("Select initial state:", options)?;
            input["stateId"] = json!(selected.id);
        }
    }

    if let Some(assignee) = &args.assignee {
        let uid = resolve::user_id(&client, assignee).await?;
        input["assigneeId"] = json!(uid);
    }

    if let Some(label) = &args.label {
        let lid = resolve::label_id(&client, label).await?;
        input["labelIds"] = json!([lid]);
    }

    let resp: IssueCreateResponse = client
        .query(mutations::ISSUE_CREATE, json!({ "input": input }))
        .await?;

    if resp.issue_create.success {
        if let Some(issue) = resp.issue_create.issue {
            let id = issue.identifier.as_deref().unwrap_or(&issue.id);
            let title = issue.title.as_deref().unwrap_or("");
            let url = issue.url.as_deref().unwrap_or("");
            println!("Created {id}: {title}");
            if !url.is_empty() {
                println!("{url}");
            }
        }
    } else {
        anyhow::bail!("Failed to create issue");
    }

    Ok(())
}
