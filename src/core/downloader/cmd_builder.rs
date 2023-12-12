use std::{env, process::Stdio};

use tokio::process::Command;

enum Domain {
    Instagram,
    Default,
}

pub enum DownloadOption<'a> {
    File(&'a str),
}

pub fn build_command(url: &url::Url, option: &DownloadOption) -> Command {
    let mut cmd = Command::new("yt-dlp");

    add_domain_specific_options(&mut cmd, url);

    cmd.arg("--max-filesize").arg("5G");

    match option {
        DownloadOption::File(filename) => {
            cmd.arg("-o").arg(filename);
        }
    }

    cmd.arg("-f").arg("mp4").arg(url.as_str());

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    cmd
}

fn add_domain_specific_options(cmd: &mut Command, url: &url::Url) {
    match map_domain(url) {
        Domain::Instagram => {
            if let Ok(proxy_url) = get_socks_proxy_url() {
                cmd.arg("--proxy").arg(proxy_url);
            }
            if let Ok(instagram_login) = env::var("INSTAGRAM_LOGIN") {
                if let Ok(instagram_pass) = env::var("INSTAGRAM_PASS") {
                    cmd.arg("--username");
                    cmd.arg(instagram_login);
                    cmd.arg("--password");
                    cmd.arg(instagram_pass);
                }
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
