use tokio::io::{AsyncBufReadExt, BufReader};

use crate::core::downloader::cmd_builder;

pub async fn download(url: &url::Url) -> anyhow::Result<String> {
    log::info!("downloading video from {url}");

    let (mut cmd, filename) = cmd_builder::build_command(url);

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
