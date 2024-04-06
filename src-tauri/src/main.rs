// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
// mod consts;
mod error;
mod node;
mod proxy;
mod run_event;
// mod tray;
#[cfg(target_os = "linux")]
mod linux;
mod utils;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate tracing;

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use config::LogLevel;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use proxy::TrojanProcessState;
use tauri::utils::platform::target_triple;
use tauri::{AppHandle, Manager};
use time::macros::{format_description, offset};
use tracing::Level;
use tracing_subscriber::fmt::time::OffsetTime;
use url::Url;

use crate::config::{Config, AUTOSTART_DIR, CONFIG, CONFIG_DIR, EXECUTABLE_DIR};
use crate::error::{Error, HunterResult};
use crate::node::ServerNode;
use crate::proxy::{kill, EXECUTABLE_FILE, PROXY};
use crate::run_event::handle_run_event;
use crate::utils::check_proxy;
use crate::utils::download::download;
use crate::utils::fs::create_dir_if_not_exists;
use crate::utils::unzip::Zip;

#[tauri::command]
async fn proxy_state() -> HunterResult<bool> {
    trace!("查询代理状态");
    let proxy = PROXY.read().await;
    proxy.auto_proxy_url_state()
}

#[tauri::command]
async fn turn_off_proxy() -> HunterResult<()> {
    trace!("关闭代理");
    let proxy = PROXY.read().await;
    proxy.disable_auto_proxy_url()?;

    // let tray = handle.tray_handle().get_item("switch");
    // tray.set_title("开启代理")?;

    info!("已关闭系统 pac");

    Ok(())
}

#[tauri::command]
async fn turn_on_proxy() -> HunterResult<()> {
    trace!("开启代理");
    {
        let config = CONFIG.read().await;
        let proxy = PROXY.read().await;
        proxy.enable_auto_proxy_url(config.pac())?;
    }

    // let tray = handle.tray_handle().get_item("switch");
    // tray.set_title(CHECKED.to_owned() + "开启代理")?;

    info!("已设置系统 pac");

    Ok(())
}

#[tauri::command]
async fn kill_process(id: Option<u32>) -> HunterResult<()> {
    trace!("杀掉 trojan 进程");

    match id {
        Some(n) => kill(n),
        None => {
            let mut proxy = PROXY.write().await;
            kill(proxy.child_id())?;
            proxy.reset_child_id();
            proxy.switch_daemon();

            Ok(())
        }
    }
}

#[tauri::command]
async fn execute() -> HunterResult<()> {
    trace!("执行 trojan 程序");
    let mut proxy = PROXY.write().await;
    proxy.execute()
}

#[tauri::command]
async fn executable_file() -> PathBuf {
    EXECUTABLE_FILE.into()
}

#[tauri::command]
async fn get_trojan_process_state() -> HunterResult<Option<TrojanProcessState>> {
    let mut proxy = PROXY.write().await;
    let res = proxy.get_trojan_process_state().await?;

    Ok(res)
}

#[tauri::command]
async fn check_executable_file() -> bool {
    trace!("检查 trojan 可执行文件是否存在");
    let proxy = PROXY.read().await;
    let exists = proxy.check_executable_file();

    info!(message = "trojan 可执行文件状态", exists = exists);

    exists
}

#[tauri::command]
async fn check_network_connectivity() -> HunterResult<f64> {
    trace!("检测网格连通性");
    #[cfg(not(target_os = "windows"))]
    let timeout = Duration::from_secs(5);
    #[cfg(target_os = "windows")]
    // windows 上可能需要更长的超时时间
    let timeout = Duration::from_secs(30);

    match check_proxy(timeout).await {
        Ok(s) => {
            info!("网络连接正常");
            Ok(s)
        }
        Err(e) => {
            error!(mesage = "网络连接异常", error = ?e);
            Err(e)
        }
    }
}

const NON_ALPHANUMERIC: &AsciiSet = &CONTROLS.add(b'/').add(b':');

#[tauri::command]
async fn download_executable_file(window: tauri::Window, id: u32) -> HunterResult<PathBuf> {
    trace!("下载 trojan 可执行文件");
    let mut cdn = Url::parse("https://mirror.ghproxy.com")?;

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    let url =
        "https://github.com/thep0y/trojan-go/releases/latest/download/trojan-go-windows-amd64.zip";

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    let url =
        "https://github.com/thep0y/trojan-go/releases/latest/download/trojan-go-darwin-amd64.zip";

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let url =
        "https://github.com/thep0y/trojan-go/releases/latest/download/trojan-go-darwin-arm64.zip";

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let url =
        "https://github.com/thep0y/trojan-go/releases/latest/download/trojan-go-linux-amd64.zip";

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let url =
        "https://github.com/thep0y/trojan-go/releases/latest/download/trojan-go-linux-arm.zip";

    cdn.set_path(&utf8_percent_encode(url, NON_ALPHANUMERIC).to_string());

    let file_path = EXECUTABLE_DIR.join("trojan-go.zip");

    match download(
        window,
        id,
        cdn,
        &file_path,
        HashMap::from([("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36 Edg/114.0.1823.82")]),
    ).await {
            Ok(_) => Ok(file_path.to_owned()),
            Err(e) => {
                error!("下载文件时出错：{}", e);
                Err(e)
            }
        }
}

#[tauri::command]
async fn change_server_node(_handle: AppHandle, name: &str) -> HunterResult<()> {
    trace!("切换服务器");

    let mut proxy = PROXY.write().await;
    proxy.set_server_node(name).await?;

    // 切换托盘
    // for location in LOCATIONS.keys() {
    //     if prefix == location.id() {
    //         let tray_item = handle.tray_handle().get_item(prefix);
    //         tray_item.set_title(location.checked(CHECKED))?;
    //         tray_item.set_enabled(false)?;
    //     } else {
    //         let tray_item = handle.tray_handle().get_item(location.id());
    //         tray_item.set_title(location.name())?;
    //         tray_item.set_enabled(true)?;
    //     }
    // }

    info!(message = "服务器已切换", server_node_name = name);

    Ok(())
}

#[tauri::command]
async fn auto_start_state() -> bool {
    trace!("检查自启状态");

    let proxy = PROXY.read().await;
    let state = proxy.auto_start_state();

    if state {
        info!("已开机自启");
    } else {
        info!("未开机自启");
    }

    state
}

#[tauri::command]
async fn switch_auto_start(current_state: bool) -> HunterResult<()> {
    trace!("切换开机自启");
    let proxy = PROXY.read().await;
    proxy.switch_auto_start(current_state)?;

    // let tray = handle.tray_handle().get_item("autostart");
    // if current_state {
    //     tray.set_title("开机启动")?;
    // } else {
    //     tray.set_title(CHECKED.to_owned() + "开机启动")?;
    // }

    Ok(())
}

#[tauri::command]
async fn unzip(zip: Zip) -> HunterResult<()> {
    debug!(message = "解压文件", zip = ?zip);

    zip.extract(true)
}

#[tauri::command]
async fn get_config() -> Config {
    let config = CONFIG.read().await;
    config.clone()
}

#[tauri::command]
async fn update_config(config: Config) {
    // 只更新 CONFIG 变量，不写入文件，用于程序的运行时，减少 io 次数
    let mut lock = CONFIG.write().await;
    *lock = config;
}

#[tauri::command]
async fn update_local_addr(addr: &str) -> HunterResult<()> {
    trace!("update - local_addr");
    let mut config = CONFIG.write().await;
    config.set_local_addr(addr);
    info!(message = "updated", local_addr = addr);

    Ok(())
}

#[tauri::command]
async fn update_local_port(port: u16) -> HunterResult<()> {
    trace!("update - local_port");
    let mut config = CONFIG.write().await;
    config.set_local_port(port);
    info!(message = "updated", local_port = port);

    Ok(())
}

#[tauri::command]
async fn update_pac(pac: &str) -> HunterResult<()> {
    trace!("update - pac");
    let mut config = CONFIG.write().await;
    config.set_pac(pac);
    info!(message = "updated", pac = pac);

    Ok(())
}

#[tauri::command]
async fn add_server_node(server_node: ServerNode) {
    trace!("add - new server node");
    let mut config = CONFIG.write().await;
    config.add_server_node(&server_node);
    info!(message = "added",server_node = ?server_node);
}

#[tauri::command]
async fn update_server_node(index: usize, server_node: ServerNode) {
    trace!("update - server node");
    let mut config = CONFIG.write().await;
    config.update_server_node(index, server_node);
    info!(message = "updated", server_node_index = index);
}

#[tauri::command]
async fn get_using_server_node() -> HunterResult<Option<ServerNode>> {
    let config = CONFIG.read().await;

    Ok(config.get_using_server_node().await?.cloned())
}

#[tauri::command]
async fn write_trojan_config(server_node: ServerNode) -> HunterResult<()> {
    trace!("修改 trojan 配置文件");
    let config = CONFIG.read().await;
    config.write_trojan_config_file(&server_node).await?;

    info!(
        message = "trojan 配置文件写入新的节点",
        server_node_name = server_node.name()
    );
    Ok(())
}

#[tauri::command]
async fn set_log_level(level: LogLevel) {
    trace!("修改 trojan-go 日志等级");

    let mut config = CONFIG.write().await;
    config.set_log_level(level.clone());

    info!(message = "trojan-go 日志等级已修改", level = ?level);
}

#[tauri::command]
async fn switch_daemon() {
    trace!("切换后台驻留");
    let mut proxy = PROXY.write().await;
    proxy.switch_daemon();
}

#[tauri::command]
async fn get_proxy_daemon() -> bool {
    trace!("获取后台驻留状态");
    let proxy = PROXY.read().await;
    proxy.daemon()
}

#[tauri::command]
async fn exit(handle: AppHandle) {
    // 主动关闭程序
    handle.exit(0);
}

#[tokio::main]
async fn main() -> HunterResult<()> {
    create_dir_if_not_exists(&[&EXECUTABLE_DIR, &CONFIG_DIR, &AUTOSTART_DIR])?;

    #[cfg(debug_assertions)]
    let timer = OffsetTime::new(
        offset!(+8),
        format_description!("[hour]:[minute]:[second].[subsecond digits:3]"),
    );
    #[cfg(not(debug_assertions))]
    let timer = OffsetTime::new(
        offset!(+8),
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"),
    );

    // NOTE: _guard must be a top-level variable
    let (writer, _guard) = {
        let file_appender = tracing_appender::rolling::never(&*CONFIG_DIR, "hunter.log");
        tracing_appender::non_blocking(file_appender)
    };

    #[cfg(debug_assertions)]
    let writer = {
        use tracing_subscriber::fmt::writer::MakeWriterExt;
        std::io::stderr.and(writer)
    };

    let builder = tracing_subscriber::fmt()
        .with_max_level(Level::WARN)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_env_filter("hunter")
        .with_timer(timer)
        .with_writer(writer);

    #[cfg(debug_assertions)]
    builder.init();

    #[cfg(not(debug_assertions))]
    builder.json().init();

    trace!("获取系统信息");
    let platform = target_triple().map_err(|e| {
        error!(message = "获取系统信息时失败", error = ?e);
        Error::Other(e.to_string())
    })?;
    info!("平台和架构: {}", platform);

    info!(
        message =  "使用的目录",
        executable_dir = ?*EXECUTABLE_DIR,
        config_dir =  ?*CONFIG_DIR,
        autostart_dir =   ?*AUTOSTART_DIR
    );

    trace!("初始化 tauri");
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_window("main") {
                info!(message = "本程序已有窗口运行，自动聚焦到此窗口");
                w.set_focus().unwrap();
            }
        }))
        // .system_tray(tray)
        // .on_system_tray_event(handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            proxy_state,
            turn_on_proxy,
            turn_off_proxy,
            executable_file,
            get_trojan_process_state,
            check_executable_file,
            download_executable_file,
            unzip,
            execute,
            kill_process,
            check_network_connectivity,
            auto_start_state,
            switch_auto_start,
            change_server_node,
            get_config,
            get_using_server_node,
            update_config,
            update_local_addr,
            update_local_port,
            update_pac,
            add_server_node,
            update_server_node,
            write_trojan_config,
            set_log_level,
            switch_daemon,
            get_proxy_daemon,
            exit,
        ])
        .build(tauri::generate_context!())
        .map_err(|e| {
            error!(message = "创建 app 失败", error = ?e);
            e
        })?
        .run(handle_run_event);

    Ok(())
}
