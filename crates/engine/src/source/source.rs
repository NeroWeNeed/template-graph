use crate::{model::node::ModelNodeId, value::Value};

pub trait SourceProvider<D> {
    type Error;
    type Source;
    fn new(
        self,
        name: impl AsRef<str>,
        data: &mut D,
        id: ModelNodeId,
    ) -> Result<Self::Source, Self::Error>;
}
pub trait Source {
    type Error;
    fn listen<S>(self, sender: S) -> impl std::future::Future<Output = Result<(), Self::Error>>
    where
        S: SourceHandler;
}
pub trait SourceHandler {
    type Error;
    fn send(
        &self,
        id: ModelNodeId,
        event: Operation,
    ) -> impl Future<Output = Result<(), Self::Error>>;
}
impl SourceHandler for tokio::sync::mpsc::Sender<(ModelNodeId, Operation)> {
    type Error = tokio::sync::mpsc::error::SendError<(ModelNodeId, Operation)>;

    async fn send(&self, id: ModelNodeId, event: Operation) -> Result<(), Self::Error> {
        self.send((id, event)).await
    }
}
#[derive(Debug, Clone)]
pub enum Operation {
    Set(Value),
    Merge(Value),
    Remove(String),
    Reload,
}
