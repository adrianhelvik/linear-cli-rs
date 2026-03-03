use chrono::{DateTime, Utc};
use colored::Colorize;
use crossterm::terminal;
use std::io::IsTerminal;
use tabled::builder::Builder;
use tabled::settings::Style;

use crate::api::types::{Issue, Team};

const MIN_TITLE_WIDTH: usize = 16;
const PREFERRED_TITLE_WIDTH: usize = 28;
const MAX_TITLE_WIDTH: usize = 110;
const MAX_ID_WIDTH: usize = 16;
const MAX_STATE_WIDTH: usize = 20;
const MAX_ASSIGNEE_WIDTH: usize = 24;
const TABLE_OVERHEAD: usize = 18;
const PRIORITY_WIDTH: usize = 8;

struct IssueTableWidths {
    id: usize,
    title: usize,
    state: usize,
    assignee: usize,
}

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

fn terminal_width() -> usize {
    terminal::size()
        .map(|(w, _)| w as usize)
        .ok()
        .or_else(|| {
            std::env::var("COLUMNS")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
        })
        .unwrap_or(120)
}

fn truncate_visible(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    if max_chars == 1 {
        return "…".to_string();
    }
    format!("{}…", text.chars().take(max_chars - 1).collect::<String>())
}

fn supports_hyperlinks() -> bool {
    if !std::io::stdout().is_terminal() {
        return false;
    }
    !matches!(std::env::var("TERM").ok().as_deref(), Some("dumb"))
}

fn hyperlink(label: &str, url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{label}\x1b]8;;\x1b\\")
}

fn compute_issue_table_widths(issues: &[Issue]) -> IssueTableWidths {
    let id_width = issues
        .iter()
        .map(|issue| issue.identifier.as_deref().unwrap_or("—").chars().count())
        .max()
        .unwrap_or(2)
        .clamp(2, MAX_ID_WIDTH);

    let state_width = issues
        .iter()
        .map(|issue| {
            issue
                .state
                .as_ref()
                .and_then(|state| state.name.as_deref())
                .unwrap_or("—")
                .chars()
                .count()
        })
        .max()
        .unwrap_or(5)
        .clamp(5, MAX_STATE_WIDTH);

    let assignee_width = issues
        .iter()
        .map(|issue| {
            issue
                .assignee
                .as_ref()
                .and_then(|user| user.display_name.as_deref().or(user.name.as_deref()))
                .unwrap_or("Unassigned")
                .chars()
                .count()
        })
        .max()
        .unwrap_or(10)
        .clamp(10, MAX_ASSIGNEE_WIDTH);

    let available_for_title = terminal_width()
        .saturating_sub(id_width + state_width + assignee_width + PRIORITY_WIDTH + TABLE_OVERHEAD);
    let title_width = if available_for_title >= PREFERRED_TITLE_WIDTH {
        available_for_title.min(MAX_TITLE_WIDTH)
    } else {
        available_for_title.max(MIN_TITLE_WIDTH)
    };

    IssueTableWidths {
        id: id_width,
        title: title_width,
        state: state_width,
        assignee: assignee_width,
    }
}

pub fn issue_table(issues: &[Issue]) {
    if issues.is_empty() {
        println!("No issues found.");
        return;
    }

    let mut builder = Builder::new();
    builder.push_record(["ID", "Title", "State", "Priority", "Assignee"]);
    let widths = compute_issue_table_widths(issues);
    let enable_hyperlinks = supports_hyperlinks();

    for issue in issues {
        let id = issue.identifier.as_deref().unwrap_or("—");
        let id_plain = truncate_visible(id, widths.id);
        let id = if enable_hyperlinks {
            issue
                .url
                .as_deref()
                .map(|url| hyperlink(&id_plain, url))
                .unwrap_or(id_plain)
        } else {
            id_plain
        };

        let title = issue.title.as_deref().unwrap_or("—");
        let title = truncate_visible(title, widths.title);
        let state = issue
            .state
            .as_ref()
            .map(|s| {
                let name = s.name.as_deref().unwrap_or("—");
                let name = truncate_visible(name, widths.state);
                let st = s.state_type.as_deref().unwrap_or("");
                state_colored(&name, st)
            })
            .unwrap_or_else(|| "—".to_string());
        let priority = priority_label(issue.priority.unwrap_or(0));
        let assignee = issue
            .assignee
            .as_ref()
            .and_then(|u| u.display_name.as_deref().or(u.name.as_deref()))
            .unwrap_or("Unassigned");
        let assignee = truncate_visible(assignee, widths.assignee);

        builder.push_record([&id, &title, &state, priority, &assignee]);
    }

    let mut table = builder.build();
    table.with(Style::rounded());
    println!("{table}");
}

fn relative_time(iso: &str) -> String {
    let Ok(dt) = iso.parse::<DateTime<Utc>>() else {
        return iso.to_string();
    };
    let now = Utc::now();
    let delta = now.signed_duration_since(dt);

    if delta.num_seconds() < 0 {
        return "just now".to_string();
    }

    let secs = delta.num_seconds();
    if secs < 60 {
        return "just now".to_string();
    }
    let mins = delta.num_minutes();
    if mins < 60 {
        return format!("{mins}m ago");
    }
    let hours = delta.num_hours();
    if hours < 24 {
        return format!("{hours}h ago");
    }
    let days = delta.num_days();
    if days < 30 {
        return format!("{days}d ago");
    }
    if days < 365 {
        let months = days / 30;
        return format!("{months}mo ago");
    }
    let years = days / 365;
    format!("{years}y ago")
}

fn section_rule(label: &str, width: usize) -> String {
    let prefix = format!("── {label} ");
    let remaining = width.saturating_sub(prefix.len());
    format!("{}{}", prefix, "─".repeat(remaining)).dimmed().to_string()
}

pub fn issue_detail(issue: &Issue) {
    let width = terminal_width().min(80);
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
        .and_then(|p| p.name.as_deref());
    let url = issue.url.as_deref().unwrap_or("—");

    // Header
    println!("{}", format!("{id}  {title}").bold());
    println!();

    // Two-column metadata
    println!(
        "  {:<10} {:<26} {:<10} {}",
        "State:".dimmed(),
        state,
        "Priority:".dimmed(),
        priority
    );
    println!(
        "  {:<10} {:<26} {:<10} {}",
        "Team:".dimmed(),
        team,
        "Assignee:".dimmed(),
        assignee
    );
    if !labels.is_empty() {
        println!("  {:<10} {}", "Labels:".dimmed(), labels);
    }
    if let Some(proj) = project {
        println!("  {:<10} {}", "Project:".dimmed(), proj);
    }
    println!("  {:<10} {}", "URL:".dimmed(), url);

    // Description
    if let Some(desc) = &issue.description {
        if !desc.is_empty() {
            println!();
            println!("{}", section_rule("Description", width));
            println!();
            for line in desc.lines() {
                println!("  {line}");
            }
        }
    }

    // Comments
    if let Some(comments) = &issue.comments {
        if !comments.nodes.is_empty() {
            let count = comments.nodes.len();
            println!();
            println!(
                "{}",
                section_rule(&format!("Comments ({count})"), width)
            );

            for comment in &comments.nodes {
                let author = comment
                    .user
                    .as_ref()
                    .and_then(|u| u.display_name.as_deref().or(u.name.as_deref()))
                    .unwrap_or("Unknown");
                let time = comment
                    .created_at
                    .as_deref()
                    .map(relative_time)
                    .unwrap_or_default();

                println!();
                println!(
                    "  {}  {}",
                    author.bold(),
                    time.dimmed()
                );
                if let Some(body) = &comment.body {
                    for line in body.lines() {
                        println!("  {line}");
                    }
                }
            }
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
