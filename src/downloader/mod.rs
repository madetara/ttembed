use url::Url;

pub mod generic;
mod ytdl;

pub trait Downloader {
    fn download(url: &Url) -> anyhow::Result<String>;
}
