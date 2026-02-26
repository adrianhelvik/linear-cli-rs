use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries;
use crate::api::types::*;
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

    if let Some(assignee) = &args.assignee {
        if assignee == "me" {
            let viewer: ViewerResponse =
                client.query(queries::VIEWER, json!({})).await?;
            filter["assignee"] = json!({ "id": { "eq": viewer.viewer.id } });
        } else {
            filter["assignee"] =
                json!({ "displayName": { "containsIgnoreCase": assignee } });
        }
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
