use crate::config::{Config, ServiceType};
use crate::daemon::daemonize_nfqws;
use crate::daemon::daemonize_nfqws2;
use crate::iptables_rust::{clear_iptables_rules, setup_iptables_rules};
use crate::{DEFAULT_STRATEGY_NFQWS, DEFAULT_STRATEGY_NFQWS2, MODULE_PATH, ZAPRETT_DIR_PATH, ZAPRETT_LIBS_PATH};
use anyhow::bail;
use log::info;
use nix::sys::signal::{Signal, kill};
use nix::unistd::{Pid, Uid};
use regex::Regex;
use std::borrow::Cow;
use std::io::ErrorKind;
use std::path::Path;
use sysctl::{Ctl, CtlValue, Sysctl};
use sysinfo::{Pid as SysPid, System};
use tokio::fs;
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

    let config_path = ZAPRETT_DIR_PATH.join("config.json");
    let mut config_contents = String::new();

    match fs::File::open(&config_path).await {
        Ok(mut file) => {
            file.read_to_string(&mut config_contents).await?;
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let default_config = Config::default();
            let json = serde_json::to_string_pretty(&default_config)?;
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            fs::write(&config_path, &json).await?;
            config_contents = json;
        }
        Err(e) => return Err(e.into()),
    }

    let config: Config = serde_json::from_str(&config_contents)?;

    let start: Cow<str> = if config.service_type() == &ServiceType::Nfqws {
        fs::read_to_string(config.strategy())
            .await
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed(DEFAULT_STRATEGY_NFQWS))
    }
    else if config.service_type() == &ServiceType::Nfqws2 {
        fs::read_to_string(config.strategy_nfqws2())
            .await
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed(DEFAULT_STRATEGY_NFQWS2))
    }
    else {
        bail!("Broken config file!");
    };

    let regex_hostlist = Regex::new(r"\$(?:hostlist|\{hostlist})")?;
    let regex_ipsets = Regex::new(r"\$(?:ipset|\{ipset})")?;
    let regex_zaprettdir = Regex::new(r"\$(?:zaprettdir|\{zaprettdir})")?;
    let regex_libsdir = Regex::new(r"\$(?:libsdir|\{libsdir})")?;

    let mut strat_modified;
    let (hosts, ipsets) = config.list_type().merge(&config).await;

    strat_modified = regex_hostlist.replace_all(&start, &hosts).into_owned();
    strat_modified = regex_ipsets
        .replace_all(&strat_modified, &ipsets)
        .into_owned();

    strat_modified = regex_zaprettdir
        .replace_all(&strat_modified, ZAPRETT_DIR_PATH.to_str().unwrap())
        .into_owned();

    strat_modified = regex_libsdir
        .replace_all(&strat_modified, ZAPRETT_LIBS_PATH.to_str().unwrap())
        .into_owned();

    let ctl = Ctl::new("net.netfilter.nf_conntrack_tcp_be_liberal")?;
    ctl.set_value(CtlValue::String("1".into()))?;

    setup_iptables_rules().expect("setup iptables rules");

    if config.service_type() == &ServiceType::Nfqws {
        daemonize_nfqws(&strat_modified).await;
    }
    else if config.service_type() == &ServiceType::Nfqws2 {
        daemonize_nfqws2(&strat_modified).await;
    }
    else {
        bail!("Broken config file!");
    }

    println!("zaprett service started!");
    Ok(())
}

pub async fn stop_service() -> anyhow::Result<()> {
    if !Uid::effective().is_root() {
        bail!("Running not from root, exiting");
    };

    if !service_status().await? {
        info!("zaprett service already stopped");
        return Ok(())
    }

    clear_iptables_rules().expect("clear iptables rules");

    let pid_str = fs::read_to_string(MODULE_PATH.join("tmp/pid.lock")).await?;
    let pid = pid_str.trim().parse::<i32>()?;

    kill(Pid::from_raw(pid), Signal::SIGKILL)?;

    println!("zaprett service stopped");
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

    let pid_i32 = match fs::read_to_string(Path::new(*MODULE_PATH).join("tmp/pid.lock")).await {
        Ok(s) => match s.trim().parse::<i32>() {
            Ok(pid) => pid,
            Err(_) => return Ok(false),
        },
        Err(_) => return Ok(false),
    };
    let pid = SysPid::from(pid_i32 as usize);
    let system = System::new_all();
    if let Some(process) = system.process(pid) {
        if process.name() == "zaprett" {
            return Ok(true);
        }
    }
    Ok(false)
}
