use anyhow::Result;
use serde_json::json;

use crate::api::client::LinearClient;
use crate::api::queries;
use crate::api::types::IssueResponse;
use crate::config;
use crate::output;

pub async fn run(id: String) -> Result<()> {
    let client = LinearClient::new(config::api_key()?);
    let resp: IssueResponse = client.query(queries::ISSUE, json!({ "id": id })).await?;
    output::issue_detail(&resp.issue);
    Ok(())
}
