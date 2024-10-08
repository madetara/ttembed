use std::{collections::HashSet, env};

use anyhow::Result;
use teloxide::{
    prelude::*,
    types::{InputFile, ReplyParameters},
    update_listeners::webhooks,
};
use tokio::fs;
use tracing::instrument;
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

    Box::pin(teloxide::repl_with_listener(
        bot,
        |bot: Bot, msg: Message| async move {
            handle_message(&bot, &msg).await;

            Ok(())
        },
        listener,
    ))
    .await;

    Ok(())
}

#[instrument(skip(bot, msg), fields(chat_id = %msg.chat.id))]
async fn handle_message(bot: &Bot, msg: &Message) {
    tracing::info!("handling message");
    if let Some(text) = msg.text() {
        for url in get_valid_links(text) {
            tracing::info!("attempting to download video from {url}");
            handle_download_via_file(bot, msg, &url).await;
        }
    }
}

async fn handle_download_via_file(bot: &Bot, msg: &Message, url: &url::Url) {
    tracing::info!("downloading via file");

    match ytdl::download_file(url).await {
        Ok(filename) => {
            match fs::metadata(&filename).await {
                Ok(metadata) => {
                    if metadata.len() <= MAX_SIZE {
                        match bot
                            .send_video(msg.chat.id, InputFile::file(&filename))
                            .reply_parameters(ReplyParameters::new(msg.id))
                            .await
                        {
                            Ok(_) => {}
                            Err(err) => {
                                tracing::error!(
                                    "failed to send video. error: {error}",
                                    error = err
                                );
                            }
                        }
                    }
                }
                Err(err) => {
                    tracing::warn!(
                        "failed to get metadata for file {filename}. error: {error}",
                        error = err
                    );
                }
            }

            match fs::try_exists(&filename).await {
                Ok(true) => {
                    tracing::info!("deleting file {filename}");
                    match fs::remove_file(&filename).await.err() {
                        Some(err) => {
                            tracing::error!("failed to delete file. error: {error}", error = err);
                        }
                        None => {
                            tracing::info!("deleted file {filename}");
                        }
                    }
                }
                Ok(false) => {
                    tracing::info!("nothing to delete");
                }
                Err(err) => {
                    tracing::error!(
                        "failed to check file existence. error: {error}",
                        error = err
                    );
                }
            }
        }
        Err(err) => {
            tracing::warn!(
                "error occurred while downloading {url}. error: {error}",
                error = err
            );
        }
    }
}

fn get_valid_links(text: &str) -> HashSet<Url> {
    lazy_static! {
        static ref ALLOWED_DOMAINS: HashSet<&'static str> = HashSet::from([
            // tiktok
            "www.tiktok.com",
            "vt.tiktok.com",
            "vm.tiktok.com",
            "tiktok.com",
            // youtube
            "youtube.com",
            "youtu.be",
            "www.youtube.com",
            // twitter
            "www.x.com",
            "www.twitter.com",
            "x.com",
            "twitter.com",
            // instagram
            "instagram.com",
            "www.instagram.com",
            // vk
            "vk.com",
            "www.vk.com",
            "vk.ru",
            "www.vk.ru",
            "vkontakte.ru",
            "www.vkontakte.ru",
            "vk.cc",
            "www.vk.cc",
            // reddit
            "www.reddit.com",
            "reddit.com",
            "www.redd.it",
            "redd.it"
        ]);
    }

    tracing::info!("looking for links");
    let mut result = HashSet::new();

    for word in text.split_whitespace() {
        if let Ok(url) = Url::parse(word) {
            if let Some(domain) = url.domain() {
                if !ALLOWED_DOMAINS.contains(domain) {
                    continue;
                }

                result.insert(url);
            }
        }
    }

    let link_count = result.len();
    tracing::info!("found {link_count} valid links");

    result
}
