use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashSet;

use super::types::{ConnectionResponse, GraphQLResponse};

pub struct LinearClient {
    client: Client,
    api_key: String,
    endpoint: String,
}

impl LinearClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            endpoint: "https://api.linear.app/graphql".into(),
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
            .post(&self.endpoint)
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

        if let Some(errors) = gql.errors
            && !errors.is_empty()
        {
            let msgs: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
            bail!("GraphQL errors: {}", msgs.join("; "));
        }

        gql.data.context("No data in GraphQL response")
    }

    /// Page through a connection query until exhausted or `max_items` is
    /// reached. The query must accept an `$after: String` variable and
    /// select `pageInfo { hasNextPage endCursor }`; page size is whatever
    /// `variables["first"]` is set to. Missing or cyclic pagination metadata is
    /// an error. Reference resolution uses `usize::MAX` to avoid partial matches.
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
        if !variables.is_object() {
            bail!("Pagination variables must be a JSON object");
        }
        if variables.get("first").is_none() {
            variables["first"] = json!(250);
        }
        let mut nodes: Vec<T> = Vec::new();
        let mut cursors = HashSet::new();
        if let Some(cursor) = variables.get("after").and_then(Value::as_str) {
            cursors.insert(cursor.to_string());
        }
        loop {
            let resp: R = self.query(query, variables.clone()).await?;
            let conn = resp.connection();
            let page = conn
                .page_info
                .context("Missing pagination metadata in Linear response")?;
            nodes.extend(conn.nodes);
            if nodes.len() >= max_items {
                nodes.truncate(max_items);
                break;
            }
            if !page.has_next_page {
                break;
            }
            let cursor = page
                .end_cursor
                .filter(|cursor| !cursor.is_empty())
                .context("Linear returned another page without a pagination cursor")?;
            if !cursors.insert(cursor.clone()) {
                bail!("Linear returned a repeated pagination cursor");
            }
            variables["after"] = json!(cursor);
        }
        Ok(nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{queries, resolve, types::*};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::task::JoinHandle;

    async fn mock(responses: Vec<Value>) -> (LinearClient, JoinHandle<Vec<Value>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let mut requests = Vec::new();
            for response in responses {
                let (mut stream, _) =
                    tokio::time::timeout(Duration::from_secs(5), listener.accept())
                        .await
                        .unwrap()
                        .unwrap();
                let request = tokio::time::timeout(Duration::from_secs(5), async {
                    let mut bytes = Vec::new();
                    let mut buffer = [0; 1024];
                    let header_end = loop {
                        let n = stream.read(&mut buffer).await.unwrap();
                        assert_ne!(n, 0, "request must contain complete headers");
                        bytes.extend_from_slice(&buffer[..n]);
                        if let Some(start) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                            break start + 4;
                        }
                    };
                    let headers = std::str::from_utf8(&bytes[..header_end]).unwrap();
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse().unwrap())
                        })
                        .expect("request content length");
                    while bytes.len() < header_end + length {
                        let n = stream.read(&mut buffer).await.unwrap();
                        assert_ne!(n, 0, "request must contain complete body");
                        bytes.extend_from_slice(&buffer[..n]);
                    }
                    serde_json::from_slice(&bytes[header_end..header_end + length]).unwrap()
                })
                .await
                .unwrap();
                requests.push(request);
                let body = response.to_string();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream.write_all(response.as_bytes()).await.unwrap();
            }
            requests
        });
        (
            LinearClient {
                client: Client::new(),
                api_key: "test-fixture".into(),
                endpoint,
            },
            server,
        )
    }

    fn page(field: &str, nodes: Value, has_next: bool, cursor: Option<&str>) -> Value {
        json!({ "data": { field: { "nodes": nodes, "pageInfo": { "hasNextPage": has_next, "endCursor": cursor } } } })
    }

    #[tokio::test]
    async fn user_resolution_finds_targets_on_later_pages() {
        let (client, server) = mock(vec![
            page(
                "users",
                json!([{ "id": "other", "name": "Alex" }]),
                true,
                Some("page-1"),
            ),
            page(
                "users",
                json!([{ "id": "target", "name": "Bea" }]),
                false,
                None,
            ),
        ])
        .await;
        let result = resolve::user_id(&client, "Bea").await;
        assert_eq!(result.unwrap(), "target");
        let requests = server.await.unwrap();
        assert_eq!(requests[0]["variables"]["first"], 250);
        assert!(requests[0]["variables"]["after"].is_null());
        assert_eq!(requests[1]["variables"]["after"], "page-1");
    }

    #[tokio::test]
    async fn a_later_exact_user_match_makes_an_early_match_ambiguous() {
        let (client, server) = mock(vec![
            page(
                "users",
                json!([{ "id": "first", "name": "Alex", "email": "a@example.test" }]),
                true,
                Some("next"),
            ),
            page(
                "users",
                json!([{ "id": "second", "name": "Alex", "email": "b@example.test" }]),
                false,
                None,
            ),
        ])
        .await;
        let result = resolve::user_id(&client, "Alex").await;
        let message = result.unwrap_err().to_string();
        assert!(message.contains("Ambiguous user"));
        assert!(message.contains("a@example.test") && message.contains("b@example.test"));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn duplicate_labels_across_pages_are_ambiguous() {
        let (client, server) = mock(vec![
            page(
                "issueLabels",
                json!([{ "id": "team-a", "name": "Bug" }]),
                true,
                Some("next"),
            ),
            page(
                "issueLabels",
                json!([{ "id": "team-b", "name": "Bug" }]),
                false,
                None,
            ),
        ])
        .await;
        let result = resolve::label_id(&client, "Bug").await;
        assert!(result.unwrap_err().to_string().contains("Ambiguous label"));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn incomplete_or_cycling_pagination_is_an_error() {
        let cases = [
            vec![json!({ "data": { "users": { "nodes": [] } } })],
            vec![page("users", json!([]), true, None)],
            vec![page("users", json!([]), true, Some(""))],
            vec![
                page("users", json!([]), true, Some("same")),
                page("users", json!([]), true, Some("same")),
            ],
            vec![
                page("users", json!([]), true, Some("a")),
                page("users", json!([]), true, Some("b")),
                page("users", json!([]), true, Some("a")),
            ],
        ];
        for responses in cases {
            let (client, server) = mock(responses).await;
            let result = client
                .query_all::<UsersResponse, User>(
                    queries::USERS,
                    json!({ "first": 250 }),
                    usize::MAX,
                )
                .await;
            server.await.unwrap();
            assert!(result.unwrap_err().to_string().contains("pagination"));
        }
    }

    #[tokio::test]
    async fn a_failed_later_page_does_not_resolve_from_partial_results() {
        let (client, server) = mock(vec![
            page(
                "users",
                json!([{ "id": "first", "name": "Alex" }]),
                true,
                Some("next"),
            ),
            json!({ "errors": [{ "message": "rate limited" }] }),
        ])
        .await;
        let result = resolve::user_id(&client, "Alex").await;
        assert!(result.unwrap_err().to_string().contains("rate limited"));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn an_empty_complete_collection_needs_only_one_request() {
        let (client, server) = mock(vec![page("users", json!([]), false, None)]).await;
        let users = client
            .query_all::<UsersResponse, User>(queries::USERS, json!({}), usize::MAX)
            .await
            .unwrap();
        assert!(users.is_empty());
        assert_eq!(server.await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn team_and_state_resolution_paginate_and_preserve_filters() {
        let (client, server) = mock(vec![
            page("teams", json!([]), true, Some("teams-next")),
            page(
                "teams",
                json!([{ "id": "team", "key": "ENG" }]),
                false,
                None,
            ),
            page("workflowStates", json!([]), true, Some("states-next")),
            page(
                "workflowStates",
                json!([{ "id": "state", "name": "Done" }]),
                false,
                None,
            ),
        ])
        .await;
        assert_eq!(resolve::team_id(&client, "eng").await.unwrap(), "team");
        assert_eq!(
            resolve::state_id(&client, "team", "done").await.unwrap(),
            "state"
        );
        let requests = server.await.unwrap();
        assert_eq!(requests[1]["variables"]["after"], "teams-next");
        assert_eq!(requests[3]["variables"]["after"], "states-next");
        for request in &requests[2..] {
            assert_eq!(
                request["variables"]["filter"],
                json!({ "team": { "id": { "eq": "team" } } })
            );
        }
    }

    #[tokio::test]
    async fn exact_email_disambiguates_users_across_pages() {
        let (client, server) = mock(vec![
            page(
                "users",
                json!([{ "id": "first", "name": "Alex", "email": "a@example.test" }]),
                true,
                Some("next"),
            ),
            page(
                "users",
                json!([{ "id": "second", "name": "Alex", "email": "b@example.test" }]),
                false,
                None,
            ),
        ])
        .await;
        assert_eq!(
            resolve::user_id(&client, "B@EXAMPLE.TEST").await.unwrap(),
            "second"
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn the_me_shortcut_does_not_load_the_workspace_directory() {
        let (client, server) = mock(vec![json!({ "data": { "viewer": { "id": "self" } } })]).await;
        assert_eq!(resolve::user_id(&client, " ME ").await.unwrap(), "self");
        let requests = server.await.unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0]["query"], queries::VIEWER);
    }

    #[tokio::test]
    async fn explicit_list_limits_keep_their_bounded_pagination_contract() {
        let (client, server) = mock(vec![page(
            "users",
            json!([{ "id": "u1" }, { "id": "u2" }]),
            true,
            Some("later"),
        )])
        .await;
        let users = client
            .query_all::<UsersResponse, User>(queries::USERS, json!({ "first": 1 }), 1)
            .await
            .unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].id, "u1");
        let requests = server.await.unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0]["variables"]["first"], 1);
    }

    #[tokio::test]
    async fn non_object_pagination_variables_are_rejected_before_network_io() {
        let result = LinearClient::new(String::new())
            .query_all::<UsersResponse, User>(queries::USERS, json!([]), usize::MAX)
            .await;
        assert!(result.unwrap_err().to_string().contains("JSON object"));
    }
}
