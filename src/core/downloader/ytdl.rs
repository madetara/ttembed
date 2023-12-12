use std::process::Stdio;

use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    process::Command,
};
use uuid::Uuid;

use crate::core::downloader::cmd_builder::{self, DownloadOption};

pub async fn download_file(url: &url::Url) -> anyhow::Result<String> {
    tracing::info!("downloading video from {url}");

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
                tracing::info!("process exited with status '{status}'");
            }
            Err(err) => {
                tracing::error!("failed to wait for process completion. error: {err}");
            }
        }
    });

    while let Some(line) = out_reader.next_line().await? {
        tracing::info!("{line}");
    }

    while let Some(line) = err_reader.next_line().await? {
        tracing::warn!("{line}");
    }

    Ok(filename)
}

pub async fn download_stream(url: &url::Url) -> anyhow::Result<impl AsyncRead> {
    tracing::info!("downloading video from {url}");

    let cmd = cmd_builder::build_command(url, &DownloadOption::Stream);

    let mut wrapped_cmd = Command::new("bash");

    wrapped_cmd
        .arg("-C")
        .arg(format!("{cmd:?}"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = wrapped_cmd.spawn()?;

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
                tracing::info!("process exited with status '{status}'");
            }
            Err(err) => {
                tracing::error!("failed to wait for process completion. error: {err}");
            }
        }
    });

    while let Some(line) = err_reader.next_line().await? {
        tracing::warn!("{line}");
    }

    Ok(out_reader)
}
