pub mod download;
pub mod execute;
pub mod fs;
pub mod unzip;

use reqwest::{Client, Proxy};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, trace};

use crate::{
    config::CONFIG,
    error::{Error, HunterResult},
};

pub async fn check_proxy(duration: Duration) -> HunterResult<f64> {
    info!("检测代理，超时时间：{:?}", duration);
    let config = CONFIG.read().await;

    let retries = 3;
    let proxy = Proxy::all(config.local_socks5_addr()).map_err(|e| {
        error!(message = "创建代理失败", error = ?e);
        e
    })?;

    let client = Client::builder().proxy(proxy).build().map_err(|e| {
        error!(message = "创建 reqwest client 失败", error =?e);
        e
    })?;

    let start = Instant::now();

    for i in 0..retries {
        trace!("第 {} 次检测", i + 1);
        match client
            .head("http://google.com")
            .timeout(duration)
            .send()
            .await
        {
            Ok(r) => {
                let cost = start.elapsed();
                debug!(
                    status = ?&r.status(),
                    headers = ?&r.headers(),
                    duration = ?cost
                );

                return Ok(cost.as_secs_f64());
            }
            Err(e) => {
                if i == retries - 1 {
                    error!(message = "已请求 google 3 次均失败", error = ?e);
                    return Err(Error::Request(e));
                }
            }
        }
    }

    Err(Error::Other("连通性检查失败".to_string()))
}
