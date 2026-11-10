use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    convert::Infallible,
};

use template_engine_commons::{
    allocator::{EdgeAllocator, ModelProvider, NodeAllocator},
    model::{Model, ModelNode, ModelNodeId},
    operation::Operation,
};

#[derive(Debug)]
pub struct ModelBuilder<'a, H> {
    pub(super) model: Model<'a, H>,
    pub(super) references: BTreeMap<Cow<'a, str>, ModelNodeId>,
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

pub(crate) fn allocate_node<'a, H>(
    model: &mut Model<'a, H>,
    node: ModelNode<'a>,
    references: &mut BTreeMap<Cow<'a, str>, ModelNodeId>,
) -> Result<ModelNodeId, crate::Error> {
    let index = model.nodes.len();
    if let Some(name) = &node.name {
        if !name.is_empty() {
            if references.insert(name.clone().into(), index).is_some() {
                return Err(crate::Error::DuplicateIdentifier(name.clone().into()));
            }
        }
    }
    model.nodes.push(node);
    Ok(index)
}
impl<'a, H> NodeAllocator for ModelBuilder<'a, H> {
    type Error = crate::Error;

    type Node = ModelNode<'a>;

    type Id = ModelNodeId;

    fn allocate_node(&mut self, node: Self::Node) -> Result<Self::Id, Self::Error> {
        allocate_node(&mut self.model, node, &mut self.references)
    }
}
impl<'a, H> EdgeAllocator for ModelBuilder<'a, H> {
    type Error = Infallible;

    type Id = ModelNodeId;

    fn allocate_edge(&mut self, from: Self::Id, to: Self::Id) -> Result<(), Self::Error> {
        self.model.edges.push(ModelNodeEdge { from, to });
        Ok(())
    }
}
impl<'a, H> ModelProvider<'a, H> for ModelBuilder<'a, H> {
    fn model(&self) -> &Model<'a, H> {
        &self.model
    }

    fn model_mut(&mut self) -> &mut Model<'a, H> {
        &mut self.model
    }
}
