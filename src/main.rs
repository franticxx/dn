mod cli;
mod download;
mod parse;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    download::run().await?;

    Ok(())
}
