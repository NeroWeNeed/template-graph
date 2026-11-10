use std::path::{Path, PathBuf};

use notify::{Config, Error, Event, RecommendedWatcher, RecursiveMode, Watcher};
use template_engine_commons::value::Value;
use tokio::sync::mpsc::{Sender, error::SendError};

pub struct BroadcastFileWatcher {
    watcher: RecommendedWatcher,
    sender: tokio::sync::broadcast::Sender<PathEvent>,
}
pub trait FileWatcher {
    fn watcher_mut(&mut self) -> &mut BroadcastFileWatcher;
    fn watcher(&self) -> &BroadcastFileWatcher;
}
impl FileWatcher for BroadcastFileWatcher {
    fn watcher_mut(&mut self) -> &mut BroadcastFileWatcher {
        self
    }

    fn watcher(&self) -> &BroadcastFileWatcher {
        self
    }
}
impl BroadcastFileWatcher {
    pub fn new(config: Config, buffer_size: usize) -> Result<Self, notify::Error> {
        let (tx, _) = tokio::sync::broadcast::channel(buffer_size);
        let watcher_tx = tx.clone();
        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, Error>| {
                futures::executor::block_on(async {
                    match res {
                        Ok(evt) => match &evt.kind {
                            notify::EventKind::Create(kind) => match kind {
                                notify::event::CreateKind::File => {
                                    for path in evt.paths {
                                        if let Err(err) = watcher_tx.send(PathEvent {
                                            value: path,
                                            kind: PathEventKind::Created,
                                        }) {
                                            break;
                                        }
                                    }
                                }
                                _ => {}
                            },
                            notify::EventKind::Modify(kind) => match kind {
                                notify::event::ModifyKind::Data(_) => {
                                    for path in evt.paths {
                                        if let Err(err) = watcher_tx.send(PathEvent {
                                            value: path,
                                            kind: PathEventKind::DataModified,
                                        }) {
                                            break;
                                        }
                                    }
                                }
                                notify::event::ModifyKind::Metadata(_) => {
                                    for path in evt.paths {
                                        if let Err(err) = watcher_tx.send(PathEvent {
                                            value: path,
                                            kind: PathEventKind::MetadataModified,
                                        }) {
                                            break;
                                        }
                                    }
                                }
                                notify::event::ModifyKind::Name(mode) => match mode {
                                    notify::event::RenameMode::From => {
                                        for path in evt.paths {
                                            if let Err(err) = watcher_tx.send(PathEvent {
                                                value: path,
                                                kind: PathEventKind::Removed,
                                            }) {
                                                break;
                                            }
                                        }
                                    }
                                    notify::event::RenameMode::To => {
                                        for path in evt.paths {
                                            if let Err(err) = watcher_tx.send(PathEvent {
                                                value: path,
                                                kind: PathEventKind::Created,
                                            }) {
                                                break;
                                            }
                                        }
                                    }
                                    _ => {}
                                },
                                _ => {}
                            },
                            notify::EventKind::Remove(kind) => match kind {
                                notify::event::RemoveKind::File => {
                                    for path in evt.paths {
                                        if let Err(err) = watcher_tx.send(PathEvent {
                                            value: path,
                                            kind: PathEventKind::Removed,
                                        }) {
                                            break;
                                        }
                                    }
                                }
                                _ => {}
                            },
                            _ => {}
                        },
                        Err(err) => {
                            error!("{err}");
                        }
                    };
                })
            },
            config,
        )?;

        Ok(Self {
            watcher,
            sender: tx,
        })
    }
    pub fn subscribe(
        &mut self,
        path: &Path,
        mode: RecursiveMode,
    ) -> Result<tokio::sync::broadcast::Receiver<PathEvent>, notify::Error> {
        self.watcher.watch(path, mode)?;
        Ok(self.sender.subscribe())
    }
}

#[derive(Debug, Clone)]
pub struct PathEvent {
    pub value: PathBuf,
    pub kind: PathEventKind,
}
#[derive(Debug, Clone, Copy)]
pub enum PathEventKind {
    Created,
    DataModified,
    MetadataModified,
    Removed,
}

#[derive(Debug, thiserror::Error)]
pub enum WatchError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
async fn handle_path<H, E>(
    channel: &Sender<Result<(PathBuf, Option<String>, Value), E>>,
    root: &Path,
    path: PathBuf,
    handler: H,
) -> Result<(), SendError<Result<(PathBuf, Option<String>, Value), E>>>
where
    H: FnOnce(&Path) -> Result<Value, E>,
{
    if path.is_file() {
        match &path.strip_prefix(root) {
            Ok(node) => match handler(&path) {
                Ok(mut value) => {
                    let mut key = None;
                    for n in node.iter().rev() {
                        let ck = Some(n.to_string_lossy().into());
                        let cv = match key {
                            Some(key) => Value::Object(vec![(key, value)].into()),
                            None => value,
                        };
                        key = ck;
                        value = cv;
                    }
                    channel.send(Ok((path, key, value))).await
                }
                Err(err) => channel.send(Err(err)).await,
            },
            _ => Ok(()),
        }
    } else {
        Ok(())
    }
}
