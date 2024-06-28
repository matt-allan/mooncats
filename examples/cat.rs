//! A simple example that reads `doc.json` JSON on stdin and writes pretty output to STDOUT.
//! It's like cat but for LuaCats files.
//! Usage example:
//! ```
//! cat testdata/doc.json | cargo run --example cat
//! ```

use std::{error::Error, io::{self, Read}};

use mooncats::{doctree::DocItem, json::Definition};

extern crate mooncats;

fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer = String::new();

    io::stdin().read_to_string(&mut buffer)?;

    let docs: Vec<Definition> = serde_json::from_str(&buffer)?;

    for doc in docs.iter() {
        let item = DocItem::parse(&doc);

        match item {
            Ok(Some(item)) => {
                println!("{:#?}", item);
            },
            Err(err) => {
                if err.to_string() != "todo" {
                    println!("{:#?}", err);
                }
            },
            _ => {},
        }
    }

    Ok(())
}