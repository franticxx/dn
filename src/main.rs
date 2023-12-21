use downloader::download::DownloadManager;

mod cli;
mod downloader;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    DownloadManager::run().await?;
    Ok(())
}
