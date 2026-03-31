mod cli;
mod commands;
mod config;
#[cfg(feature = "tui")]
mod tui;
mod ui;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Download(args) => commands::download::execute(args).await?,
        cli::Commands::Queue(args) => commands::queue::execute(args).await?,
        cli::Commands::Config(args) => commands::config::execute(args).await?,
        #[cfg(feature = "tui")]
        cli::Commands::Tui => {
            tui::run().await?;
        }
    }

    Ok(())
}
