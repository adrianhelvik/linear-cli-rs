use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::{mutations, queries, resolve};
use crate::api::types::*;
use crate::cli::UpdateArgs;
use crate::config;

pub async fn run(args: UpdateArgs) -> Result<()> {
    let client = LinearClient::new(config::api_key()?);

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

    if let Some(state_name) = &args.state {
        // Fetch issue to get team ID for state resolution
        let issue_resp: IssueResponse =
            client.query(queries::ISSUE, json!({ "id": args.id })).await?;
        let team_id = issue_resp
            .issue
            .team
            .as_ref()
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

    if let Some(label) = &args.label {
        let lid = resolve::label_id(&client, label).await?;
        input["labelIds"] = json!([lid]);
        has_update = true;
    }

    if !has_update {
        anyhow::bail!(
            "No updates specified. Use --title, --description, --priority, --state, --assignee, or --label."
        );
    }

    let resp: IssueUpdateResponse = client
        .query(
            mutations::ISSUE_UPDATE,
            json!({ "id": args.id, "input": input }),
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
