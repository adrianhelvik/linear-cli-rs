mod api;
mod cli;
mod commands;
mod config;
mod interactive;
mod output;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, IssueCommands, TeamCommands};

#[tokio::main]
async fn main() -> Result<()> {
    // Support `linear issue DIS-510` as shorthand for `linear issue view DIS-510`
    let args: Vec<String> = std::env::args().collect();
    let cli = if args.len() >= 3
        && args[1] == "issue"
        && cli::looks_like_issue_id(&args[2])
    {
        let mut patched = vec![args[0].clone(), "issue".into(), "view".into()];
        patched.extend_from_slice(&args[2..]);
        Cli::parse_from(patched)
    } else {
        Cli::parse()
    };

    match cli.command {
        Commands::Auth(args) => commands::auth::run(args).await,
        Commands::Me => commands::me::run().await,
        Commands::Api(args) => commands::api::run(args).await,
        Commands::Issue { command } => match command {
            IssueCommands::List(args) => commands::issue::list::run(args).await,
            IssueCommands::View { id, json } => commands::issue::view::run(id, json).await,
            IssueCommands::Create(args) => commands::issue::create::run(args).await,
            IssueCommands::Update(args) => commands::issue::update::run(args).await,
            IssueCommands::Search { query, json } => {
                commands::issue::search::run(query, json).await
            }
            IssueCommands::Assign { id, user } => commands::issue::assign::run(id, user).await,
            IssueCommands::State { id, state } => commands::issue::state::run(id, state).await,
            IssueCommands::Comment { id, body } => {
                commands::issue::comment::run(id, body).await
            }
        },
        Commands::Team { command } => match command {
            TeamCommands::List => commands::team::list::run().await,
        },
    }
}
