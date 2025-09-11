use crate::{
    model::node::{ModelNode, ModelNodeId},
    source::{
        Operation,
        env::EnvNodeIdStorage,
        fs::watcher::{BroadcastFileWatcher, FileWatcher},
    },
};

/// Represents an edge in the [Model]. Edges follow the pattern of from -> to, where from
/// represents something that to has a dependency on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelNodeEdge {
    pub from: ModelNodeId,
    pub to: ModelNodeId,
}

#[derive(Debug)]
pub struct Model<'a, H> {
    pub(crate) nodes: Vec<ModelNode<'a>>,
    pub(crate) edges: Vec<ModelNodeEdge>,
    pub(crate) event_receiver: tokio::sync::mpsc::Receiver<(ModelNodeId, Operation)>,
    pub(crate) event_sender: tokio::sync::mpsc::Sender<(ModelNodeId, Operation)>,
    pub(crate) header: H,
}

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
