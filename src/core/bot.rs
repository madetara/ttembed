use std::{collections::HashSet, env};

use anyhow::Result;
use teloxide::{prelude::*, types::InputFile, update_listeners::webhooks};
use tokio::fs;
use url::Url;

use crate::core::downloader::ytdl;

const MAX_SIZE: u64 = 50 * 1024 * 1024;

pub async fn run() -> Result<()> {
    let token = env::var("TG_TOKEN").expect("Telegram token not found");
    let bot = Bot::new(token);

    let bot_url = env::var("BOT_URL")
        .expect("BOT_URL not set")
        .parse()
        .expect("BOT_URL is in incorrect format");

    let bot_port = env::var("BOT_PORT")
        .expect("BOT_PORT not set")
        .parse::<u16>()
        .expect("BOT_PORT is not a number");

    let listener = webhooks::axum(
        bot.clone(),
        webhooks::Options::new(([0, 0, 0, 0], bot_port).into(), bot_url),
    )
    .await
    .expect("Webhook creation failed");

    teloxide::repl_with_listener(
        bot,
        |bot: Bot, msg: Message| async move {
            let chat_id = msg.chat.id;
            log::info!("received message from {chat_id}");

            handle_message(&bot, &msg).await;

            Ok(())
        },
        listener,
    )
    .await;

    Ok(())
}

async fn handle_message(bot: &Bot, msg: &Message) {
    if let Some(text) = msg.text() {
        for url in get_valid_links(text) {
            log::info!("attempting to download video from {url}");
            match ytdl::download(&url).await {
                Ok(filename) => {
                    match fs::metadata(&filename).await {
                        Ok(metadata) => {
                            if metadata.len() <= MAX_SIZE {
                                match bot
                                    .send_video(msg.chat.id, InputFile::file(&filename))
                                    .reply_to_message_id(msg.id)
                                    .await
                                {
                                    Ok(_) => {}
                                    Err(err) => {
                                        log::error!("failed to send video. error: {err}")
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            log::warn!("failed to get metadata for file {filename}. error: {err}");
                        }
                    }

                    match fs::remove_file(&filename).await.err() {
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
}

fn get_valid_links(text: &str) -> Vec<Url> {
    lazy_static! {
        static ref ALLOWED_DOMAINS: HashSet<&'static str> = HashSet::from([
            "www.tiktok.com",
            "vt.tiktok.com",
            "vm.tiktok.com",
            "tiktok.com",
            "youtube.com",
            "youtu.be",
            "x.com",
            "twitter.com"
        ]);
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
