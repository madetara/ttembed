use std::{env, process::Stdio};

use tokio::process::Command;
use uuid::Uuid;

enum Domain {
    Instagram,
    Default,
}

pub fn build_command(url: &url::Url) -> (Command, String) {
    let mut cmd = Command::new("yt-dlp");

    add_domain_specific_options(&mut cmd, url);

    let filename = format!("{0}.mp4", Uuid::new_v4());

    cmd.arg("--max-filesize")
        .arg("50M")
        .arg("-o")
        .arg(&filename)
        .arg("-f")
        .arg("mp4")
        .arg(url.as_str());

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    (cmd, filename)
}

fn add_domain_specific_options(cmd: &mut Command, url: &url::Url) {
    match map_domain(url) {
        Domain::Instagram => {
            if let Ok(proxy_url) = get_socks_proxy_url() {
                cmd.arg("--proxy").arg(proxy_url);
            }
        }
        Domain::Default => {}
    }
}

fn map_domain(url: &url::Url) -> Domain {
    match url.domain() {
        Some("instagram.com" | "www.instagram.com") => Domain::Instagram,
        _ => Domain::Default,
    }
}

fn get_socks_proxy_url() -> anyhow::Result<String> {
    let proxy_user = env::var("PROXY_USER")?;
    let proxy_pass = env::var("PROXY_PASS")?;
    let proxy_address = env::var("PROXY_ADDRESS")?;

    Ok(format!(
        "socks5://{proxy_user}:{proxy_pass}@{proxy_address}/"
    ))
}
