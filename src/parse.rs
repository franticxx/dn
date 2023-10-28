#![allow(dead_code)]

#[derive(Debug, Clone)]
pub enum DownloadStatus {
    Started,       //下载已开始。
    Progress(f64), //下载进行中，携带已下载的进度值。
    Completed,     //下载已完成。
    Failed,        //下载失败。
}

impl DownloadStatus {
    pub fn is_finished(&self) -> bool {
        match self {
            DownloadStatus::Completed => true,
            _ => false,
        }
    }
    pub fn is_failed(&self) -> bool {
        match self {
            DownloadStatus::Failed => true,
            _ => false,
        }
    }
    pub fn update_status(self: &mut Self, progress: f64) {
        match self {
            DownloadStatus::Started => *self = DownloadStatus::Progress(progress),
            DownloadStatus::Progress(p) => *self = DownloadStatus::Progress(*p + progress),
            _ => (),
        }
    }
}
