use anyhow::anyhow;
use std::process::Command;
use uuid::Uuid;

use super::Downloader;

pub struct Ttdl {}

impl Downloader for Ttdl {
    fn download(url: &url::Url) -> anyhow::Result<String> {
        let filename = format!("{0}.mp4", Uuid::new_v4());

        log::info!("downloading video from {url} to {filename}");

        match Command::new("python3")
            .arg("vendor/tiktok.py")
            .arg(url.as_str())
            .arg(&filename)
            .output()
        {
            Ok(_) => Ok(filename),
            Err(err) => Err(anyhow!(err)),
        }
    }
}
