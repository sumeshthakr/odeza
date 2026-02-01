//! Odeza CLI entry point

use anyhow::Result;
use clap::Parser;

use odeza_cli::{Cli, execute};

fn main() -> Result<()> {
    let cli = Cli::parse();
    execute(cli)
}
