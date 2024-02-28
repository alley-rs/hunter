use crate::error::HunterResult;
use futures_util::TryStreamExt;
use reqwest::IntoUrl;
use serde::Serialize;
use std::{collections::HashMap, path::Path};
use tauri::{Runtime, Window};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::error;

pub const DOWNLOAD_EVENT: &str = "download://progress";

#[derive(Clone, Serialize)]
struct ProgressPayload {
    id: u32,
    progress: u64,
    total: u64,
}

pub async fn download<R: Runtime, P: AsRef<Path>, U: IntoUrl>(
    window: Window<R>,
    id: u32,
    url: U,
    file_path: P,
    headers: HashMap<&str, &str>,
) -> HunterResult<u32> {
    let builder = reqwest::Client::builder();
    let client = builder.https_only(true).build().map_err(|e| {
        error!(message = "创建 request client 失败", error = ?e);
        e
    })?;

    let mut request = client.get(url);
    // Loop trought the headers keys and values
    // and add them to the request object.
    for (key, value) in headers {
        request = request.header(key, value);
    }

    let response = request.send().await?;
    let total = response.content_length().unwrap_or(0);

    let mut file = File::create(file_path).await.map_err(|e| {
        error!(message = "创建下载文件失败", error = ?e);
        e
    })?;

    let mut stream = response.bytes_stream();

    let mut downloaded_len: u64 = 0;

    while let Some(chunk) = stream.try_next().await? {
        file.write_all(&chunk).await?;

        downloaded_len += chunk.len() as u64;

        let _ = window.emit(
            DOWNLOAD_EVENT,
            ProgressPayload {
                id,
                progress: downloaded_len,
                total,
            },
        );
    }

    Ok(id)
}
