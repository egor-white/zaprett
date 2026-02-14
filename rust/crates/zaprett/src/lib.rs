pub mod cli;
pub mod config;
mod daemon;
pub mod iptables_rust;
mod service;
mod autostart;

use libnfqws::nfqws_main;
use libnfqws2::nfqws2_main;
use std::error;
use std::ffi::CString;
use std::os::raw::c_char;
use std::path::Path;
use std::sync::LazyLock;
use tokio::fs::File;
use tokio::io::{copy, AsyncWriteExt};

#[cfg(target_os = "android")]
pub static MODULE_PATH: LazyLock<&Path> =
    LazyLock::new(|| Path::new("/data/adb/modules/zaprett"));
#[cfg(target_os = "android")]
pub static ZAPRETT_DIR_PATH: LazyLock<&Path> =
    LazyLock::new(|| Path::new("/storage/emulated/0/zaprett"));
#[cfg(target_os = "android")]
pub static ZAPRETT_LIBS_PATH: LazyLock<&Path> =
    LazyLock::new(|| Path::new("/storage/emulated/0/zaprett/strategies/nfwqs2/libs"));

// Only for testing
#[cfg(target_os = "linux")]
pub static MODULE_PATH: LazyLock<&Path> =
    LazyLock::new(|| Path::new("zaprett_module"));
#[cfg(target_os = "linux")]
pub static ZAPRETT_DIR_PATH: LazyLock<&Path> =
    LazyLock::new(|| Path::new("zaprett"));
#[cfg(target_os = "linux")]
pub static ZAPRETT_LIBS_PATH: LazyLock<&Path> =
    LazyLock::new(|| Path::new("zaprett/strategies/nfwqs2/libs"));


pub static DEFAULT_STRATEGY_NFQWS: &str = "
        --filter-tcp=80 --dpi-desync=fake,split2 --dpi-desync-autottl=2 --dpi-desync-fooling=md5sig,badsum ${hostlist} --new
        --filter-tcp=443 ${hostlist} --dpi-desync=fake,split2 --dpi-desync-repeats=6 --dpi-desync-fooling=md5sig,badsum --dpi-desync-fake-tls=${zaprettdir}/bin/tls_clienthello_www_google_com.bin --new
        --filter-tcp=80,443 --dpi-desync=fake,disorder2 --dpi-desync-repeats=6 --dpi-desync-autottl=2 --dpi-desync-fooling=md5sig,badsum ${hostlist} --new
        --filter-udp=50000-50100 --dpi-desync=fake --dpi-desync-any-protocol --dpi-desync-fake-quic=0xC30000000108 --new
        --filter-udp=443 ${hostlist} --dpi-desync=fake --dpi-desync-repeats=6 --dpi-desync-fake-quic=${zaprettdir}/bin/quic_initial_www_google_com.bin --new
        --filter-udp=443 --dpi-desync=fake --dpi-desync-repeats=6 ${hostlist}
        ";
// тестовая стратегия, заменить на нормальную потом
pub static DEFAULT_STRATEGY_NFQWS2: &str = "
        --lua-init=@${libsdir}/zapret-lib.lua --lua-init=@${libsdir}/zapret-antidpi.lua
        --blob=quic_google:@${zaprettdir}/bin/quic_initial_www_google_com.bin
        --blob=tls_google:${zaprettdir}/bin/tls_clienthello_www_google_com.bin
        --blob=tls_4pda:@${zaprettdir}/bin/tls_clienthello_4pda_to.bin
        --blob=tls_max:@${zaprettdir}/bin/tls_clienthello_max_ru.bin
        --blob=zero4:0x00000000
        --filter-udp=443 --hostlist=${zaprettdir}/lists/include/list-general.txt --lua-desync=fake:blob=quic_google:repeats=6 --new
        --filter-tcp=443 --hostlist=${zaprettdir}/lists/include/list-google.txt --lua-desync=fake:blob=tls_google:repeats=6:tcp_seq=2:tls_mod=none:ip_id=zero --new
        --filter-tcp=80,443 --hostlist=${zaprettdir}/lists/include/list-general.txt --lua-desync=fake:blob=tls_google:repeats=6:tcp_seq=2:tls_mod=none
        ";

fn nfqws_version() -> &'static str {
    env!("NFQWS_VERSION")
}

fn nfqws2_version() -> &'static str {
    env!("NFQWS2_VERSION")
}

pub async fn merge_files(
    input_paths: &[impl AsRef<Path>],
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn error::Error>> {
    let output_path = output_path.as_ref();
    let mut output_file = File::create(output_path).await?;

    for input in input_paths {
        let input = input.as_ref();

        let mut input_file = File::open(input)
            .await
            .map_err(|e| format!("Failed to open {}: {e}", input.display()))?;

        copy(&mut input_file, &mut output_file).await.map_err(|e| {
            format!(
                "Failed to write contents of {}: {e}",
                input.display()
            )
        })?;
    }

    output_file.flush().await?;
    Ok(())
}

fn run_nfqws(args_str: &str) -> anyhow::Result<()> {
    let mut args = vec![
        "nfqws".to_string(),
        "--uid=0:0".to_string(),
        "--qnum=200".to_string(),
    ];

    if args_str.trim().is_empty() {
        args.push("-v".to_string());
    } else {
        args.extend(args_str.split_whitespace().map(String::from));
    }

    let c_args: Vec<CString> = args
        .into_iter()
        .map(|arg| CString::new(arg).unwrap())
        .collect();

    let mut ptrs: Vec<*const c_char> = c_args.iter().map(|arg| arg.as_ptr()).collect();

    unsafe {
        nfqws_main(c_args.len() as libc::c_int, ptrs.as_mut_ptr() as *mut _);
    }

    Ok(())
}


fn run_nfqws2(args_str: &str) -> anyhow::Result<()> {
    let mut args = vec![
        "nfqws2".to_string(),
        "--uid=0:0".to_string(),
        "--qnum=200".to_string(),
    ];

    if args_str.trim().is_empty() {
        args.push("-v".to_string());
    } else {
        args.extend(args_str.split_whitespace().map(String::from));
    }

    let c_args: Vec<CString> = args
        .into_iter()
        .map(|arg| CString::new(arg).unwrap())
        .collect();

    let mut ptrs: Vec<*const c_char> = c_args.iter().map(|arg| arg.as_ptr()).collect();

    unsafe {
        nfqws2_main(c_args.len() as libc::c_int, ptrs.as_mut_ptr() as *mut _);
    }

    Ok(())
}
