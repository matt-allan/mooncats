use itertools::Itertools;
use log::debug;

use crate::{doctree::{DocItemEnum, Field, Function, MetaFile, NamedFunction}, errors::*, json::{DefineType, ExtendsType}, workspace::SourceFile};

pub fn parse_set_fields(meta_file: &mut MetaFile, source_file: &SourceFile) -> Result<()> {
    for definition in source_file.definitions.iter() {
        if ! matches!(definition.defines.head.define_type, DefineType::SetField | DefineType::SetIndex) {
            continue
        }

        let extends = definition.defines.head.extends
            .first()
            .ok_or_else(|| anyhow!("Expected an extends for setfield at {:?}", definition.defines.head.location.range))?;
        
        let (table_name, field_name) = definition.name.splitn(2, ".").collect_tuple()
            .ok_or_else(|| anyhow!("Invalid setfield name {}", definition.name))?;

        // Usually it's a class, which already captured this via "fields".
        // Sometimes it's naming a table "foo.bar", when "foo" was declared in
        // a parent module. We don't want to capture either of these.
        if ! meta_file.items.contains_key(table_name) {
            debug!("Skipping missing table reference {}", definition.name);
            continue
        }

        let table = meta_file.items.get_mut(table_name)
            .ok_or_else(|| anyhow!("missing table"))?;

        debug!("Setting table {:?} field {:?}", table_name.to_string(), field_name.to_string());

        match table.inner {
            DocItemEnum::Table(ref mut table) => {
                match extends.extends_type {
                    ExtendsType::Binary |
                    ExtendsType::Integer |
                    ExtendsType::Nil |
                    ExtendsType::Number |
                    ExtendsType::String |
                    ExtendsType::Table => {
                        let field = Field {
                            name: field_name.to_string(),
                            description: definition.rawdesc.clone(),
                            lua_type: extends.view.clone(),
                        };
                        debug!("Adding table field {:?}", field_name.to_string());
                        table.add_field(field);
                    },
                    ExtendsType::Function => {
                        let method = NamedFunction {
                            name: field_name.to_string(),
                            function: Function::parse(extends)?,
                        };

                        table.add_function(method);
                    }
                    _ => bail!("Unexpected setfield type {:?}", extends.extends_type)
                }
            },
            DocItemEnum::Class(_) => {}, // Ignore, already set via "fields" attribute
            _ => bail!("Setting field {} for non-table {}", field_name, table.name),
        }
    }

    Ok(())
}