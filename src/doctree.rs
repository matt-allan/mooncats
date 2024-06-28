use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use url::Url;
use crate::{errors::*, workspace::{self, Workspace}};

pub fn build_docs(workspace: Workspace) -> Result<Vec<DocFile>> {
  todo!()
}


#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DocFile {
    pub uri: Url,
    pub classes: Vec<Class>,
}


#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Class {
    name: String,
    description: Option<String>,
    fields: Vec<Field>,
}


#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Field {
    name: String,
    description: Option<String>,
}