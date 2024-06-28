use serde::{Deserialize, Serialize};
use url::Url;
use crate::{errors::*, json::{self, ArgType, Define, DefineType, Definition, DefinitionType, Extends, ExtendsType, FieldType}, workspace::{self, Workspace}};

pub fn build_docs(workspace: Workspace) -> Result<Vec<MetaFile>> {
    let mut meta_files: Vec<MetaFile> = Vec::new();

    for source_file in workspace.into_iter() {
        let mut meta_file = MetaFile::new(source_file.uri.clone());

        for definition in source_file.definitions.iter() {
            let item = DocItem::parse(definition)?;

            if let Some(item) = item {
                meta_file.add_item(item);
            }
        }

        // TODO: second pass to set fields etc

        meta_files.push(meta_file);
    }

    // todo: link into tree

    Ok(meta_files)
}


#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MetaFile {
    pub uri: Url,
    pub children: Vec<MetaFile>,
    pub classes: Vec<Class>,
    pub tables: Vec<Table>,
    pub type_aliases: Vec<TypeAlias>,
    pub enums: Vec<Enum>,
    pub globals: Vec<Global>,
}

impl MetaFile {
    pub fn new(uri: Url) -> Self {
        Self {
            uri,
            children: Vec::new(),
            classes: Vec::new(),
            tables: Vec::new(),
            type_aliases: Vec::new(),
            enums: Vec::new(),
            globals: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: DocItem) {
        match item {
            DocItem::Class(class) => self.classes.push(class),
            DocItem::Table(table) => self.tables.push(table),
            DocItem::TypeAlias(type_alias) => self.type_aliases.push(type_alias),
            DocItem::Enum(doc_enum) => self.enums.push(doc_enum),
            DocItem::Global(global) => self.globals.push(global),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DocItem {
    Class(Class),
    Table(Table),
    TypeAlias(TypeAlias),
    Enum(Enum),
    Global(Global),
}

impl DocItem {
    pub fn parse(definition: &Definition) -> Result<Option<Self>> {
        let item = match definition.defines.head.define_type {
            DefineType::DocAlias => Some(DocItem::TypeAlias(TypeAlias::parse(definition)?)),
            DefineType::DocClass => Some(DocItem::Class(Class::parse(definition)?)),
            DefineType::DocEnum => Some(DocItem::Enum(Enum::parse(definition)?)),
            DefineType::SetGlobal => {
                let extends = definition.defines.head.extends.first()
                    .ok_or_else(||anyhow!("expected extends for setglobal"))?;

                match extends.extends_type {
                    ExtendsType::DocType | ExtendsType::DocExtendsName =>
                        bail!("unexpected doc extend for setglobal"),
                    ExtendsType::Table => Some(DocItem::Table(Table::parse(definition)?)),
                    _ => Some(DocItem::Global(Global::parse(definition)?)),
                }
            },
            // The other define types extend another doc item
            _ => None,
        };

        Ok(item)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Class {
    name: String,
    description: Option<String>,
    fields: Vec<Field>,
    methods: Vec<Method>,
}

impl Class {
    pub fn parse(definition: &Definition) -> Result<Self> {
        ensure!(definition.definition_type == DefinitionType::Type);
        ensure!(definition.defines.head.define_type == DefineType::DocClass);

        let fields: Vec<Field> = definition.fields
            .iter()
            .filter(|f| f.field_type == FieldType::DocField ||
                f.field_type == FieldType::SetField ||
                f.field_type == FieldType::SetMethod
            )
            .map(|f| Field::parse(f))
            .collect::<Result<Vec<Field>>>()?;

        let methods: Vec<Method> = definition.fields
            .iter()
            .filter(|f| f.field_type == FieldType::SetMethod)
            .map(|f| Method::parse(f))
            .collect::<Result<Vec<Method>>>()?;

        let class = Self {
            name: definition.name.clone(),
            description: definition.rawdesc.clone(),
            fields,
            methods,
        };

        Ok(class)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Table {
    name: String,
    description: Option<String>, 
    fields: Vec<Field>,
}

impl Table {
    pub fn parse(definition: &Definition) -> Result<Self> {
        ensure!(definition.defines.head.define_type == DefineType::SetGlobal);
        let extends = definition.defines.head.extends.first()
            .ok_or_else(||anyhow!("expected extends for setglobal"))?;
        ensure!(extends.extends_type == ExtendsType::Table);

        Ok(Table {
            name: definition.name.clone(),
            description: definition.rawdesc.clone(),
            fields: Vec::new(), // added later
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeAlias {
    name: String,
    description: Option<String>, 
    #[serde(rename = "type")]
    aliased_type: String,
}

impl TypeAlias {
    pub fn parse(definition: &Definition) -> Result<Self> {
        let define = &definition.defines.head;
        ensure!(define.define_type == DefineType::DocAlias);
        let extends = define.extends.first()
            .ok_or_else(||anyhow!("expected extends for type alias"))?;
        ensure!(extends.extends_type == ExtendsType::DocType);

        Ok(Self {
            name: definition.name.clone(),
            description: definition.rawdesc.clone(),
            aliased_type: extends.view.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Enum {
    name: String,
    description: Option<String>, 
    fields: Vec<Field>,
}

impl Enum {
    pub fn parse(definition: &Definition) -> Result<Self> {
        let define = &definition.defines.head;
        ensure!(define.define_type == DefineType::DocEnum);

        Ok({Self {
            name: definition.name.clone(),
            description: definition.rawdesc.clone(),
            fields: Vec::new(),
        }})
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Global {
    name: String,
    description: Option<String>, 
    global_type: GlobalType,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GlobalType {
    Primitive(String),
    Function(Function),
}

impl Global {
    pub fn parse(definition: &Definition) -> Result<Self> {
        ensure!(definition.defines.head.define_type == DefineType::SetGlobal);
        let extends = definition.defines.head.extends.first()
            .ok_or_else(||anyhow!("expected extends for setglobal"))?;
        let extends_primitive = 
            extends.extends_type == ExtendsType::Integer ||
            extends.extends_type == ExtendsType::Nil ||
            extends.extends_type == ExtendsType::Number ||
            extends.extends_type == ExtendsType::String ||
            extends.extends_type == ExtendsType::Binary;
        ensure!(extends_primitive || extends.extends_type == ExtendsType::Function);

        let global_type = match extends.extends_type {
            ExtendsType::Binary | ExtendsType::Integer | ExtendsType::Nil | ExtendsType::Number | ExtendsType::String => GlobalType::Primitive(extends.view.clone()),
            ExtendsType::Function => GlobalType::Function(Function::parse(extends)?),
            _ => bail!("unexpected extends type {:?}", extends.extends_type)
        };

        Ok({Self {
            name: definition.name.clone(),
            description: definition.rawdesc.clone(),
            global_type,
        }})
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Field {
    name: String,
    description: Option<String>,
    #[serde(rename = "type")]
    lua_type: String,
}

impl Field {
    pub fn parse(field: &json::Field) -> Result<Self> {
        ensure!(field.field_type == FieldType::DocField ||
                field.field_type == FieldType::SetField ||
                field.field_type == FieldType::SetMethod
        );

        Ok(Field {
            name: field.name.clone(),
            description: field.rawdesc.clone(),
            lua_type: field.extends.view.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Function {
    description: Option<String>,
    view: String,
    arguments: Vec<Argument>,
    returns: Vec<Return>,
}

impl Function {
    pub fn parse(extends: &Extends) -> Result<Self> {
        ensure!(extends.extends_type == ExtendsType::Function);

        let arguments = extends.args
            .iter()
            .map(|arg| Argument::parse(arg))
            .collect::<Result<Vec<Argument>>>()?;

        let returns = extends.returns
            .iter()
            .map(|ret| Return::parse(ret))
            .collect::<Result<Vec<Return>>>()?;

        Ok(Self {
            description: extends.rawdesc.clone(),
            view: extends.view.clone(),
            arguments,
            returns,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Method {
    name: String,
    function: Function,
}

impl Method {
    pub fn parse(field: &json::Field) -> Result<Self> {
        ensure!(field.field_type == FieldType::SetMethod);
        ensure!(field.extends.extends_type == ExtendsType::Function);
        
        Ok(Method {
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
    pub arg_type: ArgumentType,
}

impl Argument {
    pub fn parse(arg: &json::FuncArg) -> Result<Self> {
        let arg_type = match arg.arg_type {
            ArgType::DocType => ArgumentType::DocType(arg.view.clone()),
            ArgType::Local => ArgumentType::Local(arg.view.clone()),
            ArgType::SelfType => ArgumentType::SelfType,
            ArgType::VarArg => ArgumentType::VarArg,
        };

        Ok(Self {
            name: arg.name.clone(),
            description: arg.rawdesc.clone(),
            arg_type,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArgumentType {
    DocType(String),
    Local(String),
    #[serde(rename = "self")]
    SelfType,
    #[serde(rename = "...")]
    VarArg,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Return {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub return_type: String,
    pub description: Option<String>,
}

impl Return{
    pub fn parse(ret: &json::FuncReturn) -> Result<Self> {
        Ok(Self {
            name: ret.name.clone(),
            description: ret.rawdesc.clone(),
            return_type: ret.view.clone(),
        })
    }
}