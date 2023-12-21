#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DownloadStatus {
    Started,       // 下载开始
    Progress(u64), // 下载进度
    Completed,     // 下载完成
    Failed,        // 下载失败
}

impl DownloadStatus {
    pub fn update_status(self: &mut Self, progress: u64) {
        match self {
            DownloadStatus::Started => *self = DownloadStatus::Progress(progress),
            DownloadStatus::Progress(p) => *self = DownloadStatus::Progress(*p + progress),
            _ => (),
        }
    }
}
