use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct AuditWriter {
    file: File,
}

impl AuditWriter {
    pub fn open(log_dir: &PathBuf) -> Result<Self, std::io::Error> {
        std::fs::create_dir_all(log_dir)?;
        let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
        let path = log_dir.join(format!("session-{timestamp}.log"));
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self { file })
    }

    pub fn write_event(&mut self, event: &str) {
        let _ = writeln!(self.file, "{event}");
        let _ = self.file.flush();
    }
}
