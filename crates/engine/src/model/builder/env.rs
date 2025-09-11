use crate::{
    model::{
        builder::ModelBuilder,
        node::{ModelNode, ModelNodeId},
    },
    source::env::{EnvNodeIdProvider, EnvNodeIdStorage},
    value::Value,
};
const ENV_NODE_NAME: &'static str = "env";
impl<'a, H> EnvNodeIdProvider for ModelBuilder<'a, H>
where
    H: EnvNodeIdStorage,
{
    type Error = crate::Error;

    fn get_env_node_id(&mut self) -> Result<ModelNodeId, Self::Error> {
        match self.header().get_env_node_id() {
            Some(value) => Ok(value),
            None => {
                let env_node_id = self.allocate_node(
                    ModelNode {
                        name: Some(ENV_NODE_NAME.into()),
                        value: Value::Object(
                            std::env::vars()
                                .map(|(key, value)| (key.into(), Value::String(value)))
                                .collect(),
                        ),
                        output: None,
                        inherit: false,
                    },
                    false,
                )?;
                self.header_mut().set_env_node_id(env_node_id);
                Ok(env_node_id)
            }
        }
    }
}
