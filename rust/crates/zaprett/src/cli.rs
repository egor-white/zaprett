pub mod commands;

use clap::Parser;
use commands::Command;
use getset::Getters;

#[derive(Parser, Getters)]
#[command(version = env!("MODULE_VERSION"))]
#[getset(get = "pub")]
pub struct CliApp {
    #[command(subcommand)]
    cmd: Option<Command>,
}
