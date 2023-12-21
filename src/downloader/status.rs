use super::{block::Block, parse::DownloadStatus};
use crate::cli::cli::Args;
use anyhow::Result;
use clap::Parser;
use indicatif::MultiProgress;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::{collections::HashMap, env::temp_dir, path::PathBuf};

pub static ARGS: Lazy<Args> = Lazy::new(|| {
    let args = Args::parse();
    args.check_exists()
});
pub static DB: Lazy<Db> = Lazy::new(|| sled::open(TEMP_DIR.join("status")).unwrap());
pub static TEMP_DIR: Lazy<PathBuf> = Lazy::new(|| temp_dir().join(ARGS.filename()));
pub static M: Lazy<MultiProgress> = Lazy::new(|| MultiProgress::new());

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
}

impl DnStatus {
    fn new() -> Self {
        Self {
            info: DnInfo::new(),
            blocks: HashMap::new(),
        }
    }

    pub fn load_or_create() -> Self {
        let mut dn_status = Self::new();
        if DB.is_empty() {
            dn_status
        } else {
            dn_status.load().unwrap();
            dn_status
        }
    }

    fn load(&mut self) -> Result<()> {
        self.info = serde_json::from_slice(&DB.get("info")?.unwrap())?;
        self.blocks = serde_json::from_slice(&DB.get("blocks")?.unwrap())?;

        let paths = std::fs::read_dir(TEMP_DIR.as_path())?
            .filter_map(|i| i.ok())
            .filter(|i| i.file_name().to_str().unwrap().contains(".dn"))
            .collect::<Vec<_>>();

        for p in paths.iter() {
            let block_size = std::fs::metadata(&p.path())?.len();
            let bid = format!("{}", p.path().extension().unwrap().to_str().unwrap());

            if let Some(block) = self.blocks.get_mut(&bid) {
                block.status = if block_size >= block.size {
                    DownloadStatus::Completed
                } else {
                    DownloadStatus::Progress(block_size)
                }
            }

            self.info.down_size += block_size;
        }
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        DB.insert("info", serde_json::to_string(&self.info)?.as_bytes())?;
        DB.insert("blocks", serde_json::to_string(&self.blocks)?.as_bytes())?;
        Ok(())
    }

    pub fn downloaded(&self) -> bool {
        self.info.down_size != 0
    }
}

#[test]
fn tt() {
    let db = sled::open(
        temp_dir()
            .join("Everything-1.4.1.1024.x64.Lite-Setup.exe")
            .join("status"),
    )
    .unwrap();

    let info: DnInfo = serde_json::from_slice(&db.get("info").unwrap().unwrap()).unwrap();
    let blocks: HashMap<String, Block> =
        serde_json::from_slice(&db.get("blocks").unwrap().unwrap()).unwrap();

    println!("{:#?}", info);

    for (bid, block) in blocks {
        println!("{bid}: {:?}", block);
    }
}
