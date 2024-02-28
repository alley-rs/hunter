use tauri::{AppHandle, RunEvent, UpdaterEvent};
use tracing::{error, info, trace};

use crate::{
    config::{write_config_file, CONFIG},
    proxy::{kill, PROXY},
};

pub fn handle_run_event(_app_handle: &AppHandle, event: RunEvent) {
    match event {
        tauri::RunEvent::Updater(e) => match e {
            UpdaterEvent::UpdateAvailable {
                body,
                date,
                version,
            } => {
                info!("版本有更新: {} {:?} {}", body, date, version);
            }
            UpdaterEvent::Pending => {
                info!("准备下载新版本");
            }
            UpdaterEvent::DownloadProgress {
                chunk_length,
                content_length,
            } => {
                trace!("正在下载: {}/{:?}", chunk_length, content_length);
            }
            UpdaterEvent::Downloaded => {
                info!("新版本已下载");
            }
            UpdaterEvent::Updated => {
                info!("更新完成");
            }
            UpdaterEvent::AlreadyUpToDate => {
                info!("当前已是最新版本");
            }
            UpdaterEvent::Error(error) => {
                error!("更新失败: {}", error);
            }
        },
        tauri::RunEvent::Exit => {
            tokio::task::block_in_place(|| {
                tauri::async_runtime::block_on(async move {
                    let config = CONFIG.read().await;
                    if let Err(e) = write_config_file(&config).await {
                        error!("写配置文件时出错：{}", e);
                    };
                    info!("配置文件已更新 config: {:?}", config);

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
