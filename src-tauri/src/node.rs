use serde::{Deserialize, Serialize};

use crate::config::EXECUTABLE_DIR;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerNode {
    name: String,
    addr: String,
    port: u16,
    password: String,
}

impl ServerNode {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn addr(&self) -> &str {
        &self.addr
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn to_string(&self, local_addr: &str, local_port: u16) -> String {
        #[cfg(target_os = "windows")]
        let config_file_path = {
            let p = EXECUTABLE_DIR
                .join("trojan.log")
                .to_str()
                .unwrap()
                .to_string();
            p.replace("\\", "\\\\")
        };

        #[cfg(not(target_os = "windows"))]
        let config_file_path = EXECUTABLE_DIR
            .join("trojan.log")
            .to_str()
            .unwrap()
            .to_string();

        format!(
            r#"{{"run_type": "client", "log_level": 1, "log_file": "{}", "local_addr": "{}", "local_port": {}, "remote_addr": "{}", "remote_port": {}, "password": ["{}"]}}"#,
            config_file_path, local_addr, local_port, self.addr, self.port, self.password
        )
    }
}
