use once_cell::sync::Lazy;
use std::env;
use std::path::{Path, PathBuf};

macro_rules! rel_manifest_path {
    ($name:ident, $path:expr) => {
        static $name: Lazy<PathBuf> = Lazy::new(|| {
            let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            Path::new(&manifest_dir).join($path)
        });
    };
}

rel_manifest_path!(NFQ, "zapret2/nfq2");
rel_manifest_path!(NFQ_CRYPTO, "zapret2/nfq2/crypto");

fn main() {
    const SYMBOLS: &[&str] = &[
        "DLOG",
        "net32_add",
        "net16_add",
        "tcp_find_option",
        "tcp_find_scale_factor",
        "tcp_find_mss",
        "proto_skip_ipv6",
        "proto_check_ipv4",
        "proto_check_ipv6",
        "extract_ports",
        "extract_endpoints",
        "proto_name",
        "family_from_proto",
        "str_ip",
        "print_ip",
        "str_srcdst_ip6",
        "str_ip6hdr",
        "print_ip6hdr",
        "str_tcphdr",
        "print_tcphdr",
        "l7proto_str",
        "l7_proto_match",
        "posmarker_name",
        "AnyProtoPos",
        "ResolvePos",
        "HttpPos",
        "TLSPos",
        "TLSFindExt",
        "TLSAdvanceToHostInSNI",
        "ResolveMultiPos",
        "IsHttp",
        "HttpFindHost",
        "IsHttpReply",
        "HttpReplyCode",
        "HttpExtractHeader",
        "HttpExtractHost",
        "HttpReplyLooksLikeDPIRedirect",
        "TLSVersionStr",
        "TLSRecordDataLen",
        "TLSRecordLen",
    ];
    let mut cc_builder = cc::Build::new();
    cc_builder.files(
        glob::glob(&format!("{}/*.c", NFQ.display()))
            .unwrap()
            .filter_map(Result::ok),
    );
    cc_builder.files(
        glob::glob(&format!("{}/*.c", NFQ_CRYPTO.display()))
            .unwrap()
            .filter_map(Result::ok),
    );
    cc_builder.include(&*NFQ);
    cc_builder.include(&*NFQ_CRYPTO);
    cc_builder.flag("-w");
    for &symbol in SYMBOLS {
        let val = format!("nfq2_{}", symbol);
        cc_builder.define(symbol, Some(&val[..]));
    }
    cc_builder.define("main", "nfqws2_main");
    cc_builder.compile("libnfqws2.a");

    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=netfilter_queue");
    println!("cargo:rustc-link-lib=nfnetlink");
    println!("cargo:rustc-link-lib=mnl");
    println!("cargo:rustc-link-lib=static=luajit");

    let _ = env::var("NETFILTER_LIBS")
        .map(|libs| println!("cargo:rustc-link-search=native={libs}/lib"));
    let _ = env::var("LUAJIT_LIBS")
        .map(|libs| println!("cargo:rustc-link-search=native={libs}/lib"));

    println!("cargo:rustc-link-lib=static=nfqws2");
    println!("cargo:rerun-if-changed={}", NFQ.display());
    println!("cargo:rerun-if-changed={}", NFQ_CRYPTO.display());
    println!("cargo:rerun-if-changed=build.rs");

    let mut builder = bindgen::Builder::default();

    for header in glob::glob(&format!("{}/*.h", NFQ.display()))
        .unwrap()
        .filter_map(Result::ok)
    {
        builder = builder.header(header.to_string_lossy());
    }
    if let Ok(luajit) = env::var("LUAJIT") {
        builder = builder
            .clang_arg(format!("-I{}", luajit))
            .clang_arg("-Dmain=nfqws2_main");
    }

    let bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate libnfqws2");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("libnfqws2.rs"))
        .expect("Couldn't write libnfqws2");
}
