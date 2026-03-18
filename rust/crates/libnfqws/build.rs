use std::env;
use std::path::PathBuf;

fn main() {
    let dst = cmake::Config::new(env::var("CARGO_MANIFEST_DIR").unwrap()).build();
    println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
    println!("cargo:rustc-link-lib=dylib=nfqws");
    println!("cargo:rerun-if-changed=zapret/nfq");
    println!("cargo:rerun-if-changed=CMakeLists.txt");
    println!("cargo:rerun-if-changed=build.rs");
    let mut builder = bindgen::Builder::default();
    for header in glob::glob("zapret/nfq/*.h")
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