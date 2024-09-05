use tauri::{AppHandle, RunEvent};

use crate::proxy::{kill, PROXY};

pub fn handle_run_event(_app_handle: &AppHandle, event: RunEvent) {
    if let tauri::RunEvent::Exit = event {
        tokio::task::block_in_place(|| {
            tauri::async_runtime::block_on(async move {
                let proxy = PROXY.read().await;

                if !proxy.daemon() && proxy.child_id() > 0 {
                    kill(proxy.child_id()).unwrap();
                    // 关闭系统 pac 设置
                    proxy.disable_auto_proxy_url().unwrap();
                }
            });
        });
    }
}
