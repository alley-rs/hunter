use std::fmt::Debug;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::{Error, HunterResult};

use super::log_level::LogLevel;
use super::node::ServerNode;
use super::trojan::get_trojan_config;

lazy_static! {
    pub static ref AUTOSTART_DIR: PathBuf = if cfg!(target_os = "macos") {
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
    };
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub(super) local_addr: String,
    pub(super) local_port: u16,
    #[serde(default)]
    pub(super) log_level: LogLevel,
    pub(super) pac: String,
    pub(super) nodes: Vec<ServerNode>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            local_addr: "127.0.0.1".to_owned(),
            local_port: 1086,
            pac: "https://mirror.ghproxy.com/https://raw.githubusercontent.com/thep0y/pac/main/blacklist.pac".to_owned(),
            log_level: Default::default(),
            nodes: Default::default()
        }
    }
}

impl Config {
    /// 返回 None 时意味着 trojan 配置文件中的节点可能已在 config.nodes
    /// 中被删除，或被人为修改的无效节点
    pub(super) async fn get_using_server_node<P: AsRef<Path> + Debug>(
        &self,
        trojan_config_path: P,
    ) -> HunterResult<Option<&ServerNode>> {
        let nodes = &self.nodes;

        match get_trojan_config(trojan_config_path).await? {
            None => Ok(None),
            Some(trojan_config) => Ok(nodes.iter().find(|node| {
                node.addr() == trojan_config.remote_addr()
                    && node.port() == trojan_config.remote_port()
                    && node.password() == trojan_config.password()
            })),
        }
    }

    pub fn set_local_addr(&mut self, local_addr: &str) {
        self.local_addr = local_addr.to_owned();
    }

    pub fn set_local_port(&mut self, local_port: u16) {
        self.local_port = local_port;
    }

    pub fn set_pac(&mut self, pac: &str) {
        self.pac = pac.to_owned();
    }

    // pub fn server_nodes(&self) -> &[ServerNode] {
    //     &self.nodes
    // }
    //
    // pub fn nodes(&self) -> &[ServerNode] {
    //     &self.nodes
    // }

    pub fn local_socks5_addr(&self) -> String {
        format!("socks5h://{}:{}", self.local_addr, self.local_port)
    }

    pub fn pac(&self) -> &str {
        &self.pac
    }

    pub async fn write_trojan_config_file<P: AsRef<Path> + Debug>(
        &self,
        trojan_config_path: P,
        server_node: &ServerNode,
    ) -> HunterResult<()> {
        let trojan_config = server_node.to_string(
            &self.local_addr,
            self.local_port,
            self.log_level.clone().into(),
        );

        fs::write(trojan_config_path.as_ref(), trojan_config)
            .await
            .map_err(|e| {
                error!(message = "写 trojan 配置失败", error = ?e, path = ?trojan_config_path);
                e.into()
            })
    }

    pub fn update_server_node(&mut self, index: usize, server_node: ServerNode) {
        if index == self.nodes.len() {
            self.add_server_node(&server_node);
        } else {
            self.nodes[index] = server_node;
        }
    }

    pub fn add_server_node(&mut self, server_node: &ServerNode) {
        let node = self
            .nodes
            .iter()
            .find(|&node| node.name() == server_node.name() || node.addr() == server_node.addr());

        if node.is_some() {
            warn!(message = "节点已存在", server_node = ?node);
            // 节点已存在，不保存
            return;
        }

        self.nodes.push(server_node.to_owned());
    }

    /// 设置 node 前应先终止 trojan 进程
    pub async fn set_server_node<P: AsRef<Path> + Debug>(
        &self,
        trojan_config_path: P,
        name: &str,
    ) -> HunterResult<()> {
        let node = self.nodes.iter().find(|&node| node.name() == name);

        let node = match node {
            Some(n) => n,
            None => {
                error!(message = "节点不存在", server_node_name = name);
                return Err(Error::Config(format!("name [{}] 不存在", name).to_owned()));
            }
        };

        self.write_trojan_config_file(trojan_config_path, node)
            .await?;

        Ok(())
    }

    pub fn set_log_level(&mut self, log_level: LogLevel) {
        self.log_level = log_level;
    }
}
