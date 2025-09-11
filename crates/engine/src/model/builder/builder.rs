use std::{borrow::Cow, collections::BTreeMap};

use crate::{
    Error,
    model::{
        Model, ModelNodeEdge,
        node::{ModelNode, ModelNodeId},
    },
    source::Operation,
};

#[derive(Debug)]
pub struct ModelBuilder<'a, H> {
    pub(super) model: Model<'a, H>,
    references: BTreeMap<Cow<'a, str>, ModelNodeId>,
}
impl<'a, H> ModelBuilder<'a, H> {
    pub fn new<const N: usize>(
        header: H,
    ) -> (Self, tokio::sync::mpsc::Sender<(ModelNodeId, Operation)>) {
        let (tx, rx) = tokio::sync::mpsc::channel(N);
        (
            Self {
                model: Model {
                    nodes: Vec::default(),
                    edges: Vec::default(),
                    event_receiver: rx,
                    event_sender: tx.clone(),
                    header,
                },
                references: BTreeMap::default(),
            },
            tx,
        )
    }
    pub fn allocate_node(
        &mut self,
        node: ModelNode<'a>,
        overwrite: bool,
    ) -> Result<ModelNodeId, crate::Error> {
        let index = self.model.nodes.len();
        if let Some(name) = &node.name {
            if !name.is_empty() {
                if self.references.insert(name.clone(), index).is_some() && !overwrite {
                    return Err(Error::DuplicateIdentifier(name.clone().into()));
                }
            }
        }
        self.model.nodes.push(node);
        Ok(index)
    }
    pub fn allocate_edge(&mut self, from: ModelNodeId, to: ModelNodeId) {
        self.model.edges.push(ModelNodeEdge { from, to });
    }

    pub fn update<F>(&mut self, id: ModelNodeId, op: F) -> Result<(), crate::Error>
    where
        F: FnOnce(&mut ModelNode) -> Result<(), crate::Error>,
    {
        if let Some(node) = self.model.nodes.get_mut(id) {
            op(node)
        } else {
            Err(crate::Error::InvalidNode)
        }
    }
    pub fn get_node_id(&self, name: impl Into<Cow<'a, str>>) -> Option<ModelNodeId> {
        self.references.get(&name.into()).cloned()
    }

    pub async fn start(self) {}

    pub fn header(&self) -> &H {
        &self.model.header
    }

    pub fn header_mut(&mut self) -> &mut H {
        &mut self.model.header
    }
    pub fn build(self) -> Model<'a, H> {
        self.model
    }
}
