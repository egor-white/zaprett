use crate::{bin_version, get_autostart, module_version, set_autostart};
use clap::Subcommand;
use log::error;
use crate::service::{restart_service, service_status, start_service, stop_service};

#[derive(Subcommand)]
pub enum Command {
    /// Start the service
    Start,

    /// Stop the service
    Stop,

    /// Restart the service
    Restart,

    /// Show the current service status
    Status,

    /// Enable or disable automatic restart
    SetAutostart {
        /// Whether to enable (true) or disable (false) autostart
        #[arg(value_parser = clap::value_parser!(bool))]
        autostart: bool,
    },

    /// Show whether autostart is enabled
    GetAutostart,

    /// Show the module version
    ModuleVersion,

    /// Show the nfqws binary version
    BinaryVersion,
}

impl Command {
    pub async fn exec(&self) -> anyhow::Result<()> {
        match self {
            Command::Start => return start_service().await,
            Command::Stop => {
                let _ = stop_service().await;
            },
            Command::Restart => return restart_service().await,
            Command::Status => {
                println!(
                    "zaprett is {}",
                    if service_status().await {
                        "working"
                    } else {
                        "stopped"
                    }
                );
            },
            Command::SetAutostart { autostart } => {
                if let Err(err) = set_autostart(autostart).await {
                    error!("Failed to set auto start: {err}")
                }
            },
            Command::GetAutostart => get_autostart(),
            Command::ModuleVersion => module_version(),
            Command::BinaryVersion => bin_version(),
        }

        Ok(())
    }
}