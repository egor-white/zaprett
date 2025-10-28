use cmake::Config;

fn main() {
    let dst = Config::new(".").build_target("nfqws").build();
    let lib_path = dst.join("build");
    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-lib=static=nfqws");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=netfilter_queue");
    println!("cargo:rustc-link-lib=nfnetlink");
    println!("cargo:rustc-link-lib=mnl");
}
