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

rel_manifest_path!(NFQ, "zapret/nfq");
rel_manifest_path!(NFQ_CRYPTO, "zapret/nfq/crypto");

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
        let val = format!("nfq_{}", symbol);
        cc_builder.define(symbol, Some(&val[..]));
    }
    cc_builder.define("main", "nfqws_main");
    cc_builder.compile("libnfqws.a");

    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=netfilter_queue");
    println!("cargo:rustc-link-lib=nfnetlink");
    println!("cargo:rustc-link-lib=mnl");

    let _ = env::var("NETFILTER_LIBS")
        .map(|libs| println!("cargo:rustc-link-search=native={libs}/lib"));

    println!("cargo:rustc-link-lib=static=nfqws");
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

    builder = builder.clang_arg("-Dmain=nfqws_main");

    let bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate libnfqws");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("libnfqws.rs"))
        .expect("Couldn't write libnfqws");
}
