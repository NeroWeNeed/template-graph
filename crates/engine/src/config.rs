use std::{
    collections::BTreeMap,
    net::IpAddr,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use toml::Value;

use crate::{
    render::{Output, Render},
    source::{fs::config::PathSourceProvider, node::NodeSourceProvider},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(skip)]
    pub(crate) source: Option<PathBuf>,
    pub(crate) generator: BTreeMap<String, Node>,
    pub(crate) variable: BTreeMap<String, Value>,
}

impl Config {
    pub fn load(
        path: impl AsRef<Path>,
        max_depth: usize,
        follow_symlinks: bool,
        buffer: &mut Vec<Config>,
    ) -> Result<(), ConfigError> {
        fn load(
            path: impl AsRef<Path>,
            max_depth: usize,
            current_depth: usize,
            follow_symlinks: bool,
            buffer: &mut Vec<Config>,
        ) -> Result<(), ConfigError> {
            let path = path.as_ref();
            if path.try_exists()? {
                if path.is_dir() {
                    if current_depth + 1 >= max_depth {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "maximum depth reached",
                        )
                        .into());
                    }
                    let current_depth = current_depth + 1;
                    for f in path.read_dir()? {
                        let path = f?.path();
                        if !path.is_symlink() || follow_symlinks {
                            load(path, max_depth, current_depth, follow_symlinks, buffer)?;
                        } else {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "symlinks not allowed",
                            )
                            .into());
                        }
                    }
                } else {
                    if !path.is_symlink() || follow_symlinks {
                        let data = std::fs::read_to_string(path)?;
                        let mut config: Config = toml::from_str(&data)?;
                        config.source = Some(path.to_owned());
                        buffer.push(config);
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "symlinks not allowed",
                        )
                        .into());
                    }
                }

                todo!()
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "missing config file or directory",
                )
                .into());
            }
        }
        load(path, max_depth, 0, follow_symlinks, buffer)
    }
}
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Node {
    pub(crate) output: Option<Render>,
    pub(crate) source: Vec<Source>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Path {
        #[serde(flatten)]
        source: PathSourceProvider,
    },
    /* Docker {
        #[serde(flatten)]
        source: DockerSourceProvider,
    }, */
    Server {
        #[serde(flatten)]
        source: ServerSource,
    },
    Env,
    Node {
        #[serde(flatten)]
        source: NodeSourceProvider,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerSource {
    host: IpAddr,
    port: u16,
}
