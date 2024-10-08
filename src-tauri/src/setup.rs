use time::macros::{format_description, offset};
use tracing::Level;
use tracing_subscriber::fmt::time::OffsetTime;

pub(super) fn setup_logging() {
    let fmt = if cfg!(debug_assertions) {
        format_description!("[hour]:[minute]:[second].[subsecond digits:3]")
    } else {
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]")
    };

    let timer = OffsetTime::new(offset!(+8), fmt);

    #[cfg(all(desktop, not(debug_assertions)))]
    let writer = {
        use crate::global::APP_NAME;
        use std::{fs::File, sync::Mutex};
        let dir = dirs::config_dir().unwrap().join(APP_NAME);
        let log_file = File::create(dir.join("lsar.log")).expect("Failed to create the log file");
        Mutex::new(log_file)
    };

    #[cfg(any(debug_assertions, mobile))]
    let writer = std::io::stderr;

    let builder = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter("hunter_lib")
        .with_target(false)
        .with_timer(timer)
        .with_writer(writer);

    if cfg!(debug_assertions) {
        builder.init();
    } else {
        builder.json().init();
    }
}

#[cfg(desktop)]
fn setup_desktop_features(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing update plugin");
    app.handle()
        .plugin(tauri_plugin_updater::Builder::new().build())?;

    info!("Spawning update check task");
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = crate::update::update(handle).await {
            error!("Failed to check for updates: {:?}", e);
        }
    });

    Ok(())
}

pub(super) fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Setting up application: version {}",
        app.package_info().version
    );

    #[cfg(desktop)]
    setup_desktop_features(app)?;

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    apply_window_effect(app)?;

    info!("Application setup completed");

    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn apply_window_effect(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::Manager;

    let window = app.get_webview_window("main").unwrap();

    #[cfg(target_os = "macos")]
    {
        info!("Applying vibrancy effect on macOS");
        use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
        apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)?;
    }

    #[cfg(target_os = "windows")]
    {
        info!("Applying acrylic effect on Windows");
        use window_vibrancy::apply_acrylic;
        apply_acrylic(&window, Some((18, 18, 18, 125)))?;
    }

    Ok(())
}
