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
    cc::Build::new()
        .files(
            glob::glob(&format!("{}/*.c", NFQ.display()))
                .unwrap()
                .filter_map(Result::ok),
        )
        .files(
            glob::glob(&format!("{}/*.c", NFQ_CRYPTO.display()))
                .unwrap()
                .filter_map(Result::ok),
        )
        .include(&*NFQ)
        .include(&*NFQ_CRYPTO)
        .flag("-w")
        .define("main", "nfqws2_main")
        .compile("libnfqws2.a");

    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=netfilter_queue");
    println!("cargo:rustc-link-lib=nfnetlink");
    println!("cargo:rustc-link-lib=mnl");
    println!("cargo:rustc-link-lib=static=luajit-5.1");

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
