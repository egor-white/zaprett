use std::env;
fn main() {
    let nfqws_version = env::var("NFQWS_VERSION").unwrap_or("unknown".to_string());
    let nfqws2_version = env::var("NFQWS2_VERSION").unwrap_or("unknown".to_string());
    println!("cargo:rustc-env=NFQWS_VERSION={}", nfqws_version);
    println!("cargo:rustc-env=NFQWS2_VERSION={}", nfqws2_version)
}
