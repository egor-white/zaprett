use crate::autostart::{get_autostart, set_autostart};
use crate::service::{restart_service, service_status, start_service, stop_service};
use crate::{bin_version, module_version, run_nfqws};
use clap::Subcommand;

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
    SetAutostart,

    /// Show whether autostart is enabled
    GetAutostart,

    /// Show the module version
    ModuleVersion,

    /// Show the nfqws binary version
    BinaryVersion,

    /// Run nfqws
    Args {
        #[arg(allow_hyphen_values=true, trailing_var_arg = true, num_args = 0..)]
        args: Vec<String>,
    },
}

impl Command {
    pub async fn exec(&self) -> anyhow::Result<()> {
        match self {
            Command::Start => start_service().await?,
            Command::Stop => stop_service().await?,
            Command::Restart => restart_service().await?,
            Command::Status => {
                println!(
                    "zaprett is {}",
                    if service_status().await? {
                        "working"
                    } else {
                        "stopped"
                    }
                );
            }
            Command::SetAutostart => set_autostart().await?,
            Command::GetAutostart => println!("{}", get_autostart()),
            Command::ModuleVersion => println!("{}", module_version().await?),
            Command::BinaryVersion => println!("{}", bin_version()),
            Command::Args { args } => run_nfqws(&args.join(" "))?,
        }

        Ok(())
    }
}
