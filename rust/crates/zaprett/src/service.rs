use crate::config::{Config, Manifest, ServiceType};
use crate::daemon::daemonize_nfqws;
use crate::daemon::daemonize_nfqws2;
use crate::iptables_rust::{clear_iptables_rules, setup_iptables_rules};
use crate::{get_manifest, get_all_manifests, DEFAULT_STRATEGY_NFQWS, DEFAULT_STRATEGY_NFQWS2};
use anyhow::bail;
use log::info;
use nix::sys::signal::{Signal, kill};
use nix::unistd::{Pid, Uid};
use regex::Regex;
use std::borrow::Cow;
use std::collections::{HashMap};
use std::io::ErrorKind;
use std::path::Path;
use sysctl::{Ctl, CtlValue, Sysctl};
use sysinfo::{Pid as SysPid, System};
use tokio::fs;
use tokio::io::AsyncReadExt;
use crate::path::path::{MODULE_PATH, ZAPRETT_DIR_PATH, ZAPRETT_LIBS_PATH};
use crate::strategy::prepare_manifests;

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
    let strategy = get_manifest(Path::new(config.strategy())).ok();
    let default_strategy = match config.service_type() {
        ServiceType::Nfqws => DEFAULT_STRATEGY_NFQWS,
        ServiceType::Nfqws2 => DEFAULT_STRATEGY_NFQWS2
    };
    let start: Cow<str> = if let Some(manifest) = strategy {
        fs::read_to_string(manifest.file())
            .await
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed(default_strategy))
    } else {
        Cow::Borrowed(default_strategy)
    };
    let regex_hostlists = Regex::new(r"\$(?:hostlists|\{hostlists})")?;
    let regex_hostlist = Regex::new(r"\$\{hostlist:([^}]+)\}")?;
    let regex_hostlist_exclude = Regex::new(r"\$\{hostlist_exclude:([^}]+)\}")?;
    let regex_ipset = Regex::new(r"\$\{ipset:([^}]+)\}")?;
    let regex_ipset_exclude = Regex::new(r"\$\{ipset_exclude:([^}]+)\}")?;
    let regex_ipsets = Regex::new(r"\$(?:ipsets|\{ipsets})")?;
    let regex_zaprettdir = Regex::new(r"\$(?:zaprettdir|\{zaprettdir})")?;
    let regex_libsdir = Regex::new(r"\$(?:libsdir|\{libsdir})")?;
    let (hosts, ipsets) = config.list_type().merge(&config).await?;
    let hostlists: HashMap<String, Manifest> =
        get_all_manifests(&ZAPRETT_DIR_PATH.join("manifests/lists/include"))
            .unwrap_or_default()
            .into_iter()
            .map(|m| (m.name().clone(), m))
            .collect();
    let hostlists_exclude: HashMap<String, Manifest> =
        get_all_manifests(&ZAPRETT_DIR_PATH.join("manifests/lists/exclude"))
            .unwrap_or_default()
            .into_iter()
            .map(|m| (m.name().clone(), m))
            .collect();
    let ipset: HashMap<String, Manifest> =
        get_all_manifests(&ZAPRETT_DIR_PATH.join("manifests/ipset/include"))
            .unwrap_or_default()
            .into_iter()
            .map(|m| (m.name().clone(), m))
            .collect();
    let ipset_exclude: HashMap<String, Manifest> =
        get_all_manifests(&ZAPRETT_DIR_PATH.join("manifests/ipset/exclude"))
            .unwrap_or_default()
            .into_iter()
            .map(|m| (m.name().clone(), m))
            .collect();
    let strat_modified = prepare_manifests(&start, &regex_hostlist, &hostlists, &tmp_dir)?;
    let strat_modified = prepare_manifests(&strat_modified, &regex_hostlist_exclude, &hostlists_exclude, &tmp_dir)?;
    let strat_modified = prepare_manifests(&strat_modified, &regex_ipset, &ipset, &tmp_dir)?;
    let strat_modified = prepare_manifests(&strat_modified, &regex_ipset_exclude, &ipset_exclude, &tmp_dir)?;
    let strat_modified = regex_hostlists.replace_all(&strat_modified, &hosts);
    let strat_modified = regex_ipsets.replace_all(&strat_modified, &ipsets);
    let strat_modified =
        regex_zaprettdir.replace_all(&strat_modified, ZAPRETT_DIR_PATH.to_str().unwrap());
    let strat_modified =
        regex_libsdir.replace_all(&strat_modified, ZAPRETT_LIBS_PATH.to_str().unwrap());
    let strat_modified = strat_modified.into_owned();

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
