use std::path::Path;

use clap::Parser;

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
}

impl Args {
    pub fn get(&self) -> (String, String) {
        let output = match self.output.to_owned() {
            Some(output) => output,
            None => self.url.split('/').last().unwrap().to_string(),
        };

        if Path::new(&output).exists() {
            println!("file `{}` has exists", output);
        }

        let temp_file = output.split('.').next().unwrap();
        let temp_file = format!("{temp_file}.dn");
        (output, temp_file)
    }
}
