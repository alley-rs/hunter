use std::env;

use crate::{error::HunterResult, utils::execute::execute};

pub fn get_desktop_from_env() -> HunterResult<String> {
    env::var("XDG_SESSION_DESKTOP").map_err(|e| {
        error!(message = "获取桌面环境变量时出错", error = ?e);
        e.into()
    })
}

#[derive(Debug, Clone)]
pub enum Desktop {
    KDE,
    GNOME,
}

impl Desktop {
    fn read_kde_proxy_config(&self) -> HunterResult<Option<(String, String)>> {
        let proxy_type = execute(
            "kreadconfig5",
            vec![
                "--file",
                "kioslaverc",
                "--group",
                "Proxy Settings",
                "--key",
                "ProxyType",
            ],
            "获取代理类型",
        )?;

        info!(message = "代理类型", r#type = proxy_type);

        if proxy_type != "2" {
            return Ok(None);
        }

        let proxy_config_script = execute(
            "kreadconfig5",
            vec![
                "--file",
                "kioslaverc",
                "--group",
                "Proxy Settings",
                "--key",
                "Proxy Config Script",
            ],
            "获取自动代理脚本",
        )?;

        Ok(Some((proxy_type, proxy_config_script)))
    }

    fn read_gnome_proxy_config(&self) -> HunterResult<Option<(String, String)>> {
        let mode = execute(
            "gsettings",
            vec!["get", "org.gnome.system.proxy", "mode"],
            "获取代理模式",
        )?;

        if mode != "auto" {
            return Ok(None);
        }

        let url = execute(
            "gsettings",
            vec!["get", "org.gnome.system.proxy", "autoconfig-url"],
            "获取自动代理脚本",
        )?;

        Ok(Some((mode, url)))
    }

    pub fn read_proxy_config(&self) -> HunterResult<Option<(String, String)>> {
        match self {
            Desktop::KDE => self.read_kde_proxy_config(),
            Desktop::GNOME => self.read_gnome_proxy_config(),
        }
    }

    fn set_kde_proxy_config(&self, pac: &str) -> HunterResult<()> {
        execute(
            "kwriteconfig5",
            vec![
                "--file",
                "kioslaverc",
                "--group",
                "Proxy Settings",
                "--key",
                "ProxyType",
                "2",
            ],
            "设置代理类型",
        )?;

        execute(
            "kwriteconfig5",
            vec![
                "--file",
                "kioslaverc",
                "--group",
                "Proxy Settings",
                "--key",
                "Proxy Config Script",
                pac,
            ],
            "设置自动代理脚本",
        )?;

        Ok(())
    }

    fn set_gnome_proxy_config(&self, pac: &str) -> HunterResult<()> {
        execute(
            "gsettings",
            vec!["set", "org.gnome.system.proxy", "mode", "auto"],
            "设置代理模式",
        )?;

        execute(
            "gsettings",
            vec!["set", "org.gnome.system.proxy", "autoconfig-url", pac],
            "设置自动代理脚本",
        )?;

        Ok(())
    }

    pub fn set_proxy_config(&self, pac: &str) -> HunterResult<()> {
        match self {
            Desktop::KDE => self.set_kde_proxy_config(pac),
            Desktop::GNOME => self.set_gnome_proxy_config(pac),
        }
    }

    fn disable_kde_auto_proxy(&self) -> HunterResult<()> {
        execute(
            "kwriteconfig5",
            vec![
                "--file",
                "kioslaverc",
                "--group",
                "Proxy Settings",
                "--key",
                "ProxyType",
                "0",
            ],
            "设置代理类型",
        )?;

        Ok(())
    }

    fn disable_gnome_auto_proxy(&self) -> HunterResult<()> {
        execute(
            "gsettings",
            vec!["set", "org.gnome.system.proxy", "mode", "none"],
            "设置代理模式",
        )?;

        Ok(())
    }

    pub fn disable_auto_proxy(&self) -> HunterResult<()> {
        match self {
            Desktop::KDE => self.disable_kde_auto_proxy(),
            Desktop::GNOME => self.disable_gnome_auto_proxy(),
        }
    }
}

impl TryFrom<&str> for Desktop {
    type Error = String;
    fn try_from(value: &str) -> std::result::Result<Self, String> {
        match value {
            "KDE" => Ok(Desktop::KDE),
            "GNOME" => Ok(Desktop::GNOME),
            _ => Err(format!("unkown desktop: {}", value)),
        }
    }
}
