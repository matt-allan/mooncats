//! A simple example that reads `doc.json` JSON on stdin and writes pretty output to STDOUT.
//! It's like cat but for LuaCats files.
//! Usage example:
//! ```
//! cat testdata/doc.json | cargo run --example cat
//! ```

use std::{env, error::Error, io::{self, Read}, path::PathBuf};

use anyhow::anyhow;
use log::debug;
use mooncats::{doctree::{build_docs, DocItem}, json::Definition, location::FileUri, workspace::{self, Workspace}};
use url::Url;

extern crate mooncats;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    debug!("Starting mooncats example");

    let mut buffer = String::new();

    io::stdin().read_to_string(&mut buffer)?;

    let docs: Vec<Definition> = serde_json::from_str(&buffer)?;

    let path = env::current_dir()?;

    let mut workspace = Workspace::new(FileUri::try_from(path)?);
    workspace.load(docs)?;

    let meta = build_docs(workspace)?;

    println!("{:#?}", meta);

    Ok(())
}