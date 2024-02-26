pub mod download;
pub mod fs;
pub mod unzip;

use crate::error::{Error, HunterResult};
use reqwest::{Client, Proxy};
use std::time::{Duration, Instant};

pub async fn check_proxy(duration: Duration) -> HunterResult<f64> {
    let retries = 3;
    let proxy = Proxy::all("socks5h://127.0.0.1:1086")?;
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
                if i < retries - 1 {
                    continue;
                } else {
                    return Err(Error::Request(e));
                }
            }
        }
    }

    Err(Error::Other("连通性检查失败".to_string()))
}
