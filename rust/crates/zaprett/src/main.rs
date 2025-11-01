use clap::Parser;
use log::info;
use zaprett::cli::CliApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let cli = CliApp::parse();
    match &cli.cmd() {
        Some(cmd) => cmd.exec().await?,
        None => info!("zaprett installed. Join us: t.me/zaprett_module")
    }

    Ok(())
}
