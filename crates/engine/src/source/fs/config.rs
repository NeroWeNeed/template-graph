use std::path::Path;

use notify::RecursiveMode;
use serde::{Deserialize, Serialize};

use crate::{
    model::node::ModelNodeId,
    source::{
        SourceProvider,
        format::Format,
        fs::{
            source::{PathSource, PathSourceData, PathSourceKind},
            watcher::FileWatcher,
        },
    },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathSourceProvider {
    pub value: String,
    pub format: Format,
    pub recursive: Option<bool>,
}

impl<'a, T> SourceProvider<T> for PathSourceProvider
where
    T: FileWatcher,
{
    type Error = crate::Error;

    type Source = PathSource<Format>;

    fn new(
        self,
        name: impl AsRef<str>,
        data: &mut T,
        id: ModelNodeId,
    ) -> Result<Self::Source, Self::Error> {
        let watcher = data.watcher_mut();
        let (path, receiver, kind) = if self.value.ends_with("/") {
            let path = Path::new(&self.value).to_path_buf();
            let receiver = watcher.subscribe(
                &path,
                self.recursive.map_or_else(
                    || notify::RecursiveMode::NonRecursive,
                    |value| {
                        if value {
                            RecursiveMode::Recursive
                        } else {
                            RecursiveMode::NonRecursive
                        }
                    },
                ),
            )?;
            (path, receiver, PathSourceKind::Dir)
        } else {
            let path = Path::new(&self.value).to_path_buf();
            let receiver = watcher.subscribe(&path, RecursiveMode::NonRecursive)?;
            (path, receiver, PathSourceKind::File)
        };
        Ok(PathSource {
            data: PathSourceData {
                value: path,
                format: self.format,
                receiver: receiver,
                id,
                name: name.as_ref().to_string(),
            },
            kind,
        })
    }
}
