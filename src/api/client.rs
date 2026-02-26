use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};

use super::types::GraphQLResponse;

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
}
