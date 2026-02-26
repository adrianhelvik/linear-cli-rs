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
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth(args) => commands::auth::run(args).await,
        Commands::Me => commands::me::run().await,
        Commands::Issue { command } => match command {
            IssueCommands::List(args) => commands::issue::list::run(args).await,
            IssueCommands::View { id } => commands::issue::view::run(id).await,
            IssueCommands::Create(args) => commands::issue::create::run(args).await,
            IssueCommands::Update(args) => commands::issue::update::run(args).await,
            IssueCommands::Search { query } => commands::issue::search::run(query).await,
            IssueCommands::Assign { id, user } => commands::issue::assign::run(id, user).await,
            IssueCommands::State { id, state } => commands::issue::state::run(id, state).await,
        },
        Commands::Team { command } => match command {
            TeamCommands::List => commands::team::list::run().await,
        },
    }
}
