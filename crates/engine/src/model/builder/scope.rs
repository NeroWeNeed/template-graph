use std::{borrow::Cow, collections::BTreeMap, marker::PhantomData};

use template_engine_commons::{
    allocator::{EdgeAllocator, ModelProvider, NodeAllocator},
    model::{Model, ModelNode, ModelNodeId},
};

use crate::model::builder::{ModelBuilder, allocate_node};

pub struct ModelBuilderScope<'a, H, Inner> {
    inner: Inner,
    references: BTreeMap<Cow<'a, str>, ModelNodeId>,
    dependencies: Vec<ModelNodeId>,
    header: PhantomData<H>,
}
impl<'a, H> ModelBuilder<'a, H> {
    pub fn scoped(self) -> ModelBuilderScope<'a, H, Self> {
        ModelBuilderScope {
            references: self.references.clone().into_iter().collect(),
            inner: self,
            dependencies: Vec::new(),
            header: PhantomData,
        }
    }
}
impl<'a, H, Inner> NodeAllocator for ModelBuilderScope<'a, H, Inner>
where
    Inner: ModelProvider<'a, H> + EdgeAllocator<Id = ModelNodeId>,
    <Inner as EdgeAllocator>::Error: Into<crate::Error>,
{
    type Error = crate::Error;

    type Node = ModelNode<'a>;

    type Id = ModelNodeId;

    fn allocate_node(&mut self, node: Self::Node) -> Result<Self::Id, Self::Error> {
        let id = allocate_node(self.inner.model_mut(), node, &mut self.references)?;
        for dependency in &self.dependencies {
            self.inner
                .allocate_edge(*dependency, id)
                .map_err(|err| err.into())?;
        }

        Ok(id)
    }
}
impl<'a, H, Inner> EdgeAllocator for ModelBuilderScope<'a, H, Inner>
where
    Inner: EdgeAllocator,
{
    type Error = Inner::Error;

    type Id = Inner::Id;

    fn allocate_edge(&mut self, from: Self::Id, to: Self::Id) -> Result<(), Self::Error> {
        self.inner.allocate_edge(from, to)
    }
}
impl<'a, H, Inner> ModelProvider<'a, H> for ModelBuilderScope<'a, H, Inner>
where
    Inner: ModelProvider<'a, H>,
{
    fn model(&self) -> &Model<'a, H> {
        self.inner.model()
    }

    fn model_mut(&mut self) -> &mut Model<'a, H> {
        self.inner.model_mut()
    }
}
