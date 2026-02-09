use clap::Parser;
use zaprett::cli::CliApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let cli = CliApp::parse();
    match cli.cmd() {
        Some(cmd) => cmd.exec().await?,
        None => println!("zaprett installed. Join us in Telegram: t.me/zaprett_module"),
    }

    Ok(())
}
