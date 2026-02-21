mod api;
mod cli;
mod commands;
mod config;
mod error;
mod input;
mod resolver;
mod template;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    commands::run(Cli::parse()).await
}
