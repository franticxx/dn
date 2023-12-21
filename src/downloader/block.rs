use super::{
    parse::DownloadStatus,
    status::{ARGS, M, TEMP_DIR},
    utils::tools::{create_bar, create_client},
};
use anyhow::{anyhow, Result};
use futures::StreamExt;
use indicatif::ProgressBar;
use reqwest::header::RANGE;
use serde::{Deserialize, Serialize};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub start: u64,
    pub end: u64,
    pub size: u64,
    pub status: DownloadStatus,
    pub retry: u8,
    pub max_retry: u8,
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

        let range = format!("bytes={}-{}", self.start + p, self.end);
        match client.get(url).header(RANGE, range).send().await {
            Ok(res) => {
                bar.set_position(p);
                bar.set_message(format!("{} downling", self.id));
                let mut stream = res.bytes_stream();

                let file_path = TEMP_DIR.join(format!("{}.{}", ARGS.filename(), self.id));

                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .write(true)
                    .open(file_path)
                    .await?;

                while let Some(chunk) = stream.next().await {
                    let mut chunk = chunk?;
                    let chunk_length = chunk.len() as u64;
                    file.write_all_buf(&mut chunk).await?;
                    bar.inc(chunk_length);
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
