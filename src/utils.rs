use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use once_cell::sync::Lazy;

pub static M: Lazy<MultiProgress> = Lazy::new(|| MultiProgress::new());

#[derive(Debug)]
pub struct Block {
    pub id: usize,
    pub start: u64,
    pub end: u64,
    pub size: u64,
    pub bar: ProgressBar,
}

impl Block {
    pub fn new(id: usize, start: u64, end: u64, size: u64) -> Self {
        let style = ProgressStyle::with_template(
            "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )
        .unwrap()
        .progress_chars("##-");
        let bar = M.add(ProgressBar::new(size));
        bar.set_style(style);

        Self {
            id,
            start,
            end,
            size,
            bar,
        }
    }
}
