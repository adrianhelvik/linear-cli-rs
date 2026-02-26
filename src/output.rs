use colored::Colorize;
use tabled::builder::Builder;
use tabled::settings::Style;

use crate::api::types::{Issue, Team};

pub fn priority_label(p: i32) -> &'static str {
    match p {
        1 => "Urgent",
        2 => "High",
        3 => "Medium",
        4 => "Low",
        _ => "None",
    }
}

fn state_colored(name: &str, state_type: &str) -> String {
    match state_type {
        "completed" => name.green().to_string(),
        "canceled" | "cancelled" => name.red().strikethrough().to_string(),
        "started" => name.yellow().to_string(),
        "unstarted" => name.dimmed().to_string(),
        _ => name.to_string(),
    }
}

pub fn issue_table(issues: &[Issue]) {
    if issues.is_empty() {
        println!("No issues found.");
        return;
    }

    let mut builder = Builder::new();
    builder.push_record(["ID", "Title", "State", "Priority", "Assignee"]);

    for issue in issues {
        let id = issue.identifier.as_deref().unwrap_or("—");
        let title = issue.title.as_deref().unwrap_or("—");
        let title = if title.chars().count() > 60 {
            format!("{}…", title.chars().take(59).collect::<String>())
        } else {
            title.to_string()
        };
        let state = issue
            .state
            .as_ref()
            .map(|s| {
                let name = s.name.as_deref().unwrap_or("—");
                let st = s.state_type.as_deref().unwrap_or("");
                state_colored(name, st)
            })
            .unwrap_or_else(|| "—".to_string());
        let priority = priority_label(issue.priority.unwrap_or(0));
        let assignee = issue
            .assignee
            .as_ref()
            .and_then(|u| u.display_name.as_deref().or(u.name.as_deref()))
            .unwrap_or("Unassigned");

        builder.push_record([id, &title, &state, priority, assignee]);
    }

    let mut table = builder.build();
    table.with(Style::rounded());
    println!("{table}");
}

pub fn issue_detail(issue: &Issue) {
    let id = issue.identifier.as_deref().unwrap_or("—");
    let title = issue.title.as_deref().unwrap_or("—");
    let state = issue
        .state
        .as_ref()
        .map(|s| {
            let name = s.name.as_deref().unwrap_or("—");
            let st = s.state_type.as_deref().unwrap_or("");
            state_colored(name, st)
        })
        .unwrap_or_else(|| "—".to_string());
    let priority = priority_label(issue.priority.unwrap_or(0));
    let team = issue
        .team
        .as_ref()
        .and_then(|t| t.name.as_deref())
        .unwrap_or("—");
    let assignee = issue
        .assignee
        .as_ref()
        .and_then(|u| u.display_name.as_deref().or(u.name.as_deref()))
        .unwrap_or("Unassigned");
    let labels = issue
        .labels
        .as_ref()
        .map(|l| {
            l.nodes
                .iter()
                .filter_map(|l| l.name.as_deref())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let project = issue
        .project
        .as_ref()
        .and_then(|p| p.name.as_deref())
        .unwrap_or("—");
    let url = issue.url.as_deref().unwrap_or("—");

    println!("{}", format!("{id}  {title}").bold());
    println!();
    println!("State:    {state}");
    println!("Priority: {priority}");
    println!("Team:     {team}");
    println!("Assignee: {assignee}");
    if !labels.is_empty() {
        println!("Labels:   {labels}");
    }
    println!("Project:  {project}");
    println!("URL:      {url}");

    if let Some(desc) = &issue.description {
        if !desc.is_empty() {
            println!();
            println!("{}", "Description".bold());
            println!("{desc}");
        }
    }
}

pub fn team_table(teams: &[Team]) {
    if teams.is_empty() {
        println!("No teams found.");
        return;
    }

    let mut builder = Builder::new();
    builder.push_record(["Key", "Name"]);

    for team in teams {
        let key = team.key.as_deref().unwrap_or("—");
        let name = team.name.as_deref().unwrap_or("—");
        builder.push_record([key, name]);
    }

    let mut table = builder.build();
    table.with(Style::rounded());
    println!("{table}");
}
