//! A simple example that reads `doc.json` JSON on stdin and writes pretty output to STDOUT.
//! It's like cat but for LuaCats files.
//! Usage example:
//! ```
//! cat testdata/doc.json | cargo run --example cat
//! ```

use std::{error::Error, io::{self, Read}, path::PathBuf};


use log::debug;
use mooncats::{doctree::{build_docs}, json::Definition, location::FileUri, workspace::{Workspace}};


extern crate mooncats;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    debug!("Starting mooncats example");

    let mut buffer = String::new();

    io::stdin().read_to_string(&mut buffer)?;

    let docs: Vec<Definition> = serde_json::from_str(&buffer)?;

    let path = PathBuf::from("/Users/matt/Code/renoise-definitions/library");

    let mut workspace = Workspace::new(FileUri::try_from(path)?);
    workspace.load(docs)?;

    let tree = build_docs(workspace)?;

    // println!("{:#?}", meta);

    Ok(())
}