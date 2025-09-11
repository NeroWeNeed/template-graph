use std::path::{Path, PathBuf};

use log::error;

use crate::{
    model::node::ModelNodeId,
    source::{Operation, Source, format::Formatter, fs::watcher::PathEvent},
    value::Value,
};

use super::watcher::PathEventKind;
#[derive(Debug)]
pub struct PathSource<F> {
    pub data: PathSourceData<F>,
    pub kind: PathSourceKind,
}
#[derive(Debug)]
pub struct PathSourceData<F> {
    pub value: PathBuf,
    pub format: F,
    pub receiver: tokio::sync::broadcast::Receiver<PathEvent>,
    pub id: ModelNodeId,
    pub name: String,
}
#[derive(Debug)]
pub enum PathSourceKind {
    Dir,
    File,
}
fn handle<E>(
    path: &Path,
    formatter: &impl Formatter<Error = E>,
    kind: PathEventKind,
) -> Result<Option<Value>, crate::Error>
where
    crate::Error: From<E>,
{
    match kind {
        PathEventKind::DataModified | PathEventKind::Created => {
            Ok(Some(formatter.format(&std::fs::read_to_string(path)?)?))
        }
        PathEventKind::Removed => Ok(Some(Value::Null)),
        _ => Ok(None),
    }
}
impl<F> PathSourceData<F>
where
    F: Formatter<Error = crate::Error>,
{
    async fn listen_file<S>(self, sender: S) -> Result<(), crate::Error>
    where
        S: crate::source::SourceHandler,
    {
        let PathSourceData {
            value: target,
            format,
            mut receiver,
            id,
            name,
        } = self;

        loop {
            match receiver.recv().await {
                Ok(PathEvent { value, kind }) => {
                    if value == target {
                        match handle(&value, &format, kind) {
                            Ok(value) => {
                                if let Some(value) = value {
                                    sender
                                        .send(
                                            id,
                                            Operation::Set(Value::Object(vec![(
                                                name.clone(),
                                                value,
                                            )])),
                                        )
                                        .await
                                        .map_err(|_| crate::Error::SendError)?;
                                }
                            }
                            Err(err) => {
                                error!("{err}");
                            }
                        }
                    }
                }
                Err(err) => match err {
                    tokio::sync::broadcast::error::RecvError::Closed => {
                        break;
                    }
                    tokio::sync::broadcast::error::RecvError::Lagged(_) => {
                        continue;
                    }
                },
            }
        }
        Ok(())
    }
    async fn listen_dir<S>(self, sender: S) -> Result<(), crate::Error>
    where
        S: crate::source::SourceHandler,
    {
        let PathSourceData {
            value: parent,
            format,
            mut receiver,
            id,
            name,
        } = self;
        loop {
            match receiver.recv().await {
                Ok(PathEvent { value, kind }) => {
                    if let Some(node) = value.strip_prefix(&parent).ok() {
                        match handle(&value, &format, kind) {
                            Ok(value) => {
                                if let Some(value) = value {
                                    let value = Value::Object(vec![(
                                        name.clone(),
                                        node.iter().rev().fold(value, |acc, n| {
                                            Value::Object(vec![(n.to_string_lossy().into(), acc)])
                                        }),
                                    )]);
                                    sender
                                        .send(id, Operation::Merge(value))
                                        .await
                                        .map_err(|_| crate::Error::SendError)?;
                                }
                            }
                            Err(err) => {
                                error!("{err}");
                            }
                        }
                    }
                }
                Err(err) => match err {
                    tokio::sync::broadcast::error::RecvError::Closed => {
                        break;
                    }
                    tokio::sync::broadcast::error::RecvError::Lagged(_) => {
                        continue;
                    }
                },
            }
        }
        Ok(())
    }
}
impl<F> Source for PathSource<F>
where
    F: Formatter<Error = crate::Error>,
{
    type Error = crate::Error;

    async fn listen<S>(self, sender: S) -> Result<(), Self::Error>
    where
        S: crate::source::SourceHandler,
    {
        match self.kind {
            PathSourceKind::Dir => self.data.listen_dir(sender).await,
            PathSourceKind::File => self.data.listen_file(sender).await,
        }
    }
}
