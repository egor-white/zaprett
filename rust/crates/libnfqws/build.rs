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

rel_manifest_path!(NFQ, "libs/zapret/nfq");
rel_manifest_path!(NFQ_CRYPTO, "libs/zapret/nfq/crypto");

fn main() {
    cc::Build::new()
        .files(glob::glob(&format!("{}/*.c", NFQ.display()))
            .unwrap()
            .filter_map(Result::ok))
        .files(glob::glob(&format!("{}/*.c", NFQ_CRYPTO.display()))
            .unwrap()
            .filter_map(Result::ok))
        .include(&*NFQ)
        .include(&*NFQ_CRYPTO)
        .flag("-w")
        .define("main", "nfqws_main")
        .compile("libnfqws.a");

    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=netfilter_queue");
    println!("cargo:rustc-link-lib=nfnetlink");
    println!("cargo:rustc-link-lib=mnl");

    println!("cargo:rustc-link-lib=static=nfqws");
    println!("cargo:rerun-if-changed={}", NFQ.display());
    println!("cargo:rerun-if-changed={}", NFQ_CRYPTO.display());

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
