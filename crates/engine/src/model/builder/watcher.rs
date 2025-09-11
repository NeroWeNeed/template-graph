use crate::{
    model::builder::ModelBuilder,
    source::fs::watcher::{BroadcastFileWatcher, FileWatcher},
};

impl<'a, H> FileWatcher for ModelBuilder<'a, H>
where
    H: FileWatcher,
{
    fn watcher_mut(&mut self) -> &mut BroadcastFileWatcher {
        self.model.header.watcher_mut()
    }

    fn watcher(&self) -> &BroadcastFileWatcher {
        self.model.header.watcher()
    }
}
