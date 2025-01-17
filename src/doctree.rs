use std::collections::HashMap;

use crate::{
    errors::*,
    json::{
        self, ArgType, DefineType, Definition, DefinitionType, Extends, ExtendsType, FieldType,
    },
    location::{FileUri, Range},
    passes::{merge_class_tables, parse_items, parse_set_fields, parse_table_fields},
    workspace::Workspace,
};
use itertools::Itertools;
use log::debug;
use serde::{Deserialize, Serialize};

pub fn build_docs(workspace: Workspace) -> Result<DocTree> {
    debug!("building docs");

    let mut meta_files: Vec<MetaFile> = Vec::new();

    for source_file in workspace.into_iter() {
        let mut meta_file = MetaFile::new(source_file.uri.clone());

        parse_items(&mut meta_file, source_file)?;
        parse_set_fields(&mut meta_file, source_file)?;
        parse_table_fields(&mut meta_file, source_file)?;
        merge_class_tables(&mut meta_file, source_file)?;

        meta_files.push(meta_file);
    }

    let tree = build_tree(&workspace.root, meta_files);

    Ok(tree)
}

fn build_tree(root: &FileUri, meta_files: Vec<MetaFile>) -> DocTree {
    let by_depth = meta_files.into_iter().sorted_by(|a, b| {
        a.uri
            .relative_depth(root)
            .cmp(&b.uri.relative_depth(root))
            .then(a.uri.file_name().cmp(&b.uri.file_name()))
    });
    let mut tree = DocTree::new();

    for meta_file in by_depth {
        if meta_file.uri.relative_depth(root) == 1 {
            tree.add_item(meta_file);
        } else {
            tree.for_each_mut(|file| {
                if file.uri.depth() != meta_file.uri.depth() - 1 {
                    return;
                }

                if file.uri.file_stem() != meta_file.uri.dirname().unwrap_or_default() {
                    return;
                }

                file.children.push(meta_file.clone())
            });
        }
    }

    tree
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct DocTree(Vec<MetaFile>);

impl DocTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_each_mut<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut MetaFile),
    {
        for_each_mut(&mut func, &mut self.0);
    }

    pub fn add_item(&mut self, item: MetaFile) {
        self.0.push(item)
    }
}

impl IntoIterator for DocTree {
    type Item = MetaFile;

    type IntoIter = std::vec::IntoIter<MetaFile>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub fn for_each_mut<'a, F, I>(func: &mut F, items: I)
where
    F: FnMut(&mut MetaFile),
    I: IntoIterator<Item = &'a mut MetaFile>,
{
    for item in items {
        for_each_mut(func, &mut item.children);

        func(item);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MetaFile {
    pub uri: FileUri,
    pub children: Vec<MetaFile>,
    pub items: HashMap<String, DocItem>,
}

impl MetaFile {
    pub fn new(uri: FileUri) -> Self {
        Self {
            uri,
            children: Vec::new(),
            items: HashMap::new(),
        }
    }

    pub fn add_item(&mut self, item: DocItem) {
        self.items.insert(item.name.clone(), item);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DocItem {
    pub name: String,
    pub description: Option<String>,
    pub range: Range,
    #[serde(flatten)]
    pub inner: DocItemEnum,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum DocItemEnum {
    Class(Class),
    Table(Table),
    TypeAlias(TypeAlias),
    Enum(Enum),
    Global(Global),
}

impl DocItem {
    pub fn parse(definition: &Definition) -> Result<Option<Self>> {
        let inner = match definition.defines.head.define_type {
            DefineType::DocAlias => Some(DocItemEnum::TypeAlias(TypeAlias::parse(definition)?)),
            DefineType::DocClass => Some(DocItemEnum::Class(Class::parse(definition)?)),
            DefineType::DocEnum => Some(DocItemEnum::Enum(Enum::parse(definition)?)),
            DefineType::SetGlobal => {
                let extends = definition
                    .defines
                    .head
                    .extends
                    .first()
                    .ok_or_else(|| anyhow!("expected extends for setglobal"))?;

                match extends.extends_type {
                    ExtendsType::DocType | ExtendsType::DocExtendsName => {
                        bail!("unexpected doc extend for setglobal")
                    }
                    ExtendsType::Table => Some(DocItemEnum::Table(Table::parse(definition)?)),
                    _ => Some(DocItemEnum::Global(Global::parse(definition)?)),
                }
            }
            // The other define types extend another doc item
            _ => None,
        };

        Ok(inner.map(|inner| DocItem {
            name: definition.name.clone(),
            description: definition.rawdesc.clone(),
            range: definition.defines.head.location.range.clone(),
            inner,
        }))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct Class {
    pub fields: Vec<Field>,
    pub methods: Vec<NamedFunction>,
}

impl Class {
    pub fn parse(definition: &Definition) -> Result<Self> {
        ensure!(definition.definition_type == DefinitionType::Type);
        ensure!(definition.defines.head.define_type == DefineType::DocClass);

        let fields: Vec<Field> = definition
            .fields
            .iter()
            .filter(|f| {
                (f.field_type == FieldType::DocField || f.field_type == FieldType::SetField)
                    && f.extends.extends_type != ExtendsType::Function
            })
            .map(|f| Field::parse(f))
            .collect::<Result<Vec<Field>>>()?;

        let functions: Vec<NamedFunction> = definition
            .fields
            .iter()
            .filter(|f| 
                f.field_type == FieldType::SetField
                    && f.extends.extends_type == ExtendsType::Function
            ).map(|field| -> Result<NamedFunction> {
                Ok(NamedFunction {
                    name: field.name.clone(),
                    function: Function::parse(&field.extends)?,
                })
            })
            .collect::<Result<Vec<NamedFunction>>>()?;

        let methods: Vec<NamedFunction> = definition
            .fields
            .iter()
            .filter(|f| f.field_type == FieldType::SetMethod)
            .map(|f| NamedFunction::parse(f))
            .collect::<Result<Vec<NamedFunction>>>()?;

        let class = Self {
            fields,
            methods: functions.into_iter().merge(methods).collect(),
        };

        Ok(class)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct Table {
    pub view: String,
    pub fields: HashMap<String, Field>,
    pub functions: HashMap<String, NamedFunction>,
}

impl Table {
    pub fn parse(definition: &Definition) -> Result<Self> {
        ensure!(definition.defines.head.define_type == DefineType::SetGlobal);
        let extends = definition
            .defines
            .head
            .extends
            .first()
            .ok_or_else(|| anyhow!("expected extends for setglobal"))?;
        ensure!(extends.extends_type == ExtendsType::Table);

        Ok(Table {
            view: extends.view.clone(),
            fields: HashMap::new(),
            functions: HashMap::new(),
        })
    }

    pub fn add_field(&mut self, field: Field) {
        self.fields.insert(field.name.clone(), field);
    }

    pub fn add_function(&mut self, function: NamedFunction) {
        self.functions.insert(function.name.clone(), function);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeAlias {
    #[serde(rename = "type")]
    pub aliased_type: String,
}

impl TypeAlias {
    pub fn parse(definition: &Definition) -> Result<Self> {
        let define = &definition.defines.head;
        ensure!(define.define_type == DefineType::DocAlias);
        let extends = define
            .extends
            .first()
            .ok_or_else(|| anyhow!("expected extends for type alias"))?;
        ensure!(extends.extends_type == ExtendsType::DocType);

        Ok(Self {
            aliased_type: extends.view.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct Enum {
    pub fields: HashMap<String,Field>,
}

impl Enum {
    pub fn parse(definition: &Definition) -> Result<Self> {
        let define = &definition.defines.head;
        ensure!(define.define_type == DefineType::DocEnum);

        Ok(Self::default())
    }

    pub fn add_field(&mut self, field: Field) {
        self.fields.insert(field.name.clone(), field);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Global {
    Primitive(PrimitiveGlobal),
    Function(Function),
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PrimitiveGlobal {
    #[serde(rename = "type")]
    primitive_type: String,
}

impl Global {
    pub fn parse(definition: &Definition) -> Result<Self> {
        ensure!(definition.defines.head.define_type == DefineType::SetGlobal);
        let extends = definition
            .defines
            .head
            .extends
            .first()
            .ok_or_else(|| anyhow!("expected extends for setglobal"))?;
        let extends_primitive = extends.extends_type == ExtendsType::Integer
            || extends.extends_type == ExtendsType::Nil
            || extends.extends_type == ExtendsType::Number
            || extends.extends_type == ExtendsType::String
            || extends.extends_type == ExtendsType::Binary;
        ensure!(extends_primitive || extends.extends_type == ExtendsType::Function);

        Ok(match extends.extends_type {
            ExtendsType::Binary
            | ExtendsType::Integer
            | ExtendsType::Nil
            | ExtendsType::Number
            | ExtendsType::String => Global::Primitive(PrimitiveGlobal { primitive_type: extends.view.clone() }),
            ExtendsType::Function => Global::Function(Function::parse(extends)?),
            _ => bail!("unexpected extends type {:?}", extends.extends_type),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub lua_type: String,
}

impl Field {
    pub fn parse(field: &json::Field) -> Result<Self> {
        ensure!(field.field_type == FieldType::DocField || field.field_type == FieldType::SetField);

        Ok(Field {
            name: field.name.clone(),
            description: field.rawdesc.clone(),
            lua_type: field.extends.view.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Function {
    pub description: Option<String>,
    pub view: String,
    pub arguments: Vec<Argument>,
    pub returns: Vec<Return>,
}

impl Function {
    pub fn parse(extends: &Extends) -> Result<Self> {
        ensure!(extends.extends_type == ExtendsType::Function);

        let arguments = extends
            .args
            .iter()
            .map(|arg| Argument::parse(arg))
            .collect::<Result<Vec<Argument>>>()?;

        let returns = extends
            .returns
            .iter()
            .map(|ret| Return::parse(ret))
            .collect::<Result<Vec<Return>>>()?;

        let mut view = extends.view.clone();
        if let Some(stripped) = view.strip_prefix("(method) ") {
            view = stripped.to_string();
        }

        Ok(Self {
            description: extends.rawdesc.clone(),
            view,
            arguments,
            returns,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NamedFunction {
    pub name: String,
    #[serde(flatten)]
    pub function: Function,
}

impl NamedFunction {
    pub fn parse(field: &json::Field) -> Result<Self> {
        ensure!(field.field_type == FieldType::SetMethod);
        ensure!(field.extends.extends_type == ExtendsType::Function);

        Ok(NamedFunction {
            name: field.name.clone(),
            function: Function::parse(&field.extends)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Argument {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub arg_type: String,
}

impl Argument {
    pub fn parse(arg: &json::FuncArg) -> Result<Self> {
        let arg_type = match arg.arg_type {
            ArgType::DocType => arg.view.clone(),
            ArgType::Local => arg.view.clone(),
            ArgType::SelfType => "self".to_string(),
            ArgType::VarArg => "...".to_string(),
        };

        Ok(Self {
            name: arg.name.clone(),
            description: arg.rawdesc.clone(),
            arg_type,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Return {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub return_type: String,
    pub description: Option<String>,
}

impl Return {
    pub fn parse(ret: &json::FuncReturn) -> Result<Self> {
        Ok(Self {
            name: ret.name.clone(),
            description: ret.rawdesc.clone(),
            return_type: ret.view.clone(),
        })
    }
}
