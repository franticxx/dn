use crate::cli::Args;
use crate::utils::Block;
use anyhow::Result;
use clap::Parser;
use futures::{future::join_all, StreamExt};
use once_cell::sync::Lazy;
use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, RANGE};
use reqwest::{Client, Response};
use std::time::Instant;
use std::{io::SeekFrom, process, sync::Arc};
use tokio::fs::{rename, File};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex;

pub static ARGS: Lazy<Args> = Lazy::new(|| Args::parse());
pub static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());
// pub static COUNT: Lazy<> = Lazy::new(|| u);

/// get content length
pub async fn get_file_size(url: &str) -> Result<u64> {
    let res = CLIENT.head(url).send().await?;
    let headers = res.headers();
    if headers.contains_key(ACCEPT_RANGES) {
        if let Some(length) = headers.get(CONTENT_LENGTH) {
            let length = length.to_str()?;
            let length = length.parse::<u64>()?;
            return Ok(length);
        }
    }
    println!("此文件不支持下载...");
    process::exit(0)
}

/// download block
pub async fn download_block(
    block: Block,
    file: Arc<Mutex<File>>,
    progress: Arc<Mutex<Vec<f64>>>,
) -> Result<()> {
    let response = CLIENT
        .get(&ARGS.url)
        .header(RANGE, format!("bytes={}-{}", block.start, block.end))
        .send()
        .await?;

    write_block(response, &block, file, progress).await?;

    Ok(())
}

/// write block
pub async fn write_block(
    response: Response,
    block: &Block,
    file: Arc<Mutex<File>>,
    progress: Arc<Mutex<Vec<f64>>>,
) -> Result<()> {
    let mut stream = response.bytes_stream();
    let mut download_size = 0;

    while let Some(chunk) = stream.next().await {
        let mut chunk = match chunk {
            Ok(chunk) => chunk,
            Err(e) => {
                println!("failed: {:?}", e);
                process::exit(0);
            }
        };
        let mut file = file.lock().await;

        file.seek(SeekFrom::Start(block.start + download_size))
            .await?;

        let chunk_length = chunk.len() as u64;
        download_size += chunk_length;

        file.write_all_buf(&mut chunk).await?;
        block.bar.inc(chunk_length);

        let mut progress = progress.lock().await;
        progress[block.id] = (download_size as f64) / (block.size as f64);

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    println!("{:?}", block);
    block.bar.finish();
    Ok(())
}

pub async fn run() -> Result<()> {
    let now = Instant::now();
    let (output, temp_file) = ARGS.get();

    let content_length = get_file_size(&ARGS.url).await?;
    println!("{}", content_length);
    let block_size = content_length / ARGS.thread_count as u64;

    let mut handles = Vec::with_capacity(ARGS.thread_count);

    let file = Arc::new(Mutex::new(File::create(&temp_file).await?));
    let progress = Arc::new(Mutex::new(vec![0.0; ARGS.thread_count]));

    // let (tx, _) = broadcast::channel(8);

    for i in 0..ARGS.thread_count {
        let start = i as u64 * block_size;
        let end = if i == ARGS.thread_count - 1 {
            content_length
        } else {
            (i + 1) as u64 * block_size
        };
        let size = end - start;

        let block = Block::new(i, start, end, size);

        handles.push(tokio::spawn(download_block(
            block,
            file.clone(),
            progress.clone(),
        )));
    }

    join_all(handles).await;
    crate::utils::M.clear().unwrap();

    rename(temp_file, output).await?;

    println!("Completed!");
    println!("Cost: {:?}", now.elapsed());

    Ok(())
}

// url = https://www.voidtools.com/Everything-1.4.1.1024.x64.Lite-Setup.exe
