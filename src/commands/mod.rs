mod create;

use crate::cli::{Cli, Commands};
use anyhow::Result;

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Create(args) => create::run(args, cli.token).await,
    }
}
