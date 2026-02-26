use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::types::IssueUpdateResponse;
use crate::api::{mutations, resolve};
use crate::config;

pub async fn run(id: String, user: Option<String>) -> Result<()> {
    let client = LinearClient::new(config::api_key()?);

    let (input, action) = match &user {
        Some(name) => {
            let uid = resolve::user_id(&client, name).await?;
            (json!({ "assigneeId": uid }), format!("Assigned to {name}"))
        }
        None => (json!({ "assigneeId": null }), "Unassigned".to_string()),
    };

    let resp: IssueUpdateResponse = client
        .query(mutations::ISSUE_UPDATE, json!({ "id": id, "input": input }))
        .await?;

    if resp.issue_update.success {
        if let Some(issue) = resp.issue_update.issue {
            let identifier = issue.identifier.as_deref().unwrap_or(&issue.id);
            println!("{identifier}: {action}");
        }
    } else {
        anyhow::bail!("Failed to update issue");
    }

    Ok(())
}
