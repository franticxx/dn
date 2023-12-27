use clap::Parser;
use std::{
    io,
    path::{Path, PathBuf},
    process,
};

#[derive(Debug, Parser)]
#[command(name = "dn")]
#[command(author = "Kshine")]
#[command(version = "0.0.1")]
#[command(about = "A file download tool", long_about = None)]
pub struct Args {
    /// url
    pub url: String,

    /// Output path
    #[arg(short)]
    pub output: Option<String>,

    /// Header file path, default loading: header, header.dn ,header.txt
    #[arg(short('H'), long)]
    pub header: Option<String>,

    /// User-Agent
    #[arg(short('A'), long)]
    pub user_agent: Option<String>,

    /// Thread num
    #[arg(short, long, default_value_t = 8)]
    pub thread_count: usize,

    /// Retry times
    #[arg(short, long, default_value_t = 3)]
    pub retry: u8,
}

impl Args {
    pub fn save_path(&self) -> PathBuf {
        match &self.output {
            Some(output) => {
                let output = Path::new(output);
                if output.is_dir() {
                    output.join(self.filename())
                } else {
                    output.to_path_buf()
                }
            }
            None => Path::new(self.filename()).to_path_buf(),
        }
    }

    pub fn filename(&self) -> &str {
        self.url
            .split('/')
            .last()
            .and_then(|i| i.split('?').next())
            .unwrap()
    }

    pub fn check_exists(self) -> Self {
        let path = self.save_path();
        if path.exists() {
            println!("文件已存在, 是否继续下载? y/n: ");
            let mut s = String::new();
            io::stdin().read_line(&mut s).unwrap();
            if !(s.trim().is_empty() || s.trim().eq_ignore_ascii_case("y")) {
                process::exit(0)
            }
        }
        self
    }
}
