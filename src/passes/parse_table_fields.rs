use itertools::Itertools;
use log::debug;

use crate::{doctree::{DocItemEnum, Field, Function, MetaFile, NamedFunction}, errors::*, json::{DefineType, ExtendsType}, workspace::SourceFile};

pub fn parse_table_fields(meta_file: &mut MetaFile, source_file: &SourceFile) -> Result<()> {
    for definition in source_file.definitions.iter() {
        if ! matches!(definition.defines.head.define_type, DefineType::TableField) {
            continue
        }

        let (enum_name, field_name) = definition.name.splitn(2, ".").collect_tuple()
            .ok_or_else(|| anyhow!("Invalid tablefield name {}", definition.name))?;

        if ! meta_file.items.contains_key(enum_name) {
            debug!("Skipping missing enum reference {}", definition.name);
            continue
        }

        let lua_enum = meta_file.items.get_mut(enum_name)
            .ok_or_else(|| anyhow!("missing enum"))?;

        debug!("Setting enum {:?} field {:?}", enum_name.to_string(), field_name.to_string());

        match lua_enum.inner {
            DocItemEnum::Enum(ref mut lua_enum) => {
                lua_enum.add_field(Field {
                    name: field_name.to_string(),
                    description: definition.rawdesc.clone(),
                    lua_type: "".to_string(), // TODO: no types in docs?
                })
            },
            _ => bail!("Setting field {} for non enum {}", field_name, lua_enum.name),
        }
    }

    Ok(())
}