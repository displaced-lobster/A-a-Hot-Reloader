use std::{io, fs, time};
use walkdir::WalkDir;

pub enum WatcherType {
    File,
    Directory,
}

pub struct Watcher {
    watcher_type: WatcherType,
    target: String,
    last_modified: time::SystemTime,
}

impl Watcher {
    pub fn file_watcher(file: &str) -> Watcher {
        Watcher {
            watcher_type: WatcherType::File,
            target: String::from(file),
            last_modified: time::SystemTime::now(),
        }
    }

    pub fn dir_watcher(path: &str) -> Watcher {
        Watcher {
            watcher_type: WatcherType::Directory,
            target: String::from(path),
            last_modified: time::SystemTime::now(),
        }
    }

    pub fn was_modified(&mut self) -> Result<bool, io::Error> {
        match &self.watcher_type {
            WatcherType::File => {
                let check_modified = get_modified_file(&self.target)?;

                if check_modified > self.last_modified {
                    self.last_modified = check_modified;
                    return Ok(true);
                }

                Ok(false)
            },
            WatcherType::Directory => {
                if dir_been_modified(&self.target, &self.last_modified)? {
                    self.last_modified = time::SystemTime::now();
                    return Ok(true);
                }

                Ok(false)
            }
        }
    }
}

fn get_modified_file(target: &str) -> Result<time::SystemTime, io::Error> {
    let modified = fs::metadata(target)?.modified()?;

    Ok(modified)
}

fn dir_been_modified(path: &str, time: &time::SystemTime) -> Result<bool, io::Error> {
    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok()) {
            let file = entry.path().to_str();

            if let Some(file) = file {
                let file_modified = get_modified_file(&file)?;

                if file_modified > *time {
                    return Ok(true);
                }
            }
        }

    Ok(false)
}
