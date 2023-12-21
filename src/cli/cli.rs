use clap::Parser;
use std::{io, path::Path, process};

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

    /// Thread num
    #[arg(short, long, default_value_t = 8)]
    pub thread_count: usize,

    /// Retry times
    #[arg(short, long, default_value_t = 3)]
    pub retry: u8,
}

impl Args {
    pub fn save_path(&self) -> &Path {
        let output = match &self.output {
            Some(output) => output,
            None => self.filename(),
        };
        Path::new(output)
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
