use anyhow::anyhow;
use url::Url;

use super::ttdl::Ttdl;
use super::ytdl::Ytdl;
use super::Downloader;

pub struct GenericDownloader {}

impl Downloader for GenericDownloader {
    fn download(url: &Url) -> anyhow::Result<String> {
        match url.domain() {
            Some("tiktok.com") => Ttdl::download(url),
            Some("vm.tiktok.com") => Ttdl::download(url),
            Some("www.tiktok.com") => Ttdl::download(url),
            Some("vt.tiktok.com") => Ttdl::download(url),
            Some("youtube.com") => Ytdl::download(url),
            Some("youtu.be") => Ytdl::download(url),
            None => Err(anyhow!("invalid link")),
            _ => Err(anyhow!("not supported")),
        }
    }
}
