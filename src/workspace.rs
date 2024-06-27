//! The internal representation of a folder of docs on disk.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use itertools::Itertools;
use log::*;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::json::Definition;
use crate::errors::*;

/// A folder containing LuaCats definition files.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Workspace {
    /// Root URI for the workspace.
    pub root: Url,
    /// All files within the workspace.
    pub files: HashMap<Url,MetaFile>,
}

impl Workspace {
    pub fn new(root: Url) -> Self {
        Self {
            root,
            files: HashMap::new(),
        }
    }

    pub fn load(&mut self, docs: Vec<Definition>) -> Result<()> {
        for doc in docs.into_iter() {
            let uris: Vec<Url> = match &doc {
                Definition::Type(definition) => definition.defines.iter().map(|def| def.location.file.clone()).unique().collect(),
                Definition::Variable(definition) => definition.defines.iter().map(|def| def.location.file.clone()).unique().collect(),
            };

            for uri in uris.iter() {
                let relative_uri = {
                    self.root.make_relative(&uri)
                };

                if relative_uri.is_none() {
                    let def_name = match &doc {
                        Definition::Type(def) => &def.name,
                        Definition::Variable(def) => &def.name,
                    };
                    debug!("Skipping declaration outside of workspace: {}", def_name);
                    continue
                }

                if !self.files.contains_key(&uri) {
                    let file = MetaFile::open(&uri)?;
                    self.files.insert(uri.clone(), file);
                }

                let file = self.files.get_mut(&uri).unwrap();
                file.add_definition(doc.clone());
            }
        }

        Ok(())
    }
}

/// A Lua file containing only LuaCats meta.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MetaFile {
    /// The absolute URI for this file.
    pub uri: Url,
    /// Raw definitions within this file.
    pub definitions: Vec<Definition>,
    /// The file contents.
    pub text: String,
}

impl MetaFile {
    pub fn new(uri: Url, text: String) -> Self {
        Self {
            uri,
            definitions: Vec::new(),
            text,
        }
    }

    pub fn open(uri: &Url) -> Result<Self> {
        let path = uri.to_file_path()
            .map_err(|_| anyhow!("File URI '{}' is not a valid path", uri))?;
        let mut text = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut text)?;

        Ok(Self {
            uri: uri.clone(),
            definitions: Vec::new(),
            text,
        })
    }

    pub fn add_definition(&mut self, declaration: Definition) {
        self.definitions.push(declaration);
    }
}