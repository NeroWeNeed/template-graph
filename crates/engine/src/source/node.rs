use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeSourceProvider {
    pub name: String,
}
