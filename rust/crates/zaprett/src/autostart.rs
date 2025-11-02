use tokio::fs::File;
use tokio::fs;
use crate::MODULE_PATH;

pub async fn set_autostart(autostart: bool) -> Result<(), anyhow::Error> {
    let autostart_path = MODULE_PATH.join("autostart");

    if autostart {
        File::create(autostart_path).await?;
    } else {
        fs::remove_file(autostart_path).await?;
    }

    Ok(())
}

pub fn get_autostart() {
    let file = MODULE_PATH.join("autostart");
    println!("{}", file.exists());
}
