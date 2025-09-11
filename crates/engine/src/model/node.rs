use std::borrow::Cow;

use crate::{render::Render, source::Operation, value::Value};

pub type ModelNodeId = usize;
#[derive(Debug)]
pub struct ModelNode<'a> {
    pub name: Option<Cow<'a, str>>,
    pub value: Value,
    pub output: Option<Render>,
    pub inherit: bool,
}
impl<'a> Default for ModelNode<'a> {
    fn default() -> Self {
        Self {
            name: Default::default(),
            value: Value::Null,
            output: Default::default(),
            inherit: true,
        }
    }
}

impl<'a> ModelNode<'a> {
    /// Attempts to update the value, writing to disk or replacing the value in memory. returns
    /// true if the new value is considered different than the old value.
    /// Values stored in memory are checked for equality, while values stored on disk have their
    /// hashes checked.
    fn set(&mut self, value: Value) -> bool {
        if self.value != value {
            self.value = value;
            true
        } else {
            false
        }
    }
    /// Attempts to merge the incoming value with the current value. Currently this feature is only
    /// supported with in-memory operations. Attempting to merge against a location on disk will
    /// result in loading the entire location into memory, and operating on it there.
    fn merge(&mut self, value: Value) -> bool {
        let value = self.value.clone().merge(&value);
        if self.value != value {
            self.value = value;
            true
        } else {
            false
        }
    }
    pub fn apply(&mut self, operation: Operation) -> bool {
        match operation {
            Operation::Set(value) => self.set(value),
            Operation::Merge(value) => self.merge(value),
            Operation::Remove(path) => {
                if path.is_empty() {
                    self.set(Value::Null)
                } else {
                    self.merge(path.split('.').rev().fold(Value::Null, |acc, node| {
                        Value::Object(vec![(node.into(), acc)])
                    }))
                }
            }
            Operation::Reload => true,
        }
    }
}
