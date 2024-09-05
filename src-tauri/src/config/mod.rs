mod config;
mod log_level;
mod node;
mod trojan;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::fs;
use tokio::sync::{OnceCell, RwLock};

use crate::error::{Error, HunterResult};

pub use crate::config::config::Config;
pub use crate::config::log_level::LogLevel;
pub use crate::config::node::ServerNode;
use crate::global::APP_NAME;

const CONFIG_FILE_NAME: &str = "hunter.toml";
const TROJAN_CONFIG_FILE_NAME: &str = "config.json";

static CONFIG_MANAGER: OnceCell<ConfigManager> = OnceCell::const_new();

pub async fn get_or_init_config_manager() -> &'static ConfigManager {
    CONFIG_MANAGER
        .get_or_init(|| async { ConfigManager::new().await.unwrap() })
        .await
}

pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    config_path: PathBuf,
    trojan_config_path: PathBuf,
}

impl ConfigManager {
    pub async fn new() -> HunterResult<Self> {
        let config_dir = dirs::config_dir().unwrap().join(APP_NAME);
        if !config_dir.exists() {
            fs::create_dir(&config_dir).await?;
        }
        let config_path = config_dir.join(CONFIG_FILE_NAME);
        let trojan_config_path = config_dir.join(TROJAN_CONFIG_FILE_NAME);

        let config = if config_path.exists() {
            let data = fs::read_to_string(&config_path).await?;
            toml::from_str(&data)?
        } else {
            Config::default()
        };

        let manager = Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            trojan_config_path,
        };

        Ok(manager)
    }

    pub fn trojan_config_path(&self) -> &Path {
        &self.trojan_config_path
    }

    async fn save_config(&self) -> HunterResult<()> {
        let config = self.config.read().await;
        let content = toml::to_string(&*config)?;
        fs::write(&self.config_path, content).await?;
        Ok(())
    }

    pub async fn update_config<F>(&self, updater: F) -> HunterResult<()>
    where
        F: FnOnce(&mut Config),
    {
        let mut config = self.config.write().await;
        updater(&mut config);
        self.save_config().await
    }

    pub async fn get_config(&self) -> Config {
        self.config.read().await.clone()
    }

    pub async fn set_server_node(&self, name: &str) -> HunterResult<()> {
        let config = self.config.read().await;
        let node = config
            .nodes
            .iter()
            .find(|&node| node.name() == name)
            .ok_or_else(|| Error::Config(format!("name [{}] 不存在", name)))?;

        let trojan_config = node.to_string(
            &config.local_addr,
            config.local_port,
            config.log_level.clone() as i8,
        );

        fs::write(&self.trojan_config_path, trojan_config).await?;

        Ok(())
    }

    pub async fn get_using_server_node(&self) -> HunterResult<Option<ServerNode>> {
        let config = self.config.read().await;
        let node = config
            .get_using_server_node(&self.trojan_config_path)
            .await?;

        Ok(node.cloned())
    }

    pub async fn write_trojan_config_file(&self, server_node: &ServerNode) -> HunterResult<()> {
        let config = self.config.read().await;
        config
            .write_trojan_config_file(&self.trojan_config_path, server_node)
            .await
    }

    //pub async fn set_server_node(&self, name: &str) {
    //    let config = self.config.read().await;
    //    config.set_server_node(&self.trojan_config_path, name)
    //}
}
