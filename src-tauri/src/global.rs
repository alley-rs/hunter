use std::{fs, path::PathBuf, sync::LazyLock};

const APP_NAME: &str = "hunter";

pub(super) static EXECUTABLE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let dir = dirs::cache_dir().unwrap().join(APP_NAME);
    if !dir.exists() {
        fs::create_dir(&dir).expect("Failed to create cache dir");
    }

    dir
});

pub static AUTOSTART_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    if cfg!(target_os = "macos") {
        dirs::home_dir()
            .unwrap()
            .join("Library")
            .join("LaunchAgents")
    } else if cfg!(target_os = "windows") {
        dirs::data_dir()
            .unwrap()
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Startup")
    } else {
        dirs::config_dir().unwrap().join("autostart")
    }
});
