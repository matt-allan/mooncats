use anyhow::anyhow;
use mdbook::{book::{Book, Chapter, SectionNumber}, preprocess::{Preprocessor, PreprocessorContext}, BookItem};
use mdbook::errors::Error as MdBookError;
use tempdir::TempDir;
use std::{env, fs::{self}, path::PathBuf, process::Command};
use toml::value::Table;
use log::*;

use crate::{doctree::{build_docs, MetaFile}, errors::*, json::Definition, location::FileUri, markdown::{self, MarkdownRenderer}, workspace::Workspace};

/// Configuration for the preprocessor.
#[derive(Debug, Default)]
pub struct Config {
    definitions_path: Option<PathBuf>,
    part_title: Option<String>,
    nav_depth: Option<u8>,
}

impl<'a> From<Option<&'a Table>> for Config {
    fn from(table: Option<&'a Table>) -> Config {
        let mut config = Config::default();

        if let Some(table) = table {
            config.definitions_path = table
                .get("definitions-path")
                .and_then(|v| v.as_str())
                .and_then(|v| Some(v.to_owned().into()));

            config.part_title = table
                .get("part-title")
                .and_then(|v| v.as_str())
                .and_then(|v| Some(v.to_owned()));

            config.nav_depth = table
                .get("nav-depth")
                .and_then(|v| v.as_integer())
                .and_then(|v| Some(v.try_into().expect("nav-depth overflow")));
        }

        config
    }    
}

/// A mdbook preprocessor that generates LuaCATS API docs.
pub struct MoonCats;

impl MoonCats {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MoonCats {
    fn default() -> Self {
        Self {}
    }
}

impl Preprocessor for MoonCats {
    fn name(&self) -> &str {
        "mooncats-preprocessor"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, MdBookError> {
        let config: Config = ctx.config.get_preprocessor(self.name()).into();

        debug!("Using mdbook root: {:?}", ctx.root);
        debug!("Using definitions path: {:?}", config.definitions_path);

        let mut root = ctx.root.clone();
        if root.is_relative() {
            root = env::current_dir()?.join(ctx.root.clone())
        }
        let mut root_path = config.definitions_path
            .unwrap_or_else(|| PathBuf::from("library"));
        if root_path.is_relative() {
            root_path = root.join(root_path);
        }
        debug!("Using root path: {:?}", root_path);

        let docs = generate_json_docs(&root_path)?;
        debug!("Generated {} definitions", docs.len());

        let root_uri: FileUri = root_path.clone().try_into()?;
        
        let mut workspace = Workspace::new(root_uri);
        workspace.load(docs)?;
        debug!("Loaded {} root files", workspace.files.len());

        let doc_tree = build_docs(workspace)?;

        let md = MarkdownRenderer::new();

        let part_title = config.part_title.unwrap_or("API Reference".into());
        book.push_item(BookItem::PartTitle(part_title));

        for (index, file) in doc_tree.into_iter().enumerate() {
            let chapter = build_chapter(&md, &root_path, &file, index, None)?;
            book.push_item(BookItem::Chapter(chapter));
        }

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html" || renderer == "epub"
    }
}

fn build_chapter(md: &MarkdownRenderer, base: &PathBuf, file: &MetaFile, index: usize, parent: Option<&Chapter>) -> anyhow::Result<Chapter> {
    let name = file.uri.file_stem(); 
    let content = md.render_meta(file)?;
    let md_path = file.uri.to_file_path()?
        .strip_prefix(base)?
        .with_extension("md");
    let number = match parent {
        Some(parent) => {
            let mut number = parent.number.clone().unwrap_or_else(|| SectionNumber(Vec::new()));
            number.0.push(u32::try_from(index).unwrap()+1);
            number
        },
        None => SectionNumber(vec![u32::try_from(index).unwrap()+1])
    };
    let parent_names = match parent {
        Some(parent) => {
            let mut names = parent.parent_names.clone();
            names.push(parent.name.clone());
            names
        },
        None => Vec::new(),
    };

    let mut chapter = Chapter {
        name,
        content,
        number: Some(number),
        sub_items: Vec::new(),
        path: Some(md_path),
        source_path: None,
        parent_names,
    };

    chapter.sub_items = file.children
        .iter()
        .enumerate()
        .map(|(sub_index, sub_file)| -> anyhow::Result<BookItem> {
            let chapter = build_chapter(md, base, sub_file, sub_index, Some(&chapter))?;
            Ok(BookItem::Chapter(chapter))
        })
        .collect::<anyhow::Result<Vec<BookItem>>>()?;

    Ok(chapter)
}

/// Spawn the lua-language-server to generate docs.
fn generate_json_docs(definitions_path: &PathBuf) -> Result<Vec<Definition>> { 
    let tmp_dir = TempDir::new("luals-docs")?;
    let tmp_path = tmp_dir.path();

    let output = Command::new("lua-language-server")
        .arg("--doc")
        .arg(definitions_path)
        .arg("--doc_out_path")
        .arg(tmp_path)
        .arg("--logpath")
        .arg(tmp_path)
        .output()?;

    if !output.status.success() {
        let err = match output.status.code() {
            Some(code) => anyhow!("LuaLS process exited with status code {}", code),
            None => anyhow!("LuaLS process terminated by signal"),
        };
        return Err(err)
    }

    let json_doc_path = tmp_dir.path().join("doc.json");

    let json_doc = fs::read_to_string(json_doc_path)?;

    let definitions: Vec<Definition> = serde_json::from_str(&json_doc)?;

    Ok(definitions)
}

#[cfg(test)]
mod test {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn preprocessor_run() {
        init();

        let input_json = r##"[
            {
                "root": "../test_book",
                "config": {
                    "book": {
                        "authors": ["AUTHOR"],
                        "language": "en",
                        "multilingual": false,
                        "src": "src",
                        "title": "TITLE"
                    },
                    "preprocessor": {
                        "luacats": {
                          "definitions-path": "library" 
                        }
                    }
                },
                "renderer": "html",
                "mdbook_version": "0.4.21"
            },
            {
                "sections": [
                    {
                        "Chapter": {
                            "name": "Chapter 1",
                            "content": "# Chapter 1\n",
                            "number": [1],
                            "sub_items": [],
                            "path": "chapter_1.md",
                            "source_path": "chapter_1.md",
                            "parent_names": []
                        }
                    }
                ],
                "__non_exhaustive": null
            }
        ]"##;
        let input_json = input_json.as_bytes();

        let (ctx, book) = mdbook::preprocess::CmdPreprocessor::parse_input(input_json).unwrap();
        let result = MoonCats::new().run(&ctx, book);
        assert!(result.is_ok(), "preprocessor failed: {:#?}", result.err());

        let actual_book = result.unwrap();

        // TODO: better asserts
        assert_eq!(actual_book.sections.len(), 2); // Chapter 1, Chapter "hello"
    }
}
