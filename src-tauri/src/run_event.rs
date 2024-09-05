use tauri::{AppHandle, RunEvent};

use crate::{
    config::{write_config_file, CONFIG},
    proxy::{kill, PROXY},
};

pub fn handle_run_event(_app_handle: &AppHandle, event: RunEvent) {
    match event {
        tauri::RunEvent::Exit => {
            tokio::task::block_in_place(|| {
                tauri::async_runtime::block_on(async move {
                    let config = CONFIG.read().await;
                    if let Err(e) = write_config_file(&config).await {
                        error!(message = "写配置文件时出错", error = ?e);
                        return;
                    };
                    info!(message = "配置文件已更新", config = ?config);

                    let proxy = PROXY.read().await;

                    if !proxy.daemon() && proxy.child_id() > 0 {
                        kill(proxy.child_id()).unwrap();
                        // 关闭系统 pac 设置
                        proxy.disable_auto_proxy_url().unwrap();
                    }
                });
            });
        }
        // tauri::RunEvent::ExitRequested { api, .. } => {
        //     api.prevent_exit();
        //     let item_handle = app_handle.tray_handle().get_item("hide");
        //     item_handle.set_enabled(false).unwrap();
        //     item_handle.set_title("窗口不可用").unwrap();
        // }
        _ => {}
    }
}
