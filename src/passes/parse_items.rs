use crate::{doctree::{DocItem, MetaFile}, errors::*, workspace::SourceFile};

pub fn parse_items(meta_file: &mut MetaFile, source_file: &SourceFile) -> Result<()> {
    for definition in source_file.definitions.iter() {
        let item = DocItem::parse(definition)?;

        if let Some(item) = item {
            meta_file.add_item(item);
        }
    }

    Ok(())
}