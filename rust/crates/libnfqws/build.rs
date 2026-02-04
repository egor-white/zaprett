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
        "DLOG_CONDUP",
        "IsTLSRecordFull",
        "DLOG_ERR",
        "IsTLSClientHello",
        "DLOG_PERROR",
        "LOG_APPEND",
        "HOSTLIST_DEBUGLOG_APPEND",
        "hexdump_limited_dlog",
        "TLSHandshakeLen",
        "IsTLSHandshakeClientHello",
        "IsTLSHandshakeFull",
        "TLSFindExtLenOffsetInHandshake",
        "TLSFindExtLen",
        "TLSFindExtInHandshake",
        "TLSHelloExtractHost",
        "TLSHelloExtractHostFromHandshake",
        "IsQUICCryptoHello",
        "QUICDraftVersion",
        "str_udphdr",
        "QUICIsLongHeader",
        "dp_init",
        "dp_list_add",
        "dp_clear",
        "dp_entry_destroy",
        "dp_list_destroy",
        "dp_list_have_autohostlist",
        "cleanup_params",
        "progname",
        "tld",
        "QUICExtractVersion",
        "QUICExtractDCID",
        "QUICDecryptInitial",
        "print_udphdr",
        "QUICDefragCrypto",
        "IsQUICInitial",
        "IsWireguardHandshakeInitiation",
        "proto_skip_ipv4",
        "IsDiscordIpDiscoveryRequest",
        "IsStunMessage",
        "proto_check_tcp",
        "proto_skip_tcp",
        "proto_check_udp",
        "proto_skip_udp",
        "proto_dissect_l3l4",
        "tcp_synack_segment",
        "tcp_syn_segment",
        "rawsend_cleanup",
        "rawsend_preinit",
        "rawsend",
        "rawsend_rp",
        "rawsend_queue",
        "wlan_info_deinit",
        "wlan_info_init",
        "wlan_info_get_rate_limited",
        "wlans",
        "wlan_ifname2ssid",
        "wlan_ifidx2ssid",
        "wlan_ssid_search_ifname",
        "wlan_ssid_search_ifidx",
        "verdict_tcp_csum_fix",
        "dpi_desync_packet",
        "verdict_udp_csum_fix",
        "unique_size_t",
        "qsort_size_t",
        "dbgprint_socket_buffers",
        "fake_http_request_default",
        "rtrim",
        "replace_char",
        "fake_tls_clienthello_default",
        "params",
        "strncasestr",
        "load_file",
        "append_to_list_file",
        "expand_bits",
        "strip_host_to_ip",
        "ntop46",
        "ntop46_port",
        "print_sockaddr",
        "saport",
        "pntoh64",
        "set_socket_buffers",
        "phton64",
        "seq_within",
        "ipv6_addr_is_zero",
        "parse_hex_str",
        "fprint_localtime",
        "file_mod_time",
        "file_mod_signature",
        "file_open_test",
        "pf_in_range",
        "pf_parse",
        "pf_is_empty",
        "fill_random_bytes",
        "fill_random_az",
        "fill_random_az09",
        "set_console_io_buffering",
        "set_env_exedir",
        "str_cidr4",
        "print_cidr4",
        "str_cidr6",
        "print_cidr6",
        "parse_cidr4",
        "parse_cidr6",
        // "__clear_cache"
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
