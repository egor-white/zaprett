use tokio::fs;
use tokio::fs::File;
use crate::path::path::MODULE_PATH;

pub async fn set_autostart() -> Result<(), anyhow::Error> {
    let autostart_path = MODULE_PATH.join("autostart");

    if !get_autostart() {
        File::create(autostart_path).await?;
    } else {
        fs::remove_file(autostart_path).await?;
    }

    println!("{}", get_autostart());

    Ok(())
}

pub fn get_autostart() -> bool {
    MODULE_PATH.join("autostart").exists()
}
