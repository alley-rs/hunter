use serde::{Deserialize, Serialize};

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
        format!(
            r#"{{"run_type": "client", "log_level": 1, "log_file": "/tmp/trojan.log", "local_addr": "{}", "local_port": {}, "remote_addr": "{}", "remote_port": {}, "password": ["{}"]}}"#,
            local_addr, local_port, self.addr, self.port, self.password
        )
    }
}
