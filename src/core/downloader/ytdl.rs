use std::process::Stdio;

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use uuid::Uuid;

pub async fn download(url: &url::Url) -> anyhow::Result<String> {
    let filename = format!("{0}.mp4", Uuid::new_v4());

    log::info!("downloading video from {url} to {filename}");

    let mut cmd = Command::new("yt-dlp");

    cmd.arg("--max-filesize")
        .arg("50M")
        .arg("-o")
        .arg(&filename)
        .arg("-f")
        .arg("mp4")
        .arg(url.as_str());

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

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
