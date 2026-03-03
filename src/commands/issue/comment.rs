use anyhow::{Result, bail};
use serde_json::json;
use std::io::{IsTerminal, Read};

use crate::api::client::LinearClient;
use crate::api::mutations;
use crate::api::queries;
use crate::api::types::{CommentCreateResponse, IssueResponse};
use crate::config;

pub async fn run(id: String, body: Option<String>, json_output: bool) -> Result<()> {
    let body = match body {
        Some(b) => b,
        None => read_body_from_stdin()?,
    };

    if body.trim().is_empty() {
        bail!("Comment body cannot be empty");
    }

    let client = LinearClient::new(config::api_key()?);

    // Resolve identifier to ID if needed
    let issue_id = if id.contains('-') {
        let resp: IssueResponse = client.query(queries::ISSUE, json!({ "id": id })).await?;
        resp.issue.id
    } else {
        id
    };

    let resp: CommentCreateResponse = client
        .query(
            mutations::COMMENT_CREATE,
            json!({ "input": { "issueId": issue_id, "body": body } }),
        )
        .await?;

    if resp.comment_create.success {
        if json_output {
            if let Some(comment) = &resp.comment_create.comment {
                println!("{}", serde_json::to_string_pretty(comment)?);
            }
        } else if let Some(comment) = &resp.comment_create.comment {
            let author = comment
                .user
                .as_ref()
                .and_then(|u| u.display_name.as_deref().or(u.name.as_deref()))
                .unwrap_or("you");
            println!("Comment added by {author}");
        } else {
            println!("Comment added.");
        }
    } else {
        bail!("Failed to create comment");
    }
    Ok(())
}

fn read_body_from_stdin() -> Result<String> {
    if std::io::stdin().is_terminal() {
        bail!("Provide comment body with -b/--body, or pipe via stdin");
    }
    let mut body = String::new();
    std::io::stdin().read_to_string(&mut body)?;
    Ok(body)
}
