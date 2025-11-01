pub mod cli;
pub mod config;
pub mod iptables_rust;
mod service;
mod daemon;

use std::error;
use std::ffi::CString;
use std::os::raw::c_char;
use std::path::Path;
use std::sync::LazyLock;
use anyhow::bail;
use ini::Ini;
use tokio::{fs, task};
use tokio::fs::File;
use tokio::io::{copy, AsyncWriteExt};
use libnfqws::nfqws_main;

pub static MODULE_PATH: LazyLock<&Path> = LazyLock::new(|| Path::new("/data/adb/modules/zaprett"));
pub static ZAPRETT_DIR_PATH: LazyLock<&Path> =
    LazyLock::new(|| Path::new("/storage/emulated/0/zaprett"));

pub static DEFAULT_START: &str = "
        --filter-tcp=80 --dpi-desync=fake,split2 --dpi-desync-autottl=2 --dpi-desync-fooling=md5sig,badsum $hostlist --new
        --filter-tcp=443 $hostlist --dpi-desync=fake,split2 --dpi-desync-repeats=6 --dpi-desync-fooling=md5sig,badsum --dpi-desync-fake-tls=${zaprettdir}/bin/tls_clienthello_www_google_com.bin --new
        --filter-tcp=80,443 --dpi-desync=fake,disorder2 --dpi-desync-repeats=6 --dpi-desync-autottl=2 --dpi-desync-fooling=md5sig,badsum $hostlist --new
        --filter-udp=50000-50100 --dpi-desync=fake --dpi-desync-any-protocol --dpi-desync-fake-quic=0xC30000000108 --new
        --filter-udp=443 $hostlist --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fake-quic=${zaprettdir}/bin/quic_initial_www_google_com.bin --new
        --filter-udp=443 --dpi-desync=fake --dpi-desync-repeats=6 $hostlist
        ";

async fn set_autostart(autostart: &bool) -> Result<(), anyhow::Error> {
    let autostart_path = MODULE_PATH.join("autostart");

    if *autostart {
        File::create(autostart_path).await?;
    } else {
        fs::remove_file(autostart_path).await?;
    }

    Ok(())
}

fn get_autostart() {
    let file = MODULE_PATH.join("autostart");
    println!("{}", file.exists());
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

async fn run_nfqws(args_str: &str) -> anyhow::Result<()> {
    if service::service_status().await {
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
