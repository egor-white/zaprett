use clap::Parser;
use getset::Getters;
use serde::{Deserialize, Serialize};
use crate::commands::Command;

#[derive(Parser, Getters)]
#[command(version)]
#[getset(get = "pub")]
pub struct CliApp {
    #[command(subcommand)]
    cmd: Option<Command>,
}
