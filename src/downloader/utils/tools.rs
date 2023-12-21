use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{
    header::{HeaderMap, USER_AGENT},
    Client, ClientBuilder,
};

pub fn create_client() -> Client {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36".parse().unwrap());
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
