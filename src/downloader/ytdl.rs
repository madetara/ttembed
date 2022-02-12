use crate::downloader::downloader::Downloader;
use anyhow::anyhow;
use std::process::Command;
use uuid::Uuid;

pub struct YTDL {}

impl Downloader for YTDL {
    fn download(url: &url::Url) -> anyhow::Result<String> {
        let filename = format!("{0}.mp4", Uuid::new_v4().to_string());

        log::info!("downloading video from {url} to {filename}");

        match Command::new("youtube-dl")
            .arg("--default-search")
            .arg("auto")
            .arg("--max-filesize")
            .arg("50M")
            .arg("-o")
            .arg(&filename)
            .arg("-f")
            .arg("mp4")
            .arg(format!("\"{0}\"", url.as_str()))
            .output()
        {
            Ok(_) => Ok(filename),
            Err(err) => Err(anyhow!(err)),
        }
    }
}
