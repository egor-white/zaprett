use crate::config::Config;
use crate::daemon::daemonize_nfqws;
use crate::iptables_rust::{clear_iptables_rules, setup_iptables_rules};
use crate::{DEFAULT_START, MODULE_PATH, ZAPRETT_DIR_PATH};
use anyhow::bail;
use log::info;
use nix::sys::signal::{Signal, kill};
use nix::unistd::{Pid, Uid};
use regex::Regex;
use std::borrow::Cow;
use sysctl::{Ctl, CtlValue, Sysctl};
use sysinfo::{Pid as SysPid, System};
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub async fn start_service() -> anyhow::Result<()> {
    if !Uid::effective().is_root() {
        bail!("Running not from root, exiting");
    };

    if service_status().await? {
        bail!("zaprett already started")
    }

    println!("Starting zaprett service...");

    let tmp_dir = MODULE_PATH.join("tmp");
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir).await?;
        fs::create_dir_all(&tmp_dir).await?;
    }

    let mut config_contents = String::new();
    File::open(ZAPRETT_DIR_PATH.join("config.json"))
        .await
        .expect("cannot open config.json")
        .read_to_string(&mut config_contents)
        .await?;

    let config: Config = serde_json::from_str(&config_contents).expect("invalid json");

    let start = fs::read_to_string(config.strategy())
        .await
        .map(Cow::Owned)
        .unwrap_or(Cow::Borrowed(DEFAULT_START));

    let regex_hostlist = Regex::new(r"\$hostlist")?;
    let regex_ipsets = Regex::new(r"\$ipset")?;
    let regex_zaprettdir = Regex::new(r"\$\{?zaprettdir}?")?;

    let mut strat_modified;
    let (hosts, ipsets) = config.list_type().merge(&config).await;

    strat_modified = regex_hostlist.replace_all(&start, &hosts).into_owned();
    strat_modified = regex_ipsets
        .replace_all(&strat_modified, &ipsets)
        .into_owned();

    strat_modified = regex_zaprettdir
        .replace_all(&strat_modified, ZAPRETT_DIR_PATH.to_str().unwrap())
        .into_owned();

    let ctl = Ctl::new("net.netfilter.nf_conntrack_tcp_be_liberal")?;
    ctl.set_value(CtlValue::String("1".into()))?;

    setup_iptables_rules().expect("setup iptables rules");

    daemonize_nfqws(&strat_modified).await;
    println!("zaprett service started!");
    Ok(())
}

pub async fn stop_service() -> anyhow::Result<()> {
    if !Uid::effective().is_root() {
        bail!("Running not from root, exiting");
    };

    if !service_status().await? {
        bail!("zaprett service already stopped")
    }

    clear_iptables_rules().expect("clear iptables rules");

    let pid_str = fs::read_to_string(MODULE_PATH.join("tmp/pid.lock")).await?;
    let pid = pid_str.trim().parse::<i32>()?;

    kill(Pid::from_raw(pid), Signal::SIGKILL)?;

    Ok(())
}

pub async fn restart_service() -> anyhow::Result<()> {
    if !Uid::effective().is_root() {
        bail!("Running not from root, exiting");
    };
    stop_service().await?;
    start_service().await?;
    info!("zaprett service restarted!");
    Ok(())
}

pub async fn service_status() -> anyhow::Result<bool> {
    if !Uid::effective().is_root() {
        bail!("Running not from root, exiting");
    };

    let Ok(Some(pid)) = fs::read_to_string(MODULE_PATH.join("/tmp/pid.lock"))
        .await
        .map(|s| s.trim().parse::<usize>().ok())
    else {
        bail!("failed to get pid");
    };

    let is_zaprett = System::new_all()
        .process(SysPid::from(pid))
        .map(|process| process.name() == "zaprett")
        .unwrap_or(false);

    Ok(is_zaprett)
}
