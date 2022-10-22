use lazy_static::lazy_static;
use std::collections::HashSet;
use std::env;
use std::fs;
use teloxide::payloads::SendVideoSetters;
use teloxide::prelude2::*;
use teloxide::types::InputFile;
use url::Url;

mod downloader;

use downloader::generic::GenericDownloader;
use downloader::Downloader;

const MAX_SIZE: u64 = 50 * 1024 * 1024;

#[tokio::main]
async fn main() {
    teloxide::enable_logging!();

    log::info!("Starting...");

    let token = env::var("TG_TOKEN").expect("Telegram token not found");

    let bot = Bot::new(token).auto_send();

    teloxide::repls2::repl(
        bot.clone(),
        |message: Message, bot: AutoSend<Bot>| async move {
            let chat_id = message.chat.id;
            log::info!("received message from {chat_id}");
            if let Some(text) = message.text() {
                for url in get_valid_links(text) {
                    log::info!("attempting to download video from {url}");
                    match GenericDownloader::download(&url) {
                        Ok(filename) => {
                            match fs::metadata(&filename) {
                                Ok(metadata) => {
                                    if metadata.len() < MAX_SIZE {
                                        bot.send_video(message.chat.id, InputFile::file(&filename))
                                            .reply_to_message_id(message.id)
                                            .await?;
                                    }
                                }
                                Err(err) => {
                                    log::warn!(
                                        "failed to get metadata for file {filename}. error: {err}"
                                    );
                                }
                            }

                            match fs::remove_file(&filename).err() {
                                Some(err) => {
                                    log::error!("failed to delete file. message: {err}");
                                }
                                None => {
                                    log::info!("deleted file {filename}")
                                }
                            }
                        }
                        Err(err) => {
                            log::warn!("error occurred while downloading {url}. error: {err}");
                        }
                    }
                }
            }

            respond(())
        },
    )
        .await;
}

fn get_valid_links(text: &str) -> Vec<Url> {
    lazy_static! {
        static ref ALLOWED_DOMAINS: HashSet<&'static str> =
            HashSet::from([
            "www.tiktok.com",
            "vt.tiktok.com",
            "vm.tiktok.com",
            "tiktok.com",
            "youtube.com",
            "youtu.be"]);
    }

    log::info!("looking for links");
    let mut result = vec![];

    for word in text.split(' ') {
        if let Ok(url) = Url::parse(word) {
            if let Some(domain) = url.domain() {
                if !ALLOWED_DOMAINS.contains(domain) {
                    continue;
                }

                result.push(url);
            }
        }
    }

    let link_count = result.len();
    log::info!("found {link_count} valid links");

    result
}
