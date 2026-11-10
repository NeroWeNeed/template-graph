pub mod model;
pub mod render;
pub mod source;
pub mod template;

use std::{borrow::Cow, cell::RefCell, convert::Infallible};

use futures::TryFutureExt;
use log::error;

use crate::{
    model::{
        Model, ModelExtra, allocator::EdgeAllocator, allocator::NodeAllocator,
        builder::ModelBuilder, node::ModelNode,
    },
    source::{Operation, Source, SourceProvider, env::EnvNodeIdProvider},
    template::Renderer,
};

impl<'a> ModelBuilder<'a, ()> {
    pub async fn configure<const FSN: usize, const MN: usize>(
        configs: impl IntoIterator<Item = Config>,
    ) -> Result<(), Error> {
        let (mut model, mut tx) =
            ModelBuilder::new::<MN>(ModelExtra::new(notify::Config::default(), FSN)?);
        for config in configs.into_iter() {
            let Config {
                source,
                generator,
                variable,
            } = config;
            let mut global_dependencies = Vec::new();
            if !variable.is_empty() {
                global_dependencies.push(model.allocate_var_node(variable)?);
            }
            let mut sources = Vec::new();
            for (name, generator) in generator {
                let node_id = model.allocate_node(ModelNode {
                    name: Some(name.clone().into()),
                    value: value::Value::Null,
                    output: generator.output,
                    inherit: true,
                })?;
                for dependency in &global_dependencies {
                    model.allocate_edge(*dependency, node_id)?;
                }
                for source in generator.source {
                    sources.push((node_id, name.clone(), source));
                }
            }
            let mut tasks = Vec::new();
            for (id, name, source) in sources {
                let depends_on_id = match source {
                    config::Source::Path { source } => {
                        let target_id = model.allocate_node(ModelNode {
                            name: Some(name.clone().into()),
                            value: Value::Null,
                            output: None,
                            inherit: false,
                        })?;
                        tasks.push(
                            source
                                .new(name.clone(), model.header_mut(), target_id)?
                                .listen(tx.clone())
                                .map_err(|err| crate::Error::Source(err.into())),
                        );
                        target_id
                    }
                    //config::Source::Docker { source } => todo!(),
                    config::Source::Server { source } => todo!(),
                    config::Source::Env => model.get_env_node_id()?,
                    config::Source::Node { source } => {
                        let reference_id =
                            model.get_node_id(source.name).ok_or(Error::InvalidNode)?;
                        //TODO: Cycle check
                        reference_id
                    }
                };
                model.allocate_edge(depends_on_id, id)?;
            }
        }

        Ok(())
    }
    async fn serve<'b, H>(model: Model<'b, H>) -> Result<(), crate::Error> {
        let Model {
            nodes,
            mut edges,
            mut event_receiver,
            event_sender,
            header,
        } = model;
        let nodes = nodes
            .into_iter()
            .map(|node| RefCell::new(node))
            .collect::<Vec<_>>();
        edges.sort();
        let renderer = handlebars::Handlebars::new();
        while let Some((id, op)) = event_receiver.recv().await {
            let mut node = nodes.get(id).ok_or(Error::InvalidNode)?.borrow_mut();
            if node.apply(op) {
                if let Some(output) = &node.output {
                    let context: Cow<Value> =
                        if node.inherit {
                            let mut new_context = node.value.clone();
                            for dependency in edges.as_slice().into_iter().filter_map(|edge| {
                                if edge.from == id { Some(edge.to) } else { None }
                            }) {
                                if let Some(node) = nodes.get(dependency) {
                                    new_context = new_context.merge(&node.borrow().value);
                                } else {
                                    return Err(Error::InvalidNode);
                                }
                            }
                            Cow::Owned(new_context)
                        } else {
                            Cow::Borrowed(&node.value)
                        };
                    <handlebars::Handlebars as Renderer>::render(&renderer, &context, output)?;
                }

                for dependent in edges
                    .as_slice()
                    .into_iter()
                    .filter_map(|edge| if edge.from == id { Some(edge.to) } else { None })
                {
                    if let Err(_) = event_sender.send((dependent, Operation::Reload)).await {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Notify(#[from] notify::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("duplicate identifier {0}")]
    DuplicateIdentifier(String),
    #[error("invalid node referenced")]
    InvalidNode,
    #[error("unable to send data")]
    SendError,
    #[error("undefined scheme for client")]
    UnknownScheme,
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    Http(#[from] http::Error),
    #[error(transparent)]
    Source(Box<dyn std::error::Error>),
    #[error(transparent)]
    Handlebars(#[from] handlebars::RenderError),
    #[error(transparent)]
    Infallible(#[from] Infallible),
}
