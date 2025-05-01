use super::{
    parse::DownloadStatus,
    status::{ARGS, M},
    utils::tools::{create_bar, create_client},
};
use anyhow::{anyhow, Result};
use futures::StreamExt;
use indicatif::ProgressBar;
use reqwest::header::RANGE;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::OpenOptions,
    io::{AsyncSeekExt, AsyncWriteExt},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub start: u64,
    pub end: u64,
    pub size: u64,
    pub status: DownloadStatus,
    pub retry: u8,
    pub max_retry: u8,
    #[serde(skip_serializing)]
    pub status_diff: Option<u64>,
    #[serde(default)]
    pub downloaded_size: u64,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            id: "".to_string(),
            start: 0,
            end: 0,
            size: 0,
            status: DownloadStatus::Failed,
            retry: 0,
            max_retry: 0,
            status_diff: None,
            downloaded_size: 0,
        }
    }
}

impl Block {
    pub fn new(id: String, start: u64, end: u64, size: u64, max_retry: u8) -> Self {
        Self {
            id,
            start,
            end,
            size,
            status: DownloadStatus::Started,
            retry: 0,
            max_retry,
            status_diff: None,
            downloaded_size: 0,
        }
    }

    pub async fn download(&mut self) -> Result<()> {
        let bar = M.add(create_bar(self.size));
        loop {
            match self.status {
                DownloadStatus::Progress(p) => self.run(p, &bar).await?,
                DownloadStatus::Started => self.status = DownloadStatus::Progress(0),
                DownloadStatus::Completed => {
                    bar.set_message(format!("{} finished", self.id));
                    break;
                }
                DownloadStatus::Failed => {
                    bar.set_message(format!("{} failed!!", self.id));
                    break;
                }
            }
        }
        bar.finish();
        Ok(())
    }

    async fn run(&mut self, p: u64, bar: &ProgressBar) -> Result<()> {
        let client = create_client();
        let url = &ARGS.url;
        self.downloaded_size = p;

        let mut save_counter = 1024 * 1024 * 5;

        let range = format!("bytes={}-{}", self.start + p, self.end);
        match client.get(url).header(RANGE, range).send().await {
            Ok(res) => {
                bar.set_position(p);
                bar.set_message(format!("{} downling", self.id));
                let mut stream = res.bytes_stream();
                let mut file = OpenOptions::new()
                    .write(true)
                    .open(ARGS.save_path())
                    .await?;
                file.seek(std::io::SeekFrom::Start(self.start + p)).await?;
                // println!("{}: seek to {}", self.id, self.start + p);

                let mut status_file = OpenOptions::new()
                    .write(true)
                    .open(ARGS.status_file())
                    .await?;
                assert!(self.status_diff.is_some());

                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    let chunk_length = chunk.len() as u64;
                    file.write_all(&chunk).await?;
                    bar.inc(chunk_length);
                    self.downloaded_size += chunk_length;

                    save_counter -= chunk_length as i64;
                    if save_counter <= 0 {
                        save_counter = 1024 * 1024 * 5;

                        status_file
                            .seek(std::io::SeekFrom::Start(self.status_diff.unwrap()))
                            .await?;
                        status_file
                            .write_all(&self.downloaded_size.to_le_bytes())
                            .await?;
                    }
                }
                file.flush().await?;
                self.status = DownloadStatus::Completed;
            }
            Err(_) => {
                self.retry += 1;
                if self.retry > self.max_retry {
                    return Err(anyhow!(format!("{} faild", self.id)));
                }
            }
        }
        Ok(())
    }
}
