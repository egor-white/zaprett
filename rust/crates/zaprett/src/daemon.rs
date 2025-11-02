use crate::{MODULE_PATH, run_nfqws};
use daemonize::Daemonize;
use log::{error, info};
use std::fs::File;

pub async fn daemonize_nfqws(args: &str) {
    info!("Starting nfqws as a daemon");

    let stdout = File::create(MODULE_PATH.join("tmp/nfqws.out")).unwrap();
    let stderr = File::create(MODULE_PATH.join("tmp/nfqws.err")).unwrap();

    let daemonize = Daemonize::new()
        .pid_file(MODULE_PATH.join("tmp/pid.lock").as_path())
        .working_directory(MODULE_PATH.join("tmp"))
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => {
            info!("Success, daemonized");
            run_nfqws(args).unwrap()
        }
        Err(e) => error!("Error while starting nfqws daemon: {e}"),
    }
}
