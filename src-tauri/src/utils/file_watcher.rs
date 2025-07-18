use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event};
use std::path::Path;
use anyhow::Result;

pub struct FileWatcher {
    watcher: Option<RecommendedWatcher>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            watcher: None,
        }
    }
    
    pub fn start_watching<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let mut watcher = notify::recommended_watcher(|res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    // Handle file system events
                    println!("File changed: {:?}", event);
                }
                Err(e) => println!("Watch error: {:?}", e),
            }
        })?;
        
        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
        self.watcher = Some(watcher);
        
        Ok(())
    }
    
    pub fn stop_watching(&mut self) {
        if let Some(watcher) = self.watcher.take() {
            drop(watcher);
        }
    }
}