use std::fmt::Debug;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::HunterResult;

use super::node::ServerNode;

#[derive(Debug, Serialize, Deserialize)]
pub struct TrojanConfig {
    remote_addr: String,
    remote_port: u16,
    password: Vec<String>,
}

impl From<&ServerNode> for TrojanConfig {
    fn from(value: &ServerNode) -> Self {
        Self {
            remote_addr: value.addr().to_owned(),
            remote_port: value.port(),
            password: vec![value.password().to_owned()],
        }
    }
}

impl TrojanConfig {
    pub fn remote_addr(&self) -> &str {
        &self.remote_addr
    }

    pub fn remote_port(&self) -> u16 {
        self.remote_port
    }

    pub fn password(&self) -> &str {
        &self.password[0]
    }
}

pub async fn get_trojan_config<P: AsRef<Path> + Debug>(
    trojan_config_path: P,
) -> HunterResult<Option<TrojanConfig>> {
    let trojan_config_path = trojan_config_path.as_ref();

    if !trojan_config_path.exists() {
        return Ok(None);
    }

    let data = fs::read(trojan_config_path).await.map_err(|e| {
        error!(message = "读取 trojan 配置失败", error = ?e);
        e
    })?;

    let trojan_config: TrojanConfig = serde_json::from_slice(&data).map_err(|e| {
        error!(message = "反序列化 trojan 配置失败", error = ?e);
        e
    })?;

    Ok(Some(trojan_config))
}
