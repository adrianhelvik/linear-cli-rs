use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::types::*;
use crate::api::{mutations, queries, resolve};
use crate::cli::UpdateArgs;
use crate::config;

pub async fn run(args: UpdateArgs) -> Result<()> {
    let client = LinearClient::new(config::api_key()?);
    let issue_id = args.id;

    let mut input = json!({});
    let mut has_update = false;

    if let Some(title) = &args.title {
        input["title"] = json!(title);
        has_update = true;
    }

    if let Some(desc) = &args.description {
        input["description"] = json!(desc);
        has_update = true;
    }

    if let Some(p) = args.priority {
        input["priority"] = json!(p);
        has_update = true;
    }

    let needs_issue_context =
        args.state.is_some() || !args.add_label.is_empty() || !args.remove_label.is_empty();
    let issue_context = if needs_issue_context {
        let issue_resp: IssueResponse = client
            .query(queries::ISSUE, json!({ "id": &issue_id }))
            .await?;
        Some(issue_resp.issue)
    } else {
        None
    };

    if let Some(state_name) = &args.state {
        let team_id = issue_context
            .as_ref()
            .and_then(|issue| issue.team.as_ref())
            .map(|t| t.id.clone())
            .ok_or_else(|| anyhow::anyhow!("Issue has no team"))?;
        let sid = resolve::state_id(&client, &team_id, state_name).await?;
        input["stateId"] = json!(sid);
        has_update = true;
    }

    if let Some(assignee) = &args.assignee {
        if assignee.is_empty() {
            input["assigneeId"] = json!(null);
        } else {
            let uid = resolve::user_id(&client, assignee).await?;
            input["assigneeId"] = json!(uid);
        }
        has_update = true;
    }

    if args.clear_labels || !args.add_label.is_empty() || !args.remove_label.is_empty() {
        if args.clear_labels {
            input["labelIds"] = json!([]);
        } else {
            let issue = issue_context
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Unable to load current issue labels"))?;
            let mut label_ids: Vec<String> = issue
                .labels
                .as_ref()
                .map(|labels| labels.nodes.iter().map(|l| l.id.clone()).collect())
                .unwrap_or_default();

            for label in &args.add_label {
                let lid = resolve::label_id(&client, label).await?;
                if !label_ids.iter().any(|id| id == &lid) {
                    label_ids.push(lid);
                }
            }

            for label in &args.remove_label {
                let lid = resolve::label_id(&client, label).await?;
                label_ids.retain(|id| id != &lid);
            }

            input["labelIds"] = json!(label_ids);
        }
        has_update = true;
    }

    if !has_update {
        anyhow::bail!(
            "No updates specified. Use --title, --description, --priority, --state, --assignee, --add-label, --remove-label, or --clear-labels."
        );
    }

    let resp: IssueUpdateResponse = client
        .query(
            mutations::ISSUE_UPDATE,
            json!({ "id": &issue_id, "input": input }),
        )
        .await?;

    if resp.issue_update.success {
        if let Some(issue) = resp.issue_update.issue {
            let id = issue.identifier.as_deref().unwrap_or(&issue.id);
            println!("Updated {id}");
        }
    } else {
        anyhow::bail!("Failed to update issue");
    }

    Ok(())
}
