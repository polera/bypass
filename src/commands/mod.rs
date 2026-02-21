mod create;

use anyhow::Result;
use crate::cli::{Cli, Commands};

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Create(args) => create::run(args, cli.token).await,
    }
}
