pub fn init_model() {}

pub struct ModelExtra {
    file_watcher: BroadcastFileWatcher,
    env_node_id: Option<ModelNodeId>,
}
impl ModelExtra {
    pub fn new(config: notify::Config, buffer_size: usize) -> Result<Self, crate::Error> {
        let file_watcher = BroadcastFileWatcher::new(config, buffer_size)?;
        Ok(Self {
            file_watcher,
            env_node_id: None,
        })
    }
}
impl FileWatcher for ModelExtra {
    fn watcher_mut(&mut self) -> &mut BroadcastFileWatcher {
        &mut self.file_watcher
    }

    fn watcher(&self) -> &BroadcastFileWatcher {
        &self.file_watcher
    }
}

impl EnvNodeIdStorage for ModelExtra {
    fn set_env_node_id(&mut self, value: ModelNodeId) {
        self.env_node_id = Some(value);
    }

    fn get_env_node_id(&self) -> Option<ModelNodeId> {
        self.env_node_id
    }
}
