use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::types::{ConnectionResponse, GraphQLResponse};

pub struct LinearClient {
    client: Client,
    api_key: String,
}

impl LinearClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn query<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
        variables: Value,
    ) -> Result<T> {
        let body = json!({
            "query": query,
            "variables": variables,
        });

        let resp = self
            .client
            .post("https://api.linear.app/graphql")
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to reach Linear API")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            bail!("Linear API returned {status}: {text}");
        }

        let gql: GraphQLResponse<T> = resp
            .json()
            .await
            .context("Failed to parse Linear API response")?;

        if let Some(errors) = gql.errors {
            if !errors.is_empty() {
                let msgs: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
                bail!("GraphQL errors: {}", msgs.join("; "));
            }
        }

        gql.data.context("No data in GraphQL response")
    }

    /// Page through a connection query until exhausted or `max_items` is
    /// reached. The query must accept an `$after: String` variable and
    /// select `pageInfo { hasNextPage endCursor }`; page size is whatever
    /// `variables["first"]` is set to.
    pub async fn query_all<R, T>(
        &self,
        query: &str,
        mut variables: Value,
        max_items: usize,
    ) -> Result<Vec<T>>
    where
        R: for<'de> Deserialize<'de> + ConnectionResponse<T>,
        T: Serialize,
    {
        let mut nodes: Vec<T> = Vec::new();
        loop {
            let resp: R = self.query(query, variables.clone()).await?;
            let conn = resp.connection();
            nodes.extend(conn.nodes);
            if nodes.len() >= max_items {
                nodes.truncate(max_items);
                break;
            }
            let next = conn
                .page_info
                .filter(|p| p.has_next_page)
                .and_then(|p| p.end_cursor);
            match next {
                Some(cursor) => variables["after"] = json!(cursor),
                None => break,
            }
        }
        Ok(nodes)
    }
}
