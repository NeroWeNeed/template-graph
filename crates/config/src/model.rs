use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use template_engine_commons::{model::Render, source::SourceHandler};
use toml::Value;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config<Source> {
    #[serde(skip)]
    pub(crate) source: Option<PathBuf>,
    pub(crate) generator: BTreeMap<String, Node<Source>>,
    pub(crate) variable: BTreeMap<String, Value>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Node<Source> {
    pub(crate) output: Vec<Render>,
    pub(crate) source: Vec<Source>,
}

pub trait Configuration<Data, Id, Op> {
    type Error;
    fn configure<C>(
        self,
        name: impl AsRef<str>,
        data: &mut Data,
        sender: C,
    ) -> Result<(), Self::Error>
    where
        C: SourceHandler<Id, Op>;
}

/* #[derive(Serialize, Deserialize, Clone)]
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
} */

/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerSource {
    host: IpAddr,
    port: u16,
} */
