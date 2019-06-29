extern crate inotify;

use std::{io};
use walkdir::{DirEntry, WalkDir};

use inotify::{
    EventMask,
    WatchMask,
    Inotify,
};

pub enum Traversal {
    RECURSIVE,
    HEURISTIC,
}

pub enum WatcherType {
    FILE,
    DIRECTORY,
}

pub struct Watcher {
    watcher_type: WatcherType,
    notify: Inotify,
}

impl Watcher {
    pub fn file_watcher(file: &str) -> Result<Watcher, io::Error> {
        let mut inotify = Inotify::init()?;

        inotify.add_watch(file, WatchMask::MODIFY | WatchMask::DELETE)?;

        Ok(Watcher {
            watcher_type: WatcherType::FILE,
            notify: inotify,
        })
    }

    pub fn dir_watcher(path: &str, trav: Traversal) -> Result<Watcher, io::Error> {
        let mut inotify = Inotify::init()?;
        let watch_mask = WatchMask::MODIFY |
                         WatchMask::CREATE |
                         WatchMask::DELETE;

        match trav {
            Traversal::RECURSIVE => {
                for entry in WalkDir::new(path)
                    .follow_links(true)
                    .into_iter()
                    .filter_entry(|e| !is_hidden(e) && e.file_type().is_dir()) {
                        let entry = entry?;
                        let path = entry.path();
                        //println!("{:?}", path.display());
                        inotify.add_watch(path, watch_mask)?;
                }
            },
            Traversal::HEURISTIC => {
                inotify.add_watch(path, watch_mask)?;
            }
        }

        Ok(Watcher {
            watcher_type: WatcherType::DIRECTORY,
            notify: inotify,
        })
    }

    pub fn watch(&mut self) -> Result<(bool), io::Error> {
        match &self.watcher_type {
            WatcherType::FILE => self.file_event_loop(),
            WatcherType::DIRECTORY => self.dir_event_loop(),
        }
    }

    fn dir_event_loop(&mut self) -> Result<(bool), io::Error> {
        let mut buffer = [0u8; 4096];

        loop {
            let events = self.notify.read_events_blocking(&mut buffer)?;

            for event in events {
                if event.mask.contains(EventMask::CREATE) {
                    if event.mask.contains(EventMask::ISDIR) {
                        println!("Directory created: {:?}", event.name);
                    } else {
                        println!("File created: {:?}", event.name);
                    }
                } else if event.mask.contains(EventMask::DELETE) {
                    if event.mask.contains(EventMask::ISDIR) {
                        println!("Directory deleted: {:?}", event.name);
                    } else {
                        println!("File deleted: {:?}", event.name);
                    }
                } else if event.mask.contains(EventMask::MODIFY) {
                    if event.mask.contains(EventMask::ISDIR) {
                        println!("Directory modified: {:?}", event.name);
                    } else {
                        println!("File modified: {:?}", event.name);
                    }
                }
                return Ok(true);
            }
        }
    }

    fn file_event_loop(&mut self) -> Result<(bool), io::Error> {
        let mut buffer = [0u8; 4096];

        loop {
            let events = self.notify.read_events_blocking(&mut buffer)?;

            for event in events {
                if event.mask.contains(EventMask::MODIFY) {
                    println!("File modified: {:?}", event.name);
                }
                return Ok(true);
            }
        }
    }
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}
