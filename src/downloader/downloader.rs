use anyhow::anyhow;
use url::Url;

use super::ttdl::TTDL;
use super::ytdl::YTDL;

pub trait Downloader {
    fn download(url: &Url) -> anyhow::Result<String>;
}

pub struct GenericDownloader {}

impl Downloader for GenericDownloader {
    fn download(url: &Url) -> anyhow::Result<String> {
        match url.domain() {
            Some("tiktok.com") => TTDL::download(url),
            Some("vm.tiktok.com") => TTDL::download(url),
            Some("youtube.com") => YTDL::download(url),
            Some("youtu.be") => YTDL::download(url),
            None => Err(anyhow!("invalid link")),
            _ => Err(anyhow!("not supported")),
        }
    }
}
