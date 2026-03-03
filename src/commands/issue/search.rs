use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries;
use crate::api::types::IssueSearchResponse;
use crate::config;
use crate::output;

pub async fn run(query: String, json: bool) -> Result<()> {
    let client = LinearClient::new(config::api_key()?);
    let resp: IssueSearchResponse = client
        .query(
            queries::ISSUE_SEARCH,
            json!({ "query": query, "first": 50 }),
        )
        .await?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&resp.issue_search.nodes)?
        );
    } else {
        output::issue_table(&resp.issue_search.nodes);
    }
    Ok(())
}
