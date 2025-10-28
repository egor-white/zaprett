use std::env;
use std::path::{Path, PathBuf};
use once_cell::sync::Lazy;

static NFQ: Lazy<PathBuf> = Lazy::new(|| {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    Path::new(&manifest_dir).join("libs/zapret/nfq")
});

fn main() {
    cc::Build::new()
        .file(NFQ.join("nfqws.c"))
        .include(&*NFQ)
        .compile("libnfqws.a");

    println!("cargo:rustc-link-lib=static=nfqws");
    println!("cargo:rerun-if-changed={}", NFQ.display());
}
