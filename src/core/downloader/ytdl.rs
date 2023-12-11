use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use uuid::Uuid;

use crate::core::downloader::cmd_builder::{self, DownloadOption};

pub async fn download_file(url: &url::Url) -> anyhow::Result<String> {
    log::info!("downloading video from {url}");

    let filename = format!("{0}.mp4", Uuid::new_v4());

    let mut cmd = cmd_builder::build_command(url, &DownloadOption::File(&filename));

    let mut child = cmd.spawn()?;

    let stdout = child
        .stdout
        .take()
        .expect("failed to acquire handle for stdout");

    let stderr = child
        .stderr
        .take()
        .expect("failed to acquire handnle to stderr");

    let mut out_reader = BufReader::new(stdout).lines();

    let mut err_reader = BufReader::new(stderr).lines();

    tokio::spawn(async move {
        match child.wait().await {
            Ok(status) => {
                log::info!("process exited with status '{status}'");
            }
            Err(err) => {
                log::error!("failed to wait for process completion. error: {err}");
            }
        }
    });

    while let Some(line) = out_reader.next_line().await? {
        log::info!("{line}");
    }

    while let Some(line) = err_reader.next_line().await? {
        log::warn!("{line}");
    }

    Ok(filename)
}

pub async fn download_stream(url: &url::Url) -> anyhow::Result<impl AsyncRead + Send + Unpin> {
    log::info!("downloading video from {url}");

    let mut cmd = cmd_builder::build_command(url, &DownloadOption::Stream);

    let mut child = cmd.spawn()?;

    let stdout = child
        .stdout
        .take()
        .expect("failed to acquire handle for stdout");

    let stderr = child
        .stderr
        .take()
        .expect("failed to acquire handnle to stderr");

    let out_reader = BufReader::new(stdout);

    let mut err_reader = BufReader::new(stderr).lines();

    tokio::spawn(async move {
        match child.wait().await {
            Ok(status) => {
                log::info!("process exited with status '{status}'");
            }
            Err(err) => {
                log::error!("failed to wait for process completion. error: {err}");
            }
        }
    });

    while let Some(line) = err_reader.next_line().await? {
        log::warn!("{line}");
    }

    Ok(out_reader)
}
