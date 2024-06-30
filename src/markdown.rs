use crate::errors::*;
use handlebars::{no_escape, Handlebars};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};

use crate::doctree::{DocItem, DocItemEnum, MetaFile};

#[derive(Embed)]
#[folder = "templates"]
#[include = "*.hbs"]
struct Assets;

pub struct MarkdownRenderer<'a> {
    hbs: Handlebars<'a>,
}


#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct TemplateData {
    pub name: String,
    pub classes: Vec<DocItem>,
    pub tables: Vec<DocItem>,
    pub type_aliases: Vec<DocItem>,
    pub enums: Vec<DocItem>,
    pub globals: Vec<DocItem>,
}

impl From<&MetaFile> for TemplateData {
    fn from(file: &MetaFile) -> Self {
        let name = file.uri.file_stem();

        // TODO: use a macro or something to clean this up
        let classes: Vec<DocItem> = file
            .items
            .values()
            .filter(|item| matches!(item.inner, DocItemEnum::Class(_)))
            .cloned()
            .collect();
        let tables: Vec<DocItem> = file
            .items
            .values()
            .filter(|item| matches!(item.inner, DocItemEnum::Table(_)))
            .cloned()
            .collect();
        let type_aliases: Vec<DocItem> = file
            .items
            .values()
            .filter(|item| matches!(item.inner, DocItemEnum::TypeAlias(_)))
            .cloned()
            .collect();
        let enums: Vec<DocItem> = file
            .items
            .values()
            .filter(|item| matches!(item.inner, DocItemEnum::Enum(_)))
            .cloned()
            .collect();
        let globals: Vec<DocItem> = file
            .items
            .values()
            .filter(|item| matches!(item.inner, DocItemEnum::Global(_)))
            .cloned()
            .collect();

        Self {
            name,
            classes,
            tables,
            type_aliases,
            enums,
            globals,
        }
    }
}

impl<'a> MarkdownRenderer<'a> {
    pub fn new() -> Self {
        let mut hbs = Handlebars::new();

        hbs.set_strict_mode(true);
        hbs.register_embed_templates_with_extension::<Assets>(".hbs").expect("invalid templates");
        hbs.register_escape_fn(no_escape);

        Self {
            hbs
        }
    }

    pub fn render_meta(&self, meta_file: &MetaFile) -> Result<String> {
        let data: TemplateData = meta_file.into();

        Ok(self.hbs.render("meta_file", &data)?)
    }
}