use std::collections::BTreeMap;

use crate::{
    model::{
        allocator::NodeAllocator,
        builder::ModelBuilder,
        node::{ModelNode, ModelNodeId},
    },
    value::Value,
};

const VAR_NODE_NAME: &'static str = "var";
impl<'a, H> ModelBuilder<'a, H> {
    pub fn allocate_var_node(
        &mut self,
        variables: BTreeMap<String, toml::Value>,
    ) -> Result<ModelNodeId, crate::Error> {
        let mut var_values = Vec::new();
        for (name, variable) in variables {
            var_values.push((name, variable.try_into()?));
        }
        let node_id = self.allocate_node(ModelNode {
            name: Some(VAR_NODE_NAME.into()),
            value: Value::Object(vec![(VAR_NODE_NAME.into(), Value::Object(var_values))]),
            output: None,
            inherit: false,
        })?;
        Ok(node_id)
    }
}
