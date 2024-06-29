//! The internal representation of a folder of docs on disk.

use std::char::REPLACEMENT_CHARACTER;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use itertools::Itertools;
use log::*;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::json::Definition;
use crate::errors::*;
use crate::location::FileUri;

/// A root folder containing LuaCats definition files.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Workspace {
    /// Root URI for the workspace.
    root: FileUri,
    /// All files within the workspace.
    files: HashMap<FileUri,SourceFile>,
}

impl Workspace {
    pub fn new(root: FileUri) -> Self {
        Self {
            root,
            files: HashMap::new(),
        }
    }

    pub fn load(&mut self, docs: Vec<Definition>) -> Result<()> {
        for doc in docs.into_iter() {
            let uris: Vec<FileUri> = doc
                .defines
                .iter()
                .map(|def| def.location.file.clone())
                .unique()
                .collect();

            for uri in uris.iter() {
                if !uri.starts_with_path(&self.root) {
                    debug!("Skipping declaration outside of workspace: {}", doc.name);
                    continue
                }

                if !self.files.contains_key(&uri) {
                    let file = SourceFile::open(&uri)?;
                    self.files.insert(uri.clone(), file);
                }

                let file = self.files.get_mut(&uri).unwrap();
                file.add_definition(doc.clone())?;
            }
        }

        Ok(())
    }
}

impl<'a> IntoIterator for &'a Workspace {
    type Item = &'a SourceFile;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.files
            .values()
            .sorted_by(|a, b|
                a.uri.depth()
                    .cmp(&b.uri.depth())
                    .then(
                        a.uri.file_name()
                            .cmp(&b.uri.file_name()))
            )
            .into_iter()
    }
}

/// A Lua file containing only LuaCats meta.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceFile {
    /// The absolute URI for this file.
    pub uri: FileUri,
    /// Raw definitions within this file.
    pub definitions: Vec<Definition>,
    /// The file contents.
    pub text: String,
}

impl SourceFile {
    pub fn new(uri: FileUri, text: String) -> Self {
        Self {
            uri,
            definitions: Vec::new(),
            text,
        }
    }

    pub fn open(uri: &FileUri) -> Result<Self> {
        let path = uri.to_file_path()?;
        let mut text = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut text)?;

        Ok(Self {
            uri: uri.clone(),
            definitions: Vec::new(),
            text,
        })
    }

    pub fn add_definition(&mut self, mut definition: Definition) -> Result<()> {
        definition.defines = definition.defines
            .into_iter()
            .filter(|define| define.location.file == self.uri)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| anyhow!("No defines belong to this file"))?;
        
        self.definitions.push(definition);

        Ok(())
    }
}