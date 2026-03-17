#[cfg(target_os = "android")]
pub mod path {
    use std::path::Path;
    use std::sync::LazyLock;
    pub static MODULE_PATH: LazyLock<&Path> =
        LazyLock::new(|| Path::new("/data/adb/modules/zaprett"));
    pub static ZAPRETT_DIR_PATH: LazyLock<&Path> =
        LazyLock::new(|| Path::new("/storage/emulated/0/zaprett"));
    pub static ZAPRETT_LIBS_PATH: LazyLock<&Path> =
        LazyLock::new(|| Path::new("/storage/emulated/0/zaprett/files/strategies/nfqws2/libs"));
}

// Only for testing
#[cfg(target_os = "linux")]
pub mod path {
    use std::path::Path;
    use std::sync::LazyLock;

    pub static MODULE_PATH: LazyLock<&Path> =
        LazyLock::new(|| Path::new("zaprett_module"));
    pub static ZAPRETT_DIR_PATH: LazyLock<&Path> =
        LazyLock::new(|| Path::new("zaprett_dir"));
    pub static ZAPRETT_LIBS_PATH: LazyLock<&Path> =
        LazyLock::new(|| Path::new("zaprett_dir/strategies/nfqws2/libs"));
}