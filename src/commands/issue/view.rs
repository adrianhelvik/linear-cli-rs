use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries;
use crate::api::types::{IssueCommentsResponse, IssueResponse};
use crate::config;
use crate::output;

pub async fn run(id: String, json: bool) -> Result<()> {
    let client = LinearClient::new(config::api_key()?);
    let resp: IssueResponse = client.query(queries::ISSUE, json!({ "id": id })).await?;
    let mut issue = resp.issue;

    // Fetch remaining comment pages, if any.
    if let Some(comments) = issue.comments.as_mut() {
        let mut cursor = comments
            .page_info
            .as_ref()
            .filter(|p| p.has_next_page)
            .and_then(|p| p.end_cursor.clone());
        while let Some(after) = cursor {
            let page: IssueCommentsResponse = client
                .query(queries::ISSUE_COMMENTS, json!({ "id": id, "after": after }))
                .await?;
            let conn = page.issue.comments;
            comments.nodes.extend(conn.nodes);
            cursor = conn
                .page_info
                .filter(|p| p.has_next_page)
                .and_then(|p| p.end_cursor);
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&issue)?);
    } else {
        output::issue_detail(&issue);
    }
    Ok(())
}
