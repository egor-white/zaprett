mod libnfqws;

use clap::{ArgAction, Parser, Subcommand, builder::BoolishValueParser};
use ini::Ini;
use procfs::process::all_processes;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::fs::File;
use std::io::BufReader;
use std::io::{Read, Write};
use std::os::raw::c_char;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{fs, path::Path};
use sysctl::{CtlValue, Sysctl};
use crate::libnfqws::nfqws_main;

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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match &cli.cmd {
        Some(Commands::Start) => start_service(),
        Some(Commands::Stop) => stop_service(),
        Some(Commands::Restart) => restart_service(),
        Some(Commands::Status) => service_status(),
        Some(Commands::SetAutostart { autostart }) => set_autostart(autostart),
        Some(Commands::GetAutostart) => get_autostart(),
        Some(Commands::ModuleVer) => module_version(),
        Some(Commands::BinVer) => todo!(), //bin_version(),
        //None => println!("zaprett installed. Join us: t.me/zaprett_module"),
        None => run_nfqws("-v").await,
    }
    tokio::signal::ctrl_c().await.unwrap();
}

fn start_service() {
    println!("Starting zaprett service...");

    let tmp_dir = Path::new("/data/adb/modules/zaprett/tmp");
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir).unwrap()
    }

    let reader =
        BufReader::new(File::open("/sdcard/zaprett/config.json").expect("cannot open config.json"));
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
        fs::read_to_string(&config.strategy).unwrap_or_else(|_| def_strat)
    } else {
        def_strat
    };

    let regex_hostlist = Regex::new(r"\$hostlist").unwrap();
    let regex_ipsets = Regex::new(r"\$ipset").unwrap();
    let regex_zaprettdir = Regex::new(r"\$\{?zaprettdir\}?").unwrap();

    let zaprett_dir = String::from("/sdcard/zaprett");
    let mut strat_modified = String::new();

    if list_type.eq("whitelist") {
        merge_files(
            config.active_lists,
            "/data/adb/modules/zaprett/tmp/hostlist",
        )
        .unwrap();
        merge_files(config.active_ipsets, "/data/adb/modules/zaprett/tmp/ipset").unwrap();

        let hosts = String::from("--hostlist=/data/adb/modules/zaprett/tmp/hostlist");
        let ipsets = String::from("--ipset=/data/adb/modules/zaprett/tmp/ipset");

        strat_modified = regex_hostlist.replace_all(&strat, &hosts).into_owned();
        strat_modified = regex_ipsets
            .replace_all(&strat_modified, &ipsets)
            .into_owned();
        strat_modified = regex_zaprettdir
            .replace_all(&strat_modified, &zaprett_dir)
            .into_owned();
    } else if list_type.eq("blacklist") {
        merge_files(
            config.active_exclude_lists,
            "/data/adb/modules/zaprett/tmp/hostlist-exclude",
        )
        .unwrap();
        merge_files(
            config.active_exclude_ipsets,
            "/data/adb/modules/zaprett/tmp/ipset-exclude",
        )
        .unwrap();

        let hosts =
            String::from("--hostlist-exclude=/data/adb/modules/zaprett/tmp/hostlist-exclude");
        let ipsets = String::from("--ipset-exclude=/data/adb/modules/zaprett/tmp/ipset-exclude");

        strat_modified = regex_hostlist.replace_all(&strat, &hosts).into_owned();
        strat_modified = regex_ipsets
            .replace_all(&strat_modified, &ipsets)
            .into_owned();
        strat_modified = regex_zaprettdir
            .replace_all(&strat_modified, &zaprett_dir)
            .into_owned();
    } else {
        panic!("no list-type called {}", &list_type)
    }

    let ctl = sysctl::Ctl::new("net.netfilter.nf_conntrack_tcp_be_liberal").unwrap();
    ctl.set_value(CtlValue::Int(1)).unwrap();

    setup_iptables_rules();
    //run_nfqws(&strat_modified);
    todo!();
    println!("zaprett service started!");
}
fn stop_service() {
    clear_iptables_rules();
    todo!()
}
fn restart_service() {
    stop_service();
    start_service();
    println!("zaprett service restarted!")
}
fn set_autostart(autostart: &bool) {
    if *autostart {
        if let Err(e) = std::fs::File::create("/data/adb/modules/zaprett/autostart") {
            eprintln!("autostart: cannot create flag file: {e}");
        }
    } else {
        fs::remove_file("/data/adb/modules/zaprett/autostart").unwrap()
    }
}
fn get_autostart() {
    let file = Path::new("/data/adb/modules/zaprett/autostart");
    println!("{}", file.exists());
}
fn service_status() {
    let running = match all_processes() {
        Ok(iter) => iter
            .filter_map(|rp| rp.ok())
            .filter_map(|p| p.stat().ok())
            .any(|st| st.comm == "nfqws"),
        Err(_) => false,
    };

    println!("zaprett is {}", if running { "working" } else { "stopped" });
}

fn module_version() {
    if let Ok(prop) = Ini::load_from_file("/data/adb/modules/zaprett/module.prop") {
        if let Some(props) = prop.section::<String>(None) {
            if let Some(v) = props.get("version") {
                println!("{}", v);
            }
        }
    }
}
fn bin_version() {
    todo!()
    /*if let Ok(output) = Command::new("nfqws").arg("--version").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(re) = Regex::new(r"version v[0-9.]+") {
                if let Some(m) = re.find(&stdout) {
                    if let Some(v) = m.as_str().split_whitespace().nth(1) {
                        println!("{}", v);
                        return;
                    }
                }
            }
        }
    }*/
}
fn merge_files(
    input_paths: Vec<String>,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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
fn setup_iptables_rules() {
    let ipt = iptables::new(false).unwrap();

    ipt.insert(
        "mangle",
        "POSTROUTING",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
        1,
    )
    .unwrap();
    ipt.insert(
        "mangle",
        "PREROUTING",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
        1,
    )
    .unwrap();
    ipt.append(
        "filter",
        "FORWARD",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
    )
    .unwrap();
}
fn clear_iptables_rules() {
    let ipt = iptables::new(false).unwrap();

    ipt.delete(
        "mangle",
        "POSTROUTING",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
    )
    .unwrap();
    ipt.delete(
        "mangle",
        "PREROUTING",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
    )
    .unwrap();
    ipt.delete(
        "filter",
        "FORWARD",
        "-j NFQUEUE --queue-num 200 --queue-bypass",
    )
    .unwrap();
}
async fn run_nfqws(args_str: &str) {
    static RUNNING: AtomicBool = AtomicBool::new(false);

    if RUNNING.load(Ordering::SeqCst) {
        panic!("Thread with nfqws already started!");
    }

    let mut args: Vec<&str> = vec!["nfqws"];

    if args_str.trim().is_empty() {
        args.push("-v");
    } else {
        for token in args_str.trim().split_whitespace() {
            args.push(token);
        }
    }
    let c_args: Vec<CString> = args.iter().map(|&arg| CString::new(arg).unwrap()).collect();
    let argv: Vec<*const c_char> = c_args.iter().map(|arg| arg.as_ptr()).collect();

    RUNNING.store(true, Ordering::SeqCst);
    // tokio::task::spawn_blocking(move || );

    unsafe {
        nfqws_main(argv.len() as libc::c_int, argv.as_ptr());
    }
}
