use anyhow::Result;

pub struct Synchronizer;

impl Synchronizer {
    pub fn new() -> Self {
        Self
    }

    pub fn sync_all(&self, dry_run: bool) -> Result<()> {
        if dry_run {
            println!("Dry run: would sync all rules across tools");
        } else {
            println!("Syncing all rules across tools (not yet implemented)");
        }
        Ok(())
    }
}

impl Default for Synchronizer {
    fn default() -> Self {
        Self::new()
    }
} 