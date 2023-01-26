use crate::static_var::{EMBED_DIR, TEMPLATES};
use anyhow::Result;
use config::Value;
use tera::Context;

static INVALID_CONFIG_VALUE: &str = "invalid value in configuration";

#[derive(Debug)]
pub enum TemplateContent {
    Path { path: String, is_embedded: bool },
    Content(String),
}

#[derive(Debug)]
pub struct Template {
    pub name: String,
    pub content: TemplateContent,
    pub content_type: String,
}

impl Template {
    pub fn from((k, v): (String, Value)) -> Option<Self> {
        let v = v.clone().into_table().expect(INVALID_CONFIG_VALUE);
        if let Some(disabled) = v.get("disabled") {
            if disabled.clone().into_bool().unwrap_or(false){
                return None;
            }
        }

        let content: TemplateContent = if v.contains_key("path") {
            let is_embedded = v.get("is_embedded").map_or(false, |v| {
                v.clone().into_bool().expect(INVALID_CONFIG_VALUE)
            });
            let path = v.get("path").expect(INVALID_CONFIG_VALUE).to_string();
            TemplateContent::Path { path, is_embedded }
        } else {
            let content = v.get("content").expect(INVALID_CONFIG_VALUE).to_string();
            TemplateContent::Content(content)
        };
        let content_type = v
            .get("content_type")
            .map_or("text/html; charset=UTF-8".to_string(), |v| v.to_string());

        Some(Self {
            name: k.clone(),
            content,
            content_type,
        })
    }

    pub fn get_content(self: &Self) -> &str {
        match &self.content {
            TemplateContent::Path {
                path,
                is_embedded: true,
            } => EMBED_DIR
                .get_file(path)
                .expect("file not found in embedded files")
                .contents_utf8()
                .unwrap_or_default(),
            TemplateContent::Path {
                path: _,
                is_embedded: false,
            } => panic!("not implemented"),
            TemplateContent::Content(v) => &v,
        }
    }

    pub fn render(self: &Self, context: &Context) -> Result<(&str, String)> {
        Ok((&self.content_type, TEMPLATES.render(&self.name, context)?))
    }
}
