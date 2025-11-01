use crate::{MODULE_PATH, run_nfqws};
use daemonize::Daemonize;
use log::{error, info};

pub async fn daemonize_nfqws(args: &str) {
    info!("Starting nfqws as a daemon");
    let daemonize = Daemonize::new()
        .pid_file(MODULE_PATH.join("tmp/pid.lock").as_path())
        .working_directory("/tmp")
        // .group("daemon")
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => {
            info!("Success, daemonized");
            run_nfqws(args).await.unwrap()
        }
        Err(e) => error!("Error while starting nfqws daemon: {e}"),
    }
}
