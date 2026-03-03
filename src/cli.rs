use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "linear",
    about = "CLI for the Linear API",
    version,
    after_help = "\x1b[1mExamples:\x1b[0m
  linear issue list                  List my active issues
  linear issue DIS-510               View an issue (shorthand)
  linear issue view DIS-510 --json   View as JSON (for agents)
  linear issue create --team ENG     Create issue interactively
  linear me                          Show authenticated user
  linear api -q '{ viewer { id } }'  Run raw GraphQL"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Authenticate with Linear
    Auth(AuthArgs),
    /// Show the authenticated user
    Me,
    /// Run a raw GraphQL query/mutation against Linear
    #[command(alias = "ap")]
    Api(ApiArgs),
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
#[command(
    after_help = "\x1b[1mExamples:\x1b[0m
  linear issue list                        List my active issues
  linear issue list --team ENG --all       All ENG issues including done
  linear issue view DIS-510                View issue with comments
  linear issue DIS-510                     Shorthand for view
  linear issue view DIS-510 --json         View as JSON (for agents)
  linear issue list --json                 List as JSON (for agents)
  linear issue create --team ENG           Create issue (interactive)
  linear issue update DIS-510 --state 'In Progress'
  linear issue comment DIS-510 -b 'Fixed'  Add a comment
  echo 'details...' | linear issue comment DIS-510"
)]
pub enum IssueCommands {
    /// List issues with filters
    List(ListArgs),
    /// View issue details (aliases: get, show)
    #[command(alias = "get", alias = "show")]
    View {
        /// Issue ID or identifier (e.g. ENG-123)
        id: String,
        /// Output as JSON (for scripts and agents)
        #[arg(long)]
        json: bool,
    },
    /// Create a new issue
    Create(CreateArgs),
    /// Update an existing issue
    Update(UpdateArgs),
    /// Search issues
    Search {
        /// Search query
        query: String,
        /// Output as JSON (for scripts and agents)
        #[arg(long)]
        json: bool,
    },
    /// Assign an issue to a user
    Assign {
        /// Issue ID or identifier
        id: String,
        /// User to assign (display name/name/email, or omit to unassign)
        user: Option<String>,
    },
    /// Change issue workflow state
    State {
        /// Issue ID or identifier
        id: String,
        /// State name (e.g. "In Progress")
        state: Option<String>,
    },
    /// Add a comment to an issue
    Comment {
        /// Issue ID or identifier (e.g. ENG-123)
        id: String,
        /// Comment body (reads from stdin if omitted in non-interactive mode)
        #[arg(short, long)]
        body: Option<String>,
    },
}

/// Check if a string looks like a Linear issue identifier (e.g. ENG-123, DIS-510).
pub fn looks_like_issue_id(s: &str) -> bool {
    let Some((prefix, number)) = s.split_once('-') else {
        return false;
    };
    !prefix.is_empty()
        && prefix.chars().all(|c| c.is_ascii_alphanumeric())
        && !number.is_empty()
        && number.chars().all(|c| c.is_ascii_digit())
}

#[derive(clap::Args)]
pub struct AuthArgs {
    /// Read API key from file (recommended for scripts)
    #[arg(long = "key-file", value_name = "FILE")]
    pub key_file: Option<std::path::PathBuf>,
}

#[derive(clap::Args)]
pub struct ApiArgs {
    /// GraphQL query/mutation text
    #[arg(long, short = 'q', value_name = "QUERY", conflicts_with = "query_file")]
    pub query: Option<String>,
    /// Read GraphQL query/mutation from file
    #[arg(long = "query-file", value_name = "FILE", conflicts_with = "query")]
    pub query_file: Option<std::path::PathBuf>,
    /// Variables as JSON object
    #[arg(
        long,
        short = 'v',
        value_name = "JSON",
        conflicts_with = "variables_file"
    )]
    pub variables: Option<String>,
    /// Read variables JSON object from file
    #[arg(
        long = "variables-file",
        value_name = "FILE",
        conflicts_with = "variables"
    )]
    pub variables_file: Option<std::path::PathBuf>,
}

#[derive(clap::Args)]
pub struct ListArgs {
    /// Filter by team key (e.g. ENG)
    #[arg(long)]
    pub team: Option<String>,
    /// Filter by state name
    #[arg(long)]
    pub state: Option<String>,
    /// Filter by assignee (display name/name/email, or "me")
    #[arg(long)]
    pub assignee: Option<String>,
    /// Shortcut for --assignee me
    #[arg(long, conflicts_with_all = ["assignee", "all_assignees"])]
    pub mine: bool,
    /// Do not apply implicit "my issues" default
    #[arg(long, conflicts_with_all = ["assignee", "mine"])]
    pub all_assignees: bool,
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
    /// Output as JSON (for scripts and agents)
    #[arg(long)]
    pub json: bool,
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
    /// Assignee (display name/name/email, or "me")
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
    /// New assignee (display name/name/email, "me", or "" to unassign)
    #[arg(long)]
    pub assignee: Option<String>,
    /// Add a label (repeatable)
    #[arg(long = "add-label", value_name = "LABEL")]
    pub add_label: Vec<String>,
    /// Remove a label (repeatable)
    #[arg(long = "remove-label", value_name = "LABEL")]
    pub remove_label: Vec<String>,
    /// Remove all labels
    #[arg(long, conflicts_with_all = ["add_label", "remove_label"])]
    pub clear_labels: bool,
}

#[derive(Subcommand)]
pub enum TeamCommands {
    /// List all teams
    List,
}
