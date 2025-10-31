use std::error;
use anyhow::bail;
use clap::{ArgAction, Parser, Subcommand, builder::BoolishValueParser};
use daemonize::Daemonize;
use ini::Ini;
use libnfqws::nfqws_main;
use log::{error, info};
use nix::sys::signal::{Signal, kill};
use nix::unistd::{Pid, Uid};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::fs::File;
use std::io::BufReader;
use std::io::{Read, Write};
use std::os::raw::c_char;
use std::sync::LazyLock;
use std::{fs, path::Path};
use sysctl::Sysctl;
use tokio::task;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Start service")]
    Start,

    #[clap(about = "Stop service")]
    Stop,

    #[clap(about = "Restart service")]
    Restart,

    #[clap(about = "Show service status")]
    Status,

    #[clap(about = "Enable/disable autorestart")]
    SetAutostart {
        #[arg(
                    value_name = "boolean",
                    action = ArgAction::Set,
                    value_parser = BoolishValueParser::new()
        )]
        autostart: bool,
    },

    #[clap(about = "Get autorestart state")]
    GetAutostart,

    #[clap(about = "Get module version")]
    ModuleVer,

    #[clap(about = "Get nfqws binary version")]
    BinVer,
}

#[derive(Serialize, Deserialize)]
struct Config {
    active_lists: Vec<String>,
    active_ipsets: Vec<String>,
    active_exclude_lists: Vec<String>,
    active_exclude_ipsets: Vec<String>,
    list_type: String,
    strategy: String,
    app_list: String,
    whitelist: Vec<String>,
    blacklist: Vec<String>,
}

pub static MODULE_PATH: LazyLock<&Path> = LazyLock::new(|| Path::new("/data/adb/modules/zaprett"));
pub static ZAPRETT_DIR_PATH: LazyLock<&Path> =
    LazyLock::new(|| Path::new("/storage/emulated/0/zaprett"));

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let cli = Cli::parse();
    match &cli.cmd {
        Some(Commands::Start) => start_service().await,
        Some(Commands::Stop) => {
            let _ = stop_service().await;
            Ok(())
        }
        Some(Commands::Restart) => {
            restart_service().await;
            Ok(())
        }
        Some(Commands::Status) => {
            println!(
                "zaprett is {}",
                if service_status() {
                    "working"
                } else {
                    "stopped"
                }
            );
            Ok(())
        }
        Some(Commands::SetAutostart { autostart }) => {
            set_autostart(autostart);
            Ok(())
        }
        Some(Commands::GetAutostart) => {
            get_autostart();
            Ok(())
        }
        Some(Commands::ModuleVer) => {
            module_version();
            Ok(())
        }
        Some(Commands::BinVer) => {
            bin_version();
            Ok(())
        }
        None => {
            info!("zaprett installed. Join us: t.me/zaprett_module");
            Ok(())
        }
    }
}

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
        fs::remove_dir_all(&tmp_dir)?;
        fs::create_dir_all(&tmp_dir)?;
    }

    let reader = BufReader::new(
        File::open(ZAPRETT_DIR_PATH.join("config.json")).expect("cannot open config.json"),
    );
    let config: Config = serde_json::from_reader(reader).expect("invalid json");

    let list_type: &String = &config.list_type;

    let def_strat: String = String::from("
        --filter-tcp=80 --dpi-desync=fake,split2 --dpi-desync-autottl=2 --dpi-desync-fooling=md5sig,badsum $hostlist --new
        --filter-tcp=443 $hostlist --dpi-desync=fake,split2 --dpi-desync-repeats=6 --dpi-desync-fooling=md5sig,badsum --dpi-desync-fake-tls=${zaprettdir}/bin/tls_clienthello_www_google_com.bin --new
        --filter-tcp=80,443 --dpi-desync=fake,disorder2 --dpi-desync-repeats=6 --dpi-desync-autottl=2 --dpi-desync-fooling=md5sig,badsum $hostlist --new
        --filter-udp=50000-50100 --dpi-desync=fake --dpi-desync-any-protocol --dpi-desync-fake-quic=0xC30000000108 --new
        --filter-udp=443 $hostlist --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fake-quic=${zaprettdir}/bin/quic_initial_www_google_com.bin --new
        --filter-udp=443 --dpi-desync=fake --dpi-desync-repeats=6 $hostlist
        ");
    let strat = if Path::new(&config.strategy).exists() {
        fs::read_to_string(&config.strategy).unwrap_or(def_strat)
    } else {
        def_strat
    };

    let regex_hostlist = Regex::new(r"\$hostlist")?;
    let regex_ipsets = Regex::new(r"\$ipset")?;
    let regex_zaprettdir = Regex::new(r"\$\{?zaprettdir}?")?;

    let mut strat_modified;

    if list_type.eq("whitelist") {
        merge_files(
            config.active_lists,
            MODULE_PATH.join("tmp/hostlist").as_path(),
        )
        .unwrap();
        merge_files(
            config.active_ipsets,
            MODULE_PATH.join("tmp/ipset").as_path(),
        )
        .unwrap();

        let hosts = format!(
            "--hostlist={}/tmp/hostlist",
            MODULE_PATH.to_str().unwrap()
        );
        let ipsets = format!(
            "--ipset={}tmp/ipset",
            MODULE_PATH.to_str().unwrap()
        );

        strat_modified = regex_hostlist.replace_all(&strat, &hosts).into_owned();
        strat_modified = regex_ipsets
            .replace_all(&strat_modified, &ipsets)
            .into_owned();
        strat_modified = regex_zaprettdir
            .replace_all(&strat_modified, ZAPRETT_DIR_PATH.to_str().unwrap())
            .into_owned();
    } else if list_type.eq("blacklist") {
        merge_files(
            config.active_exclude_lists,
            MODULE_PATH.join("tmp/hostlist-exclude").as_path(),
        )
        .unwrap();
        merge_files(
            config.active_exclude_ipsets,
            MODULE_PATH.join("tmp/ipset-exclude").as_path(),
        )
        .unwrap();

        let hosts = format!(
            "--hostlist-exclude={}/tmp/hostlist-exclude",
            MODULE_PATH.to_str().unwrap()
        );
        let ipsets = format!(
            "--ipset-exclude={}/tmp/ipset-exclude",
            MODULE_PATH.to_str().unwrap()
        );

        strat_modified = regex_hostlist.replace_all(&strat, &hosts).into_owned();
        strat_modified = regex_ipsets
            .replace_all(&strat_modified, &ipsets)
            .into_owned();
        strat_modified = regex_zaprettdir
            .replace_all(&strat_modified, ZAPRETT_DIR_PATH.to_str().unwrap())
            .into_owned();
    } else {
        bail!("no list-type called {}", &list_type)
    }

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

    let pid_str = fs::read_to_string(MODULE_PATH.join("tmp/pid.lock").as_path())?;
    let pid = pid_str.trim().parse::<i32>()?;

    kill(Pid::from_raw(pid), Signal::SIGKILL)?;

    /*for proc in all_processes().unwrap() {
        if let Ok(p) = proc {
            if let Ok(stat) = p.stat() {
                if stat.comm == "zaprett" {
                    let pid = Pid::from_raw(p.pid as i32);
                    if let Err(_) = kill(pid, Signal::SIGTERM) {
                        println!("failed to stop zaprett service")
                    } else {
                        println!("zaprett service stopped!")
                    }
                }
            }
        }
    }*/
    Ok(())
}

async fn restart_service() {
    stop_service().await.unwrap();
    start_service().await.unwrap();
    info!("zaprett service restarted!")
}

fn set_autostart(autostart: &bool) {
    if *autostart {
        if let Err(e) = File::create(MODULE_PATH.join("autostart")) {
            error!("Autostart: cannot create flag file: {e}");
        }
    } else {
        fs::remove_file(MODULE_PATH.join("autostart")).unwrap()
    }
}

fn get_autostart() {
    let file = MODULE_PATH.join("autostart");
    println!("{}", file.exists());
}

fn service_status() -> bool {
    fs::read_to_string(MODULE_PATH.join("tmp/pid.lock"))
        .ok()
        .and_then(|pid_str| pid_str.trim().parse::<i32>().ok())
        .is_some()
    /*match all_processes() {
        Ok(iter) => iter
            .filter_map(|rp| rp.ok())
            .filter_map(|p| p.stat().ok())
            .any(|st| st.pid == pid),
        Err(_) => false,
    }*/
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

fn merge_files(
    input_paths: Vec<String>,
    output_path: &Path,
) -> Result<(), Box<dyn error::Error>> {
    let mut combined_content = String::new();

    for path_str in input_paths {
        let path = Path::new(&path_str);
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        combined_content.push_str(&content);
    }

    let mut output_file = File::create(output_path)?;
    output_file.write_all(combined_content.as_bytes())?;

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
    if service_status() {
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
