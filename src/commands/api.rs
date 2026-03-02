use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::io::{IsTerminal, Read};

use crate::api::client::LinearClient;
use crate::cli::ApiArgs;
use crate::config;

pub async fn run(args: ApiArgs) -> Result<()> {
    let query = read_query(&args)?;
    if query.trim().is_empty() {
        bail!("GraphQL query cannot be empty");
    }

    let variables = read_variables(&args)?;
    let client = LinearClient::new(config::api_key()?);
    let data: Value = client.query(&query, variables).await?;
    println!("{}", serde_json::to_string_pretty(&data)?);
    Ok(())
}

fn read_query(args: &ApiArgs) -> Result<String> {
    if let Some(query) = &args.query {
        return Ok(query.clone());
    }

    if let Some(path) = &args.query_file {
        return std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read query file {}", path.display()));
    }

    if !std::io::stdin().is_terminal() {
        let mut query = String::new();
        std::io::stdin()
            .read_to_string(&mut query)
            .context("Failed to read query from stdin")?;
        return Ok(query);
    }

    bail!("Provide --query, --query-file, or pipe a query via stdin")
}

fn read_variables(args: &ApiArgs) -> Result<Value> {
    if let Some(raw) = &args.variables {
        return parse_variables(raw).context("Invalid JSON passed to --variables");
    }

    if let Some(path) = &args.variables_file {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read variables file {}", path.display()))?;
        return parse_variables(&raw)
            .with_context(|| format!("Invalid JSON in variables file {}", path.display()));
    }

    Ok(json!({}))
}

fn parse_variables(raw: &str) -> Result<Value> {
    let value: Value = serde_json::from_str(raw)?;
    if !value.is_object() {
        bail!("Variables must be a JSON object");
    }
    Ok(value)
}
