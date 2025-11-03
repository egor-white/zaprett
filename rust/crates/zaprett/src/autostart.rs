use crate::MODULE_PATH;
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::fs;
use tokio::fs::File;

static AUTOSTART: LazyLock<PathBuf> = LazyLock::new(|| MODULE_PATH.join("autostart"));

pub async fn set_autostart(autostart: bool) -> Result<(), anyhow::Error> {
    if autostart {
        File::create(&*AUTOSTART).await?;
    } else {
        fs::remove_file(&*AUTOSTART).await?;
    }

    Ok(())
}

pub fn get_autostart() -> bool {
    AUTOSTART.exists()
}
