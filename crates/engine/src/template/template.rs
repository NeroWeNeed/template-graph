use std::{
    borrow::Cow,
    io::{Write, stdout},
    path::PathBuf,
};

use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

use crate::{render::Render, value::Value};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Template {
    Value(String),
    File(PathBuf),
}

pub trait Renderer {
    fn render(&self, context: &Value, render: &Render) -> Result<(), crate::Error>;
}
impl<'a> Renderer for Handlebars<'a> {
    fn render(&self, context: &Value, render: &Render) -> Result<(), crate::Error> {
        let template: Cow<str> = match &render.template {
            Template::Value(template) => template.into(),
            Template::File(template) => std::fs::read_to_string(template)?.into(),
        };
        match &render.output {
            crate::render::Output::Stdout => {
                self.render_template_to_write(&template, context, stdout())?;
            }
            crate::render::Output::File(location) => {
                self.render_template_to_write(
                    &template,
                    context,
                    std::fs::File::create(location)?,
                )?;
            }
        }
        Ok(())
    }
}
