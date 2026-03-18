use std::env;
use std::path::PathBuf;

fn main() {
    let dst = cmake::Config::new(env::var("CARGO_MANIFEST_DIR").unwrap()).build();
    println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
    println!("cargo:rustc-link-lib=dylib=nfqws2");

    println!("cargo:rerun-if-changed=CMakeLists.txt");
    println!("cargo:rerun-if-changed=zapret2/nfq2");
    println!("cargo:rerun-if-changed=build.rs");
    let mut builder = bindgen::Builder::default();
    for header in glob::glob("zapret2/nfq2/*.h")
        .unwrap()
        .filter_map(Result::ok)
    {
        builder = builder.header(header.to_string_lossy());
    }

    builder = builder.clang_arg("-Dmain=nfqws2_main");

    if let Ok(luajit) = env::var("LUAJIT") {
        builder = builder.clang_arg(format!("-I{}", luajit));
    }

    let bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .blocklist_file(".*/linux/icmp.*")
        .generate()
        .expect("Unable to generate libnfqws2");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("libnfqws2.rs"))
        .expect("Couldn't write libnfqws2");
}