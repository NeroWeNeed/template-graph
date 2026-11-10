use std::path::{Path, PathBuf};

use crate::{error::Error, watcher::PathEvent};
use template_engine_commons::{
    format::Formatter, operation::Operation, source::SourceHandler, value::Value,
};

use super::watcher::PathEventKind;
#[derive(Debug)]
pub struct PathSource<F, Id> {
    pub data: PathSourceData<F, Id>,
    pub kind: PathSourceKind,
}
#[derive(Debug)]
pub struct PathSourceData<F, Id> {
    pub value: PathBuf,
    pub format: F,
    pub receiver: tokio::sync::broadcast::Receiver<PathEvent>,
    pub id: Id,
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
) -> Result<Option<Value>, Error>
where
    Error: From<E>,
{
    match kind {
        PathEventKind::DataModified | PathEventKind::Created => {
            Ok(Some(formatter.format(&std::fs::read_to_string(path)?)?))
        }
        PathEventKind::Removed => Ok(Some(Value::Null)),
        _ => Ok(None),
    }
}
impl<F, Id> PathSourceData<F, Id>
where
    F: Formatter,
    Error: From<F::Error>,
{
    pub(crate) async fn listen_file<S>(self, sender: S) -> Result<(), Error>
    where
        S: SourceHandler<Id, Operation>,
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
                                    if let Err(err) = sender
                                        .send(
                                            id,
                                            Operation::Set(Value::Object(vec![(
                                                name.clone(),
                                                value,
                                            )])),
                                        )
                                        .await
                                    {
                                        break;
                                    }
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
    pub(crate) async fn listen_dir<S>(self, sender: S) -> Result<(), Error>
    where
        S: SourceHandler<Id, Operation>,
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
                                    if let Err(err) = sender.send(id, Operation::Merge(value)).await
                                    {
                                        break;
                                    }
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
