use tracing::trace;

use crate::{error::HunterResult, utils::execute::execute};

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
            "get proxy type",
        )?;

        trace!("read_proxy_type result: {}", proxy_type);

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
            "get Proxy Config Script",
        )?;

        Ok(Some((proxy_type, proxy_config_script)))
    }

    fn read_gnome_proxy_config(&self) -> HunterResult<Option<(String, String)>> {
        let mode = execute(
            "gsettings",
            vec!["get", "org.gnome.system.proxy", "mode"],
            "read proxy config mode",
        )?;

        if mode != "auto" {
            return Ok(None);
        }

        let url = execute(
            "gsettings",
            vec!["get", "org.gnome.system.proxy", "autoconfig-url"],
            "read proxy auto config url",
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
            "set proxy type",
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
            "set proxy config script",
        )?;

        Ok(())
    }

    fn set_gnome_proxy_config(&self, pac: &str) -> HunterResult<()> {
        execute(
            "gsettings",
            vec!["set", "org.gnome.system.proxy", "mode", "auto"],
            "set proxy config mode",
        )?;

        execute(
            "gsettings",
            vec!["set", "org.gnome.system.proxy", "autoconfig-url", pac],
            "set proxy auto config url",
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
            "set proxy type",
        )?;

        Ok(())
    }

    fn disable_gnome_auto_proxy(&self) -> HunterResult<()> {
        execute(
            "gsettings",
            vec!["set", "org.gnome.system.proxy", "mode", "none"],
            "set proxy config mode",
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
