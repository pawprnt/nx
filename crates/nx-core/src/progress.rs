use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct Spinner {
    pb: ProgressBar,
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::with_template("{spinner:.blue} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
        );
        pb.set_message(message.to_string());

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let pb_clone = pb.clone();

        let handle = thread::spawn(move || {
            while running_clone.load(Ordering::Relaxed) {
                pb_clone.tick();
                thread::sleep(Duration::from_millis(80));
            }
        });

        Self {
            pb,
            running,
            handle: Some(handle),
        }
    }

    pub fn finish_with_message(&self, msg: &str) {
        self.running.store(false, Ordering::Relaxed);
        self.pb.finish_with_message(msg.to_string());
    }

    pub fn fail_with_message(&self, msg: &str) {
        self.running.store(false, Ordering::Relaxed);
        self.pb.finish_with_message(msg.to_string());
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
