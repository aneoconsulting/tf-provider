use std::{collections::HashMap, fmt::Display, hash::Hash};

use crate::tfplugin6;

/// Specify if a description must interpreted as markdown or plain
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum StringKind {
    #[default]
    Plain = 0,
    Markdown = 1,
}

/// Description
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Description {
    /// Kind of the description (either `Plain` or `Markdown`)
    pub kind: StringKind,
    /// Content of the description
    pub content: String,
}

impl Description {
    /// Create a plain description
    pub fn plain<T>(content: T) -> Self
    where
        T: ToString,
    {
        Self {
            kind: StringKind::Plain,
            content: content.to_string(),
        }
    }
    /// Create a markdown description
    pub fn markdown<T>(content: String) -> Self
    where
        T: ToString,
    {
        Self {
            kind: StringKind::Markdown,
            content: content.to_string(),
        }
    }
}

impl<T> From<T> for Description
where
    T: ToString,
{
    fn from(value: T) -> Self {
        Description::plain(value.to_string())
    }
}

/// NestedBlock
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NestedBlock {
    /// The nested block can appear at most once
    Single(Block),
    /// The nested block can appear multiple times
    List(Block),
    /// The nested block can appear multiple times (unordered)
    Set(Block),
    /// The nested block can appear multiple times and have names
    Map(Block),
    /// The nested block can appear at most once (if not given, it will be populate with Nulls)
    Group(Block),
}

/// Block
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Block {
    /// Version of the block
    pub version: i64,
    /// Attributes of the block
    pub attributes: HashMap<String, Attribute>,
    /// Nested blocks of the block
    pub blocks: HashMap<String, NestedBlock>,
    /// Description for the block
    pub description: Description,
    /// Is the block deprecated
    pub deprecated: bool,
}

impl Block {
    /// Create an empty block (with a version of 1, and a description "empty")
    pub fn empty() -> Block {
        Block {
            version: 1,
            attributes: Default::default(),
            blocks: Default::default(),
            description: "empty".into(),
            deprecated: false,
        }
    }
}

fn cvt_nested_blocks_tf6(
    blocks: HashMap<String, NestedBlock>,
) -> ::prost::alloc::vec::Vec<tfplugin6::schema::NestedBlock> {
    use tfplugin6::schema::nested_block::NestingMode;
    blocks
        .into_iter()
        .map(|(name, block)| {
            let (nesting_mode, block) = match block {
                NestedBlock::Single(block) => (NestingMode::Single, block),
                NestedBlock::List(block) => (NestingMode::List, block),
                NestedBlock::Set(block) => (NestingMode::Set, block),
                NestedBlock::Map(block) => (NestingMode::Map, block),
                NestedBlock::Group(block) => (NestingMode::Group, block),
            };
            tfplugin6::schema::NestedBlock {
                type_name: name,
                block: Some(block.into()),
                nesting: nesting_mode as i32,
                min_items: 0,
                max_items: if nesting_mode == NestingMode::Single {
                    1
                } else {
                    i64::MAX
                },
            }
        })
        .collect()
}

#[allow(deprecated)]
fn cvt_attributes_tf6(
    attrs: HashMap<String, Attribute>,
) -> ::prost::alloc::vec::Vec<tfplugin6::schema::Attribute> {
    use tfplugin6::schema::object::NestingMode;
    use tfplugin6::schema::Object;
    attrs
        .into_iter()
        .map(|(name, attr)| {
            let attr_type = attr.attr_type.to_string().into();
            let nested = match attr.attr_type {
                AttributeType::AttributeSingle(attrs) => Some((NestingMode::Single, attrs)),
                AttributeType::AttributeList(attrs) => Some((NestingMode::List, attrs)),
                AttributeType::AttributeSet(attrs) => Some((NestingMode::Set, attrs)),
                AttributeType::AttributeMap(attrs) => Some((NestingMode::Map, attrs)),
                _ => None,
            }
            .map(|(nesting_mode, attrs)| Object {
                attributes: cvt_attributes_tf6(attrs),
                nesting: nesting_mode as i32,
                min_items: 0,
                max_items: if nesting_mode == NestingMode::Single {
                    1
                } else {
                    i64::MAX
                },
            });
            tfplugin6::schema::Attribute {
                name,
                r#type: attr_type,
                nested_type: nested,
                description: attr.description.content,
                required: attr.constraint == AttributeConstraint::Required,
                optional: attr.constraint == AttributeConstraint::OptionalComputed
                    || attr.constraint == AttributeConstraint::Optional,
                computed: attr.constraint == AttributeConstraint::OptionalComputed
                    || attr.constraint == AttributeConstraint::Computed,
                sensitive: attr.sensitive,
                description_kind: match attr.description.kind {
                    StringKind::Plain => tfplugin6::StringKind::Plain,
                    StringKind::Markdown => tfplugin6::StringKind::Markdown,
                } as i32,
                deprecated: attr.deprecated,
            }
        })
        .collect()
}

impl From<Block> for tfplugin6::schema::Block {
    fn from(value: Block) -> Self {
        Self {
            attributes: cvt_attributes_tf6(value.attributes),
            block_types: cvt_nested_blocks_tf6(value.blocks),
            version: value.version,
            description: value.description.content,
            description_kind: match value.description.kind {
                StringKind::Plain => tfplugin6::StringKind::Plain,
                StringKind::Markdown => tfplugin6::StringKind::Markdown,
            } as i32,
            deprecated: value.deprecated,
        }
    }
}

/// Specify the Attribute type
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AttributeType {
    /// String
    String,
    /// Number (int or float)
    Number,
    /// Boolean
    Bool,
    /// List
    List(Box<AttributeType>),
    /// Set
    Set(Box<AttributeType>),
    /// Map
    Map(Box<AttributeType>),
    /// Object
    Object(HashMap<String, AttributeType>),
    /// Tuple
    Tuple(Vec<AttributeType>),
    /// Nested attributes
    AttributeSingle(HashMap<String, Attribute>),
    /// List of nested attributes
    AttributeList(HashMap<String, Attribute>),
    /// Set of nested attributes
    AttributeSet(HashMap<String, Attribute>),
    /// Map of nested attributes
    AttributeMap(HashMap<String, Attribute>),
    /// Dynamic (serialized into a json pair `[type, value]`)
    Any,
}

impl Display for AttributeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String => f.write_str("\"string\""),
            Self::Number => f.write_str("\"number\""),
            Self::Bool => f.write_str("\"bool\""),
            Self::List(attr_type) => f.write_fmt(format_args!("[\"list\",{}]", *attr_type)),
            Self::Set(attr_type) => f.write_fmt(format_args!("[\"set\",{}]", *attr_type)),
            Self::Map(attr_type) => f.write_fmt(format_args!("[\"map\",{}]", *attr_type)),
            Self::Object(attr_types) => {
                f.write_str("[\"object\",{")?;
                let mut sep = "";
                for (k, v) in attr_types {
                    f.write_fmt(format_args!("{}\"{}\":{}", sep, k, v))?;
                    sep = ",";
                }
                f.write_str("}]")
            }
            Self::Tuple(attr_types) => {
                f.write_str("[\"tuple\",[")?;
                let mut sep = "";
                for e in attr_types {
                    f.write_fmt(format_args!("{}{}", sep, e))?;
                    sep = ",";
                }
                f.write_str("]]")
            }
            Self::AttributeSingle(attrs) => {
                f.write_str("[\"object\",{")?;
                let mut sep = "";
                for (k, v) in attrs {
                    f.write_fmt(format_args!("{}\"{}\":{}", sep, k, v.attr_type))?;
                    sep = ",";
                }
                f.write_str("}]")
            }
            Self::AttributeList(attrs) => {
                f.write_str("[\"list\",[\"object\",{")?;
                let mut sep = "";
                for (k, v) in attrs {
                    f.write_fmt(format_args!("{}\"{}\":{}", sep, k, v.attr_type))?;
                    sep = ",";
                }
                f.write_str("}]]")
            }
            Self::AttributeSet(attrs) => {
                f.write_str("[\"set\",[\"object\",{")?;
                let mut sep = "";
                for (k, v) in attrs {
                    f.write_fmt(format_args!("{}\"{}\":{}", sep, k, v.attr_type))?;
                    sep = ",";
                }
                f.write_str("}]]")
            }
            Self::AttributeMap(attrs) => {
                f.write_str("[\"map\",[\"object\",{")?;
                let mut sep = "";
                for (k, v) in attrs {
                    f.write_fmt(format_args!("{}\"{}\":{}", sep, k, v.attr_type))?;
                    sep = ",";
                }
                f.write_str("}]]")
            }
            Self::Any => f.write_str("\"dynamic\""),
        }
    }
}

/// Specify the Attribute type
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum AttributeConstraint {
    /// The attribute is computed, but cannot be specified by the practitioner (output only)
    Computed,
    /// The attribute is optional, and the provider cannot generate a value for it
    Optional,
    /// The attribute is both optional and computed:
    /// the practitioner can omit the value, and the provider can generate a value for it
    OptionalComputed,
    /// The attribute is required
    Required,
}

/// Attribute
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Attribute {
    /// Type of the Attribute
    pub attr_type: AttributeType,
    /// Description of the Attribute
    pub description: Description,
    /// Is the attribute required
    pub constraint: AttributeConstraint,
    /// Is the attribute sensitive
    pub sensitive: bool,
    /// Is the attribute deprecated
    pub deprecated: bool,
}

impl Attribute {}

/// Schema
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Schema {
    /// Version of the schema
    pub version: i64,
    /// Root block of the schema
    pub block: Block,
}

impl From<Schema> for tfplugin6::Schema {
    fn from(value: Schema) -> Self {
        Self {
            version: value.version,
            block: Some(value.block.into()),
        }
    }
}
