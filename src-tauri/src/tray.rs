use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, SystemTraySubmenu,
};

use crate::{
    config::CONFIG,
    consts::{
        CHECKED, TRAY_CHANGE_LOCATION_EVENT, TRAY_SWITCH_AUTO_START_EVENT, TRAY_SWITCH_PROXY_EVENT,
    },
    error::HunterResult,
    node::ServerNode,
    proxy::PROXY,
};

pub fn tray_new_server_nodes(
    server_nodes: &[ServerNode],
    current_node: Option<&ServerNode>,
) -> SystemTrayMenu {
    let mut sub_menu_items = SystemTrayMenu::new();

    match current_node {
        None => {
            for node in server_nodes.iter() {
                let item = CustomMenuItem::new(node.name(), node.name());
                sub_menu_items = sub_menu_items.add_item(item);
            }
        }
        Some(n) => {
            for node in server_nodes.iter() {
                if n.name() == node.name() {
                    let mut item =
                        CustomMenuItem::new(node.name(), CHECKED.to_owned() + node.name());
                    item.enabled = false;
                    sub_menu_items = sub_menu_items.add_item(item);
                } else {
                    let item = CustomMenuItem::new(node.name(), node.name());
                    sub_menu_items = sub_menu_items.add_item(item);
                }
            }
        }
    }

    sub_menu_items
}

pub async fn tray_switch_proxy(handle: &AppHandle, pac: &str) -> HunterResult<bool> {
    let proxy = PROXY.read().await;
    let state = proxy.auto_proxy_url_state()?;
    if state {
        proxy.disable_auto_proxy_url()?;
        let tray = handle.tray_handle().get_item("switch");
        tray.set_title("开启代理")?;
    } else {
        proxy.enable_auto_proxy_url(pac)?;
        let tray = handle.tray_handle().get_item("switch");
        tray.set_title(CHECKED.to_owned() + "开启代理")?;
    }

    Ok(!state)
}

pub async fn tray_switch_auto_start(handle: &AppHandle, pac: &str) -> HunterResult<bool> {
    let proxy = PROXY.read().await;
    let state = proxy.auto_start_state();
    proxy.switch_auto_start(state)?;
    if state {
        let tray = handle.tray_handle().get_item("autostart");
        tray.set_title("开机启动")?;
    } else {
        proxy.enable_auto_proxy_url(pac)?;
        let tray = handle.tray_handle().get_item("autostart");
        tray.set_title(CHECKED.to_owned() + "开机启动")?;
    }

    Ok(!state)
}

pub fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        // TODO: 托盘改变程序状态后通过 emit 通知前端刷新页面
        SystemTrayEvent::LeftClick {
            position: _,
            size: _,
            ..
        } => {
            println!("system tray received a left click");
        }
        SystemTrayEvent::RightClick {
            position: _,
            size: _,
            ..
        } => {
            println!("system tray received a right click");
        }
        SystemTrayEvent::DoubleClick {
            position: _,
            size: _,
            ..
        } => {
            // macos 上无法响应双击事件
            println!("system tray received a double click");
            let window = match app.get_window("main") {
                Some(w) => w,
                None => {
                    warn!("前端窗口已关闭，无法通过双击打开窗口");

                    return;
                }
            };
            let is_visible = window.is_visible().unwrap();
            if !is_visible {
                window.show().unwrap();
                let item_handle = app.tray_handle().get_item("quit");
                item_handle.set_title("隐藏").unwrap();
            }
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            let item_handle = app.tray_handle().get_item(&id);

            match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "hide" => {
                    let window = match app.get_window("main") {
                        Some(w) => w,
                        None => {
                            warn!("前端窗口已关闭，不修改隐藏按钮状态");

                            return;
                        }
                    };

                    let is_visible = window.is_visible().unwrap();
                    if is_visible {
                        window.hide().unwrap();
                        item_handle.set_title("显示").unwrap();
                    } else {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        item_handle.set_title("隐藏").unwrap();
                    }
                }

                "switch" => {
                    let a = app.clone();
                    tokio::runtime::Handle::current().spawn(async move {
                        let config = CONFIG.read().await;

                        let result = tray_switch_proxy(&a, config.pac()).await.unwrap();

                        let window = match a.get_window("main") {
                            Some(w) => w,
                            None => {
                                warn!("前端窗口已关闭，不通知事件");

                                return;
                            }
                        };

                        window.emit(TRAY_SWITCH_PROXY_EVENT, result).unwrap();
                    });
                }
                "autostart" => {
                    let a = app.clone();
                    tokio::runtime::Handle::current().spawn(async move {
                        let config = CONFIG.read().await;

                        let result = tray_switch_auto_start(&a, config.pac()).await.unwrap();

                        let window = match a.get_window("main") {
                            Some(w) => w,
                            None => {
                                warn!("前端窗口已关闭，不通知事件");

                                return;
                            }
                        };
                        window.emit(TRAY_SWITCH_AUTO_START_EVENT, result).unwrap();
                    });
                }

                // TODO: 托盘切换服务器应完成后台清理、重新启动的全过程
                // "rn" => {
                //     let a = app.clone();
                //     tokio::runtime::Handle::current().spawn(async move {
                //         tray_change_server_node(a, Location::SanJose).await;
                //     });
                // }
                // "gg" => {
                //     let a = app.clone();
                //     tokio::runtime::Handle::current().spawn(async move {
                //         tray_change_server_node(a, Location::HongKong).await;
                //     });
                // }
                // "dy" => {
                //     let a = app.clone();
                //     tokio::runtime::Handle::current().spawn(async move {
                //         tray_change_server_node(a, Location::Australia).await;
                //     });
                // }
                _ => {}
            }
        }
        _ => {}
    }
}

pub async fn new_tray() -> HunterResult<SystemTray> {
    let sub_menu = {
        let config = CONFIG.read().await;

        let current_node = config.get_using_server_node().await?;
        SystemTraySubmenu::new(
            "服务器",
            tray_new_server_nodes(config.server_nodes(), current_node),
        )
    };

    let proxy = PROXY.read().await;

    let switch = CustomMenuItem::new(
        "switch",
        if proxy.auto_proxy_url_state()? {
            CHECKED.to_owned() + "开启代理"
        } else {
            "开启代理".to_owned()
        },
    );
    let autostart = CustomMenuItem::new(
        "autostart",
        if proxy.auto_start_state() {
            CHECKED.to_owned() + "开机启动"
        } else {
            "开机启动".to_owned()
        },
    );
    let quit = CustomMenuItem::new("quit", "退出");
    let hide = CustomMenuItem::new("hide", "隐藏");

    let tray_menu = SystemTrayMenu::new()
        .add_item(switch)
        .add_item(autostart)
        .add_submenu(sub_menu)
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(hide);
    let tray = SystemTray::new().with_menu(tray_menu);

    Ok(tray)
}

async fn tray_change_server_node(app: AppHandle, name: &str) -> HunterResult<()> {
    let mut proxy = PROXY.write().await;
    proxy.set_server_node(name).await?;

    if proxy.auto_start_state() {
        // 如果开启了自动，需要重写自启脚本
        proxy.switch_auto_start(false)?;
    }

    // 窗口关闭后不通知前端事件
    let window = match app.get_window("main") {
        Some(w) => w,
        None => return Ok(()),
    };

    window.emit(TRAY_CHANGE_LOCATION_EVENT, name).unwrap();

    Ok(())
}
