use url::Url;

pub mod generic;
mod ttdl;
mod ytdl;

pub trait Downloader {
    fn download(url: &Url) -> anyhow::Result<String>;
}
