use anyhow::Result;
use futures::{future::join_all, StreamExt};
use indicatif::HumanBytes;
use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE};
use std::fs::create_dir_all;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::downloader::block::Block;
use crate::downloader::status::{DnStatus, M};
use crate::downloader::utils::tools::{create_bar, create_client};

use super::status::ARGS;

pub struct DownloadManager;

impl DownloadManager {
    /// 获取文件大小
    async fn get_file_size() -> Result<(bool, Option<u64>)> {
        let client = create_client();
        let res = client.head(&ARGS.url).send().await?;
        let headers = res.headers();
        let is_resumed = headers.contains_key(ACCEPT_RANGES) || headers.contains_key(CONTENT_RANGE);
        let mut file_size = None;
        if let Some(length) = res.headers().get(CONTENT_LENGTH) {
            let length = length.to_str()?.parse()?;
            file_size = Some(length);
        }
        Ok((is_resumed, file_size))
    }

    /// 单线程下载
    async fn download_one(file_size: Option<u64>) -> Result<()> {
        let client = create_client();
        let response = client.get(&ARGS.url).send().await?;
        let mut file = File::create(ARGS.save_path()).await.unwrap();
        if let Some(length) = file_size {
            println!("文件大小：{}", HumanBytes(length));
            let bar = create_bar(length);
            bar.set_message("downloading");
            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let mut chunk = chunk?;
                let chunk_length = chunk.len() as u64;
                file.write_all_buf(&mut chunk).await?;
                bar.inc(chunk_length);
            }
            bar.finish_with_message("downloaded");
        } else {
            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let mut chunk = chunk?;
                file.write_all_buf(&mut chunk).await?;
            }
        }

        Ok(())
    }

    fn init(filesize: Option<u64>) -> Result<()> {
        if let Some(path) = ARGS.save_path().parent() {
            create_dir_all(path)?;
        }

        if ARGS.save_path().is_file() {
            return Ok(());
        }

        if let Some(filesize) = filesize {
            let file = std::fs::File::create(ARGS.save_path())?;
            file.set_len(filesize)?;
        }
        Ok(())
    }

    pub async fn run() -> Result<()> {
        let now = Instant::now();

        let (is_resumed, file_size) = Self::get_file_size().await?;

        Self::init(file_size)?;
        let mut dn_status: DnStatus = DnStatus::load_or_create();

        if !dn_status.downloaded() {
            dn_status.info.file_size = file_size;

            if !is_resumed || file_size.is_none() {
                println!("此文件不支持多线程下载");
                Self::download_one(file_size).await?;
            } else {
                let file_size = file_size.unwrap();
                println!("文件大小：{}", HumanBytes(file_size));

                let block_size = file_size / ARGS.thread_count as u64;

                let mut handles = Vec::with_capacity(ARGS.thread_count + 1);
                for i in 0..ARGS.thread_count {
                    let start = if i == 0 { 0 } else { i as u64 * block_size + 1 };
                    let end = if i == ARGS.thread_count - 1 {
                        file_size - 1
                    } else {
                        (i + 1) as u64 * block_size
                    };
                    let size = end - start + 1;

                    let bid = format!("dn{i:02}");
                    let block = Block::new(bid.clone(), start, end, size, ARGS.retry);
                    dn_status.blocks.insert(bid, block.clone());
                }

                dn_status.save().unwrap();
                dn_status.set_block_status_diff().unwrap();

                for (_, block) in dn_status.blocks {
                    handles.push(tokio::spawn(async move { block.clone().download().await }))
                }

                join_all(handles).await;
                M.clear().unwrap();
            }
            println!("下载完成! 总耗时: {:.2?}", now.elapsed());
        } else {
            println!(
                "继续下载，上次下载进度: {} / {}",
                HumanBytes(dn_status.info.down_size),
                HumanBytes(dn_status.info.file_size.unwrap())
            );
            let mut handles = Vec::with_capacity(ARGS.thread_count);

            dn_status.set_block_status_diff().unwrap();

            for (_, mut block) in dn_status.blocks {
                handles.push(tokio::spawn(async move { block.download().await }))
            }

            join_all(handles).await;
            M.clear().unwrap();
            println!("下载完成! 总耗时: {:.2?}", now.elapsed());
        }

        std::fs::remove_file(ARGS.status_file()).unwrap();

        Ok(())
    }
}

// url = https://www.voidtools.com/Everything-1.4.1.1024.x64.Lite-Setup.exe
