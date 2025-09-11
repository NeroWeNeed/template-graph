use crate::model::node::ModelNodeId;

pub trait EnvNodeIdStorage {
    fn set_env_node_id(&mut self, value: ModelNodeId);
    fn get_env_node_id(&self) -> Option<ModelNodeId>;
}
pub trait EnvNodeIdProvider {
    type Error;
    fn get_env_node_id(&mut self) -> Result<ModelNodeId, Self::Error>;
}
