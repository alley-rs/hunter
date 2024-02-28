pub mod download;
pub mod fs;
pub mod unzip;

use reqwest::{Client, Proxy};
use std::time::{Duration, Instant};
use tracing::{debug, info, trace};

use crate::{
    config::CONFIG,
    error::{Error, HunterResult},
};

pub async fn check_proxy(duration: Duration) -> HunterResult<f64> {
    info!("检测代理，超时时间：{:?}", duration);
    let config = CONFIG.read().await;

    let retries = 3;
    let proxy = Proxy::all(config.local_socks5_addr())?;
    let client = Client::builder().proxy(proxy).build()?;

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
                    "状态码：{} 响应头：{:?} 耗时：{:?}",
                    &r.status(),
                    &r.headers(),
                    cost
                );
                return Ok(cost.as_secs_f64());
            }
            Err(e) => {
                if i == retries - 1 {
                    return Err(Error::Request(e));
                }
            }
        }
    }

    Err(Error::Other("连通性检查失败".to_string()))
}
