use std::{fs::File, io::Read, path::Path, time::Duration};

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{
    header::{HeaderMap, HeaderName, USER_AGENT},
    Client, ClientBuilder,
};

use crate::downloader::status::ARGS;

pub fn create_client() -> Client {
    let mut headers = load_header(ARGS.header.as_ref()).unwrap();

    if let Some(ua) = &ARGS.user_agent {
        headers.insert(USER_AGENT, ua.parse().unwrap());
    }

    if !headers.contains_key(USER_AGENT) {
        headers.insert(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36".parse().unwrap());
    }

    ClientBuilder::new()
        .default_headers(headers)
        .connect_timeout(Duration::from_secs(60))
        .build()
        .unwrap()
}

pub fn create_bar(size: u64) -> ProgressBar {
    let style = ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] [{bar:50.cyan/blue}] {msg:<13} {bytes:>8}/{total_bytes} ({eta})",
    )
    .unwrap()
    .progress_chars("##>");
    let bar = ProgressBar::new(size);
    bar.set_style(style);
    bar
}

pub fn load_header(header_path: Option<&String>) -> Result<HeaderMap> {
    let mut header = HeaderMap::new();

    let mut load = |header_path: &str| {
        let header_path = Path::new(header_path);
        if header_path.exists() {
            let mut text = String::new();
            let mut file = File::open(header_path).unwrap();
            file.read_to_string(&mut text).unwrap();

            for line in text.lines() {
                let parts: Vec<&str> = line.split(":").collect();
                if parts.len() >= 2 {
                    let key = parts[0].trim().as_bytes();
                    let key = HeaderName::from_bytes(key).unwrap();
                    let value = parts[1..].join(":").trim().to_owned();
                    header.insert(key, value.parse().unwrap());
                }
            }
        }
    };

    match header_path {
        Some(header_path) => load(&header_path),
        None => {
            load("header");
            load("header.dn");
            load("header.txt");
        }
    }
    Ok(header)
}
