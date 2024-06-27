//!Type definitions for the JSON data types used by the LuaLS `doc.json` files.
//! https://luals.github.io/wiki/export-docs/

use std::{fmt, marker::PhantomData};

use serde::{de::{self, MapAccess, Visitor}, Deserialize, Deserializer, Serialize};
use nonempty::NonEmpty;

use crate::location::{Location, Range};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Definition {
    Type(TypeDefinition),
    Variable(VarDefinition),
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeDefinition {
    pub name: String,
    pub desc: Option<String>,
    pub rawdesc: Option<String>,
    pub defines: NonEmpty<TypeDefine>,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VarDefinition {
    pub name: String,
    pub desc: Option<String>,
    pub rawdesc: Option<String>,
    pub defines: NonEmpty<VarDefine>,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeDefine {
    #[serde(rename = "type")]
    pub define_type: TypeDefineType,
    #[serde(flatten)]
    pub location: Location,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_extends")]
    pub extends: Vec<TypeExtends>,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VarDefine {
    #[serde(rename = "type")]
    pub define_type: VarDefineType,
    #[serde(flatten)]
    pub location: Location,
    pub extends: VarExtends,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VarDefineType {
    SetGlobal,
    SetField,
    SetMethod,
    SetIndex,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeDefineType {
    #[serde(rename = "doc.alias")]
    DocAlias,
    #[serde(rename = "doc.class")]
    DocClass,
    #[serde(rename = "doc.enum")]
    DocEnum,
    #[serde(rename = "doc.field")]
    DocField,
    #[serde(rename = "doc.type")]
    DocType,
    TableField,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VarExtends {
    #[serde(flatten)]
    pub range: Range,
    #[serde(rename = "type")]
    pub lua_type: VarType,
    pub view: String,
    pub desc: Option<String>,
    pub rawdesc: Option<String>,
    #[serde(rename = "async")]
    pub is_async: Option<bool>,
    pub deprecated: Option<bool>,
    /// Only present for functions (type = "function") with args
    #[serde(default)]
    pub args: Vec<FuncArg>,
    /// Only present for functions (type = "function") with returns
    #[serde(default)]
    pub returns: Vec<FuncReturn>,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeExtends {
    #[serde(flatten)]
    pub range: Range,
    #[serde(rename = "type")]
    pub lua_type: TypeExtendsType,
    pub view: String,
    pub desc: Option<String>,
    pub rawdesc: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VarType {
    #[serde(rename = "doc.type")]
    DocType,
    Binary,
    Function,
    Integer,
    Nil,
    Number,
    String,
    Table,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeExtendsType {
    #[serde(rename = "doc.extends.name")]
    DocExtendsName,
    #[serde(rename = "doc.type")]
    DocType,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub desc: Option<String>,
    pub rawdesc: Option<String>,
    #[serde(flatten)]
    pub location: Location,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    pub visible: Option<Visibility>,
    #[serde(rename = "async")]
    pub is_async: Option<bool>,
    pub deprecated: Option<bool>,
    pub extends: VarExtends,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    #[serde(rename = "doc.field")]
    DocField,
    SetMethod,
    SetField,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Protected,
    Private,
    Package,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct FuncArg {
    /// The name is missing for varargs ("...")
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub arg_type: ArgType,
    pub desc: Option<String>,
    pub rawdesc: Option<String>,
    pub view: String,
    #[serde(flatten)]
    pub range: Range,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArgType {
    #[serde(rename = "doc.type")]
    DocType,
    Binary,
    Function,
    Integer,
    Local,
    Nil,
    Number,
    #[serde(rename = "self")]
    SelfType,
    String,
    Table,
    #[serde(rename = "...")]
    VarArg,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct FuncReturn {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub return_type: ReturnType,
    pub view: String,
    pub desc: Option<String>,
    pub rawdesc: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReturnType {
    #[serde(rename = "doc.type")]
    DocType,
    Binary,
    Function,
    #[serde(rename = "function.return")]
    FuncReturn,
    Local,
    Nil,
    Number,
    String,
    Table,
}

/// Implement the value of "extends", which may be missing, null, an array
/// of maps, or a single map. We always deserialize into a vector of maps (which
/// may be empty) for consistency.
fn deserialize_extends<'de, D>(deserializer: D) -> Result<Vec<TypeExtends>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ExtendData(PhantomData<fn() -> Vec<ExtendData>>);

    impl<'de> Visitor<'de> for ExtendData
    {
        type Value = Vec<TypeExtends>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("array or map or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error, { 
            Ok(Vec::new())
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>, { 
            Ok(Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))?)
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            Ok(vec![Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))?])
        }
    }

    deserializer.deserialize_any(ExtendData(PhantomData))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::error::Error;

    #[test]
    fn parse_json() -> Result<(), Box<dyn Error>> {
        let data = include_str!("../testdata/doc.json");

        let docs: Vec<Definition> = serde_json::from_str(data)?;

        assert!(docs.len() >= 1);

        Ok(())
    }
}