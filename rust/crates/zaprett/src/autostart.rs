use crate::MODULE_PATH;
use tokio::fs;
use tokio::fs::File;

pub async fn set_autostart() -> Result<(), anyhow::Error> {
    let autostart_path = MODULE_PATH.join("autostart");

    if !get_autostart() {
        File::create(MODULE_PATH.join("autostart")).await?;
    } else {
        fs::remove_file(autostart_path).await?;
    }

    println!("{}", get_autostart());

    Ok(())
}

pub fn get_autostart() -> bool {
    return MODULE_PATH.join("autostart").exists();
}
