use super::Downloader;
use anyhow::anyhow;
use std::process::Command;
use uuid::Uuid;

pub struct Ytdl {}

impl Downloader for Ytdl {
    fn download(url: &url::Url) -> anyhow::Result<String> {
        let filename = format!("{0}.mp4", Uuid::new_v4());

        log::info!("downloading video from {url} to {filename}");

        match Command::new("yt-dlp")
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
