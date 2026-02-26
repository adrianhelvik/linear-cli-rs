use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "linear", about = "CLI for the Linear API", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Authenticate with Linear
    Auth,
    /// Show the authenticated user
    Me,
    /// Manage issues
    Issue {
        #[command(subcommand)]
        command: IssueCommands,
    },
    /// Manage teams
    Team {
        #[command(subcommand)]
        command: TeamCommands,
    },
}

#[derive(Subcommand)]
pub enum IssueCommands {
    /// List issues with filters
    List(ListArgs),
    /// View issue details
    View {
        /// Issue ID or identifier (e.g. ENG-123)
        id: String,
    },
    /// Create a new issue
    Create(CreateArgs),
    /// Update an existing issue
    Update(UpdateArgs),
    /// Search issues
    Search {
        /// Search query
        query: String,
    },
    /// Assign an issue to a user
    Assign {
        /// Issue ID or identifier
        id: String,
        /// User to assign (name/email, or omit to unassign)
        user: Option<String>,
    },
    /// Change issue workflow state
    State {
        /// Issue ID or identifier
        id: String,
        /// State name (e.g. "In Progress")
        state: Option<String>,
    },
}

#[derive(clap::Args)]
pub struct ListArgs {
    /// Filter by team key (e.g. ENG)
    #[arg(long)]
    pub team: Option<String>,
    /// Filter by state name
    #[arg(long)]
    pub state: Option<String>,
    /// Filter by assignee (name/email, or "me")
    #[arg(long)]
    pub assignee: Option<String>,
    /// Filter by priority (0=none, 1=urgent, 2=high, 3=medium, 4=low)
    #[arg(long)]
    pub priority: Option<i32>,
    /// Filter by label name
    #[arg(long)]
    pub label: Option<String>,
    /// Filter by project name
    #[arg(long)]
    pub project: Option<String>,
    /// Include completed and canceled issues
    #[arg(long)]
    pub all: bool,
    /// Maximum number of issues to return
    #[arg(long, default_value = "50")]
    pub limit: i32,
}

#[derive(clap::Args)]
pub struct CreateArgs {
    /// Team key (e.g. ENG)
    #[arg(long)]
    pub team: Option<String>,
    /// Issue title
    #[arg(long)]
    pub title: Option<String>,
    /// Issue description
    #[arg(long)]
    pub description: Option<String>,
    /// Priority (0=none, 1=urgent, 2=high, 3=medium, 4=low)
    #[arg(long)]
    pub priority: Option<i32>,
    /// State name
    #[arg(long)]
    pub state: Option<String>,
    /// Assignee (name/email, or "me")
    #[arg(long)]
    pub assignee: Option<String>,
    /// Label name
    #[arg(long)]
    pub label: Option<String>,
}

#[derive(clap::Args)]
pub struct UpdateArgs {
    /// Issue ID or identifier
    pub id: String,
    /// New title
    #[arg(long)]
    pub title: Option<String>,
    /// New description
    #[arg(long)]
    pub description: Option<String>,
    /// New priority
    #[arg(long)]
    pub priority: Option<i32>,
    /// New state name
    #[arg(long)]
    pub state: Option<String>,
    /// New assignee (name/email, "me", or "" to unassign)
    #[arg(long)]
    pub assignee: Option<String>,
    /// New label
    #[arg(long)]
    pub label: Option<String>,
}

#[derive(Subcommand)]
pub enum TeamCommands {
    /// List all teams
    List,
}
