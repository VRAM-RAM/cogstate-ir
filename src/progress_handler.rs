use hf_hub::progress::{DownloadEvent, ProgressEvent, ProgressHandler};
use indicatif::{ProgressBar, ProgressStyle};

/// Wraps an `indicatif::ProgressBar` behind `hf_hub::progress::ProgressHandler`.
pub struct HfProgress {
    pb: ProgressBar,
}

impl HfProgress {
    pub fn new(label: String) -> Self {
        let pb = ProgressBar::new(0);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} [{bar:30}] {bytes}/{total_bytes} ({percent}%)")
                .unwrap()
                .progress_chars("=> "),
        );
        pb.set_message(label);
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Self { pb }
    }
}

impl ProgressHandler for HfProgress {
    fn on_progress(&self, event: &ProgressEvent) {
        match event {
            ProgressEvent::Download(DownloadEvent::Start { total_bytes, .. }) => {
                self.pb.set_length(*total_bytes);
                self.pb.reset();
            }
            ProgressEvent::Download(DownloadEvent::Progress { files }) => {
                let done: u64 = files.iter().map(|f| f.bytes_completed).sum();
                if done > self.pb.position() {
                    self.pb.set_position(done);
                }
            }
            ProgressEvent::Download(DownloadEvent::AggregateProgress {
                bytes_completed, ..
            }) => {
                self.pb.set_position(*bytes_completed);
            }
            ProgressEvent::Download(DownloadEvent::Complete) => {
                self.pb.finish_and_clear();
            }
            _ => {}
        }
    }
}
