pub mod commands;
pub mod cli;
pub mod config;

use std::error;
use std::ffi::CString;
use std::os::raw::c_char;
use std::path::Path;
use std::sync::LazyLock;
use anyhow::bail;
use daemonize::Daemonize;
use ini::Ini;
use log::{error, info};
use nix::sys::signal::{kill, Signal};
use nix::unistd::{Pid, Uid};
use regex::Regex;
use sysctl::Sysctl;
use tokio::{fs, task};
use tokio::fs::File;
use tokio::io::{copy, AsyncReadExt, AsyncWriteExt};
use libnfqws::nfqws_main;
use crate::config::Config;

pub static MODULE_PATH: LazyLock<&Path> = LazyLock::new(|| Path::new("/data/adb/modules/zaprett"));
pub static ZAPRETT_DIR_PATH: LazyLock<&Path> =
    LazyLock::new(|| Path::new("/storage/emulated/0/zaprett"));

async fn daemonize_nfqws(args: &str) {
    info!("Starting nfqws as a daemon");
    let daemonize = Daemonize::new()
        .pid_file(MODULE_PATH.join("tmp/pid.lock").as_path())
        .working_directory("/tmp")
        .group("daemon")
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => {
            info!("Success, daemonized");
            run_nfqws(args).await.unwrap()
        }
        Err(e) => error!("Error while starting nfqws daemon: {e}"),
    }
}

async fn start_service() -> anyhow::Result<()> {
    if !Uid::effective().is_root() {
        bail!("Running not from root, exiting");
    };

    info!("Starting zaprett service...");

    let tmp_dir = MODULE_PATH.join("/tmp");
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir).await?;
        fs::create_dir_all(&tmp_dir).await?;
    }

    let mut config_contents = String::new();
    File::open(ZAPRETT_DIR_PATH.join("config.json"))
        .await
        .expect("cannot open config.json")
        .read_to_string(&mut config_contents).await?;

    let config: Config = serde_json::from_str(&config_contents).expect("invalid json");

    let def_strat = String::from("
        --filter-tcp=80 --dpi-desync=fake,split2 --dpi-desync-autottl=2 --dpi-desync-fooling=md5sig,badsum $hostlist --new
        --filter-tcp=443 $hostlist --dpi-desync=fake,split2 --dpi-desync-repeats=6 --dpi-desync-fooling=md5sig,badsum --dpi-desync-fake-tls=${zaprettdir}/bin/tls_clienthello_www_google_com.bin --new
        --filter-tcp=80,443 --dpi-desync=fake,disorder2 --dpi-desync-repeats=6 --dpi-desync-autottl=2 --dpi-desync-fooling=md5sig,badsum $hostlist --new
        --filter-udp=50000-50100 --dpi-desync=fake --dpi-desync-any-protocol --dpi-desync-fake-quic=0xC30000000108 --new
        --filter-udp=443 $hostlist --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fake-quic=${zaprettdir}/bin/quic_initial_www_google_com.bin --new
        --filter-udp=443 --dpi-desync=fake --dpi-desync-repeats=6 $hostlist
        ");

    let start = fs::read_to_string(config.strategy())
        .await
        .unwrap_or(def_strat);

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

    let ctl = sysctl::Ctl::new("net.netfilter.nf_conntrack_tcp_be_liberal")?;
    ctl.set_value(sysctl::CtlValue::String("1".into()))?;

    setup_iptables_rules().expect("setup iptables rules");

    daemonize_nfqws(&strat_modified).await;
    info!("zaprett service started!");
    Ok(())
}

async fn stop_service() -> anyhow::Result<()> {
    if !Uid::effective().is_root() {
        bail!("Running not from root, exiting");
    };

    clear_iptables_rules().expect("clear iptables rules");

    let pid_str = fs::read_to_string(MODULE_PATH.join("tmp/pid.lock")).await?;
    let pid = pid_str.trim().parse::<i32>()?;

    kill(Pid::from_raw(pid), Signal::SIGKILL)?;

    Ok(())
}

async fn restart_service() -> anyhow::Result<()> {
    stop_service().await?;
    start_service().await?;
    info!("zaprett service restarted!");
    Ok(())
}

async fn set_autostart(autostart: &bool) -> Result<(), anyhow::Error> {
    if *autostart {
        File::create(MODULE_PATH.join("autostart")).await?;
    } else {
        fs::remove_file(MODULE_PATH.join("autostart")).await?;
    }

    Ok(())
}

fn get_autostart() {
    let file = MODULE_PATH.join("autostart");
    println!("{}", file.exists());
}

async fn service_status() -> bool {
    fs::read_to_string(MODULE_PATH.join("tmp/pid.lock"))
        .await
        .ok()
        .and_then(|pid_str| pid_str.trim().parse::<i32>().ok())
        .is_some()
}

fn module_version() {
    if let Ok(prop) = Ini::load_from_file(MODULE_PATH.join("module.prop"))
        && let Some(props) = prop.section::<String>(None)
        && let Some(version) = props.get("version")
    {
        println!("{version}");
    }
}

fn bin_version() {
    println!("{}", env!("ZAPRET_VERSION"));
}

pub async fn merge_files(
    input_paths: &[impl AsRef<Path>],
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn error::Error>> {
    let output_path = output_path.as_ref();
    let mut output_file = File::create(output_path).await?;

    for input in input_paths {
        let input_path = input.as_ref();
        let mut input_file = File::open(input_path)
            .await
            .map_err(|e| format!("Failed to open {}: {}", input_path.display(), e))?;

        copy(&mut input_file, &mut output_file)
            .await
            .map_err(|e| {
                format!(
                    "Failed to write contents of {}: {}",
                    input_path.display(),
                    e
                )
            })?;
    }

    output_file.flush().await?;
    Ok(())
}

fn setup_iptables_rules() -> Result<(), Box<dyn error::Error>> {
    let ipt = iptables::new(false)?;

    ipt.insert(
        "mangle",
        "POSTROUTING",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
        1,
    )?;

    ipt.insert(
        "mangle",
        "PREROUTING",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
        1,
    )?;

    ipt.append(
        "filter",
        "FORWARD",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
    )?;

    Ok(())
}

fn clear_iptables_rules() -> Result<(), Box<dyn error::Error>> {
    let ipt = iptables::new(false)?;

    ipt.delete(
        "mangle",
        "POSTROUTING",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
    )?;

    ipt.delete(
        "mangle",
        "PREROUTING",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
    )?;

    ipt.delete(
        "filter",
        "FORWARD",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
    )?;

    Ok(())
}

async fn run_nfqws(args_str: &str) -> anyhow::Result<()> {
    if service_status().await {
        bail!("nfqws already started!");
    }

    let mut args = vec!["nfqws".to_string()];

    if args_str.trim().is_empty() {
        args.push("-v".to_string());
    } else {
        args.extend(args_str.split_whitespace().map(String::from));
    }

    task::spawn_blocking(move || {
        let c_args: Vec<CString> = args
            .into_iter()
            .map(|arg| CString::new(arg).unwrap())
            .collect();

        let mut ptrs: Vec<*const c_char> = c_args.iter().map(|arg| arg.as_ptr()).collect();

        unsafe {
            nfqws_main(c_args.len() as libc::c_int, ptrs.as_mut_ptr() as *mut _);
        }
    })
        .await?;

    Ok(())
}
