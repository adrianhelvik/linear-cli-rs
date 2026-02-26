use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries;
use crate::api::resolve;
use crate::api::types::IssuesResponse;
use crate::cli::ListArgs;
use crate::config;
use crate::output;

pub async fn run(args: ListArgs) -> Result<()> {
    let client = LinearClient::new(config::api_key()?);

    let mut filter = json!({});

    if let Some(team) = &args.team {
        filter["team"] = json!({ "key": { "eqIgnoreCase": team } });
    }

    if let Some(state) = &args.state {
        filter["state"] = json!({ "name": { "eqIgnoreCase": state } });
    } else if !args.all {
        filter["state"] = json!({ "type": { "nin": ["completed", "canceled"] } });
    }

    let resolved_assignee = if let Some(assignee) = args.assignee.as_deref() {
        Some(assignee)
    } else if args.mine {
        Some("me")
    } else if args.all_assignees || args.team.is_some() {
        None
    } else {
        // Sensible default: no assignee/team flags means "my active issues".
        Some("me")
    };

    if let Some(assignee) = resolved_assignee {
        let assignee_id = resolve::user_id(&client, assignee).await?;
        filter["assignee"] = json!({ "id": { "eq": assignee_id } });
    }

    if let Some(priority) = args.priority {
        filter["priority"] = json!({ "eq": priority });
    }

    if let Some(label) = &args.label {
        filter["labels"] = json!({ "some": { "name": { "eqIgnoreCase": label } } });
    }

    if let Some(project) = &args.project {
        filter["project"] = json!({ "name": { "containsIgnoreCase": project } });
    }

    let variables = json!({
        "filter": filter,
        "first": args.limit,
    });

    let resp: IssuesResponse = client.query(queries::ISSUES, variables).await?;
    output::issue_table(&resp.issues.nodes);
    Ok(())
}
