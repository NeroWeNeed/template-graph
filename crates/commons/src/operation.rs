use crate::value::Value;

#[derive(Debug, Clone)]
pub enum Operation {
    Set(Value),
    Merge(Value),
    Remove(String),
    Reload,
}
