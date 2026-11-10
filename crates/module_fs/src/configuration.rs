use std::path::Path;

use notify::RecursiveMode;
use template_engine_commons::{
    allocator::{NodeAllocator, TaskAllocator},
    format::Format,
    model::ModelNode,
    operation::Operation,
    source::SourceHandler,
    value::Value,
};
use template_engine_config::model::Configuration;

use crate::{
    error::Error,
    source::{PathSource, PathSourceData, PathSourceKind},
    watcher::FileWatcher,
};

pub struct PathSourceConfiguration {
    pub value: String,
    pub format: Format,
    pub recursive: Option<bool>,
}

impl<'a, Data, Id> Configuration<Data, Id, Operation> for PathSourceConfiguration
where
    Data: NodeAllocator<Id = Id, Node = ModelNode<'a>> + FileWatcher + TaskAllocator,
    Error: From<<Data as TaskAllocator>::AllocationError> + From<<Data as NodeAllocator>::Error>,
{
    type Error = Error;
    fn configure<C>(
        self,
        name: impl AsRef<str>,
        data: &mut Data,
        sender: C,
    ) -> Result<(), Self::Error>
    where
        C: SourceHandler<Id, Operation>,
    {
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
        let id = data.allocate_node(ModelNode {
            name: Some(name.as_ref().into()),
            value: Value::Null,
            output: None,
            inherit: false,
        })?;
        let source = PathSource {
            data: PathSourceData {
                value: path,
                format: self.format,
                receiver: receiver,
                id,
                name: name.as_ref().to_string(),
            },
            kind,
        };

        data.allocate_task(async move {
            match source.kind {
                PathSourceKind::Dir => source.data.listen_dir(sender).await,
                PathSourceKind::File => source.data.listen_file(sender).await,
            };
            Ok(())
        });

        Ok(())
    }
}
