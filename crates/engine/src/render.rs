use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::template::Template;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Render {
    pub template: Template,
    pub output: Output,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Output {
    Stdout,
    File(PathBuf),
}
