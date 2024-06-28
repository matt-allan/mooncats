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

/// A root folder containing LuaCats definition files.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Workspace {
    /// Root URI for the workspace.
    root: Url,
    /// All files within the workspace.
    files: HashMap<Url,SourceFile>,
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
            let uris: Vec<Url> = doc
                .defines
                .iter()
                .map(|def| def.location.file.clone())
                .unique()
                .collect();

            for uri in uris.iter() {
                let relative_uri = {
                    self.root.make_relative(&uri)
                };

                if relative_uri.is_none() {
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

    fn file_depth(&self, file: &SourceFile) -> Result<usize> {
        let rel_url = self.root.make_relative(&file.uri)
            .ok_or_else(|| anyhow!("File not rooted in workspace"))?;
        let rel_url = Url::parse(&rel_url)?;
        let segments = rel_url.path_segments().ok_or_else(|| anyhow!("invalid file URI"));

        let mut n = 0;
        for _ in segments.iter() {
            n += 1
        }
        n = (n-1).max(0); // don't count the last segment, which is the filename
        Ok(n)
    }
}

impl<'a> IntoIterator for &'a Workspace {
    type Item = &'a SourceFile;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.files
            .values()
            .sorted_by(|a, b|
                // Unwrapping here is generally safe because we already checked the URI on load
                self.file_depth(&a).unwrap()
                    .cmp(&self.file_depth(&b).unwrap())
                    .then(
                        a.uri.to_file_path().unwrap().file_name().unwrap()
                            .cmp(&b.uri.to_file_path().unwrap().file_name().unwrap()))
            )
            .into_iter()
    }
}

/// A Lua file containing only LuaCats meta.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceFile {
    /// The absolute URI for this file.
    pub uri: Url,
    /// Raw definitions within this file.
    pub definitions: Vec<Definition>,
    /// The file contents.
    pub text: String,
}

impl SourceFile {
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