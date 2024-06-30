use log::debug;

use crate::{doctree::{DocItem, DocItemEnum, MetaFile}, errors::*, workspace::SourceFile};

pub fn merge_class_tables(meta_file: &mut MetaFile, _source_file: &SourceFile) -> Result<()> {
    let tables: Vec<&DocItem> = meta_file.items
        .values()
        .filter(|item| matches!(item.inner, DocItemEnum::Table(_)))
        .collect();

    let classes: Vec<&DocItem> = meta_file.items
        .values()
        .filter(|item| matches!(item.inner, DocItemEnum::Class(_)))
        .collect();

    let mut removals = Vec::new();

    // TODO: cleaner way to do this match?
    for table_item in tables.iter() {
        for class_item in classes.iter() {
            match &table_item.inner {
                DocItemEnum::Table(table) => {
                    if table.view == class_item.name {
                        debug!("Merging table {} with class {}", table_item.name, class_item.name);
                        removals.push(table_item.name.clone());
                    }
                },
                _ => bail!("expected table"),
            }
        }
    }

    for key in removals.iter() {
        meta_file.items.remove(key);
    }

    Ok(())
}