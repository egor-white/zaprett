use std::env;
fn main() {
    let zapret_version = env::var("ZAPRET_VERSION").unwrap_or("unknown".to_string());
    println!("cargo:rustc-env=ZAPRET_VERSION={}", zapret_version);
}
