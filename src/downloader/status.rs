use super::block::Block;
use crate::{cli::args::Args, downloader::parse::DownloadStatus};
use anyhow::Result;
use clap::Parser;
use indicatif::MultiProgress;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

pub static ARGS: Lazy<Args> = Lazy::new(|| {
    let args = Args::parse();
    args.check_exists()
});
pub static TEMP_FILE: Lazy<PathBuf> = Lazy::new(|| ARGS.status_file());
pub static M: Lazy<MultiProgress> = Lazy::new(MultiProgress::new);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnInfo {
    pub url: String,
    pub save_path: String,
    pub thread_count: usize,
    pub retry: u8,
    pub file_size: Option<u64>,
    pub down_size: u64,
}

impl DnInfo {
    pub fn new() -> Self {
        Self {
            url: ARGS.url.clone(),
            save_path: ARGS.save_path().to_str().unwrap().to_string(),
            thread_count: ARGS.thread_count,
            retry: ARGS.retry,
            file_size: None,
            down_size: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnStatus {
    pub info: DnInfo,
    pub blocks: HashMap<String, Block>,
    #[serde(default)]
    pub block_json_len: u64,
}

impl Default for DnStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl DnStatus {
    fn new() -> Self {
        Self {
            info: DnInfo::new(),
            blocks: HashMap::new(),
            block_json_len: 0,
        }
    }

    pub fn load_or_create() -> Self {
        let mut dn_status = Self::new();
        let json_path = TEMP_FILE.to_path_buf();
        if !json_path.exists() {
            dn_status
        } else {
            dn_status.load().unwrap();
            dn_status
        }
    }

    fn load(&mut self) -> Result<()> {
        self.info.down_size = 0;

        let json_path = TEMP_FILE.to_path_buf();
        if json_path.exists() {
            let data = fs::read(json_path)?;
            let mut cursor = 0;

            // Read JSON length (u64)
            let json_len = u64::from_le_bytes(data[cursor..cursor + 8].try_into()?) as usize;
            cursor += 8;

            self.block_json_len = json_len as u64;

            // Read JSON string
            let json_str = String::from_utf8(data[cursor..cursor + json_len].to_vec())?;
            cursor += json_len;

            // Deserialize JSON
            let status: Self = serde_json::from_str(&json_str)?;
            self.info = status.info;
            self.blocks = status.blocks;

            let mut blocks: Vec<_> = self.blocks.iter_mut().collect();
            blocks.sort_by(|a, b| a.0.cmp(b.0));

            // Read block progress data
            for (_, block) in blocks {
                let progress = u64::from_le_bytes(data[cursor..cursor + 8].try_into()?);
                cursor += 8;
                // println!("{}: {}, expected: {}", block.id, progress, block.size);

                assert!(progress <= block.size);

                self.info.down_size += progress;

                if progress == block.size {
                    block.status = DownloadStatus::Completed;
                } else {
                    block.status = DownloadStatus::Progress(progress);
                }
            }
        }
        Ok(())
    }

    pub fn set_block_status_diff(&mut self) -> Result<()> {
        assert!(self.block_json_len > 0);

        let mut blocks: Vec<_> = self.blocks.iter_mut().collect();
        blocks.sort_by(|a, b| a.0.cmp(b.0));

        for (index, (_, block)) in blocks.into_iter().enumerate() {
            block.status_diff = Some(self.block_json_len + 8 + index as u64 * 8);
        }
        Ok(())
    }

    pub fn save(&mut self) -> Result<()> {
        // This method should only be called once for each download.
        // Don't call when resume.

        let json_path = TEMP_FILE.to_path_buf();
        let json_str = serde_json::to_string(&self)?;
        let json_len = json_str.len() as u64;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(json_path)
            .unwrap();
        file.write_all(&json_len.to_le_bytes())?;
        file.write_all(json_str.as_bytes())?;
        file.set_len(json_len + 8 + self.blocks.len() as u64 * 8)?;

        self.block_json_len = json_str.len() as u64;

        Ok(())
    }

    pub fn downloaded(&self) -> bool {
        self.info.down_size != 0
    }
}
