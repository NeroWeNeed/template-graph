use crate::model::Model;

pub trait NodeAllocator {
    type Error;
    type Node;
    type Id;
    fn allocate_node(&mut self, node: Self::Node) -> Result<Self::Id, Self::Error>;
}
pub trait EdgeAllocator {
    type Error;
    type Id;
    fn allocate_edge(&mut self, from: Self::Id, to: Self::Id) -> Result<(), Self::Error>;
}
pub trait ModelProvider<'a, H> {
    fn model(&self) -> &Model<'a, H>;
    fn model_mut(&mut self) -> &mut Model<'a, H>;
}
pub trait TaskAllocator {
    type TaskError;
    type AllocationError;
    fn allocate_task(
        &mut self,
        task: impl Future<Output = Result<(), Self::TaskError>>,
    ) -> Result<(), Self::AllocationError>;
}
