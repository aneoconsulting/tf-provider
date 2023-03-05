use std::collections::HashMap;

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
}

impl Block {
    /// Create an empty block (with a version of 1, and a description "empty")
    pub fn empty() -> Block {
        Block {
            version: 1,
            attributes: Default::default(),
            blocks: Default::default(),
            description: "empty".into(),
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
    /// Nested attribute
    AttributeSingle(HashMap<String, Attribute>),
    /// List of nested attribute
    AttributeList(HashMap<String, Attribute>),
    /// Set of nested attribute
    AttributeSet(HashMap<String, Attribute>),
    /// Map of nested attribute
    AttributeMap(HashMap<String, Attribute>),
    /// Dynamic (serialized into a json pair `[type, value]`)
    Any,
}

/*
impl Display for AttributeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeType::String => f.write_str("\"string\""),
            AttributeType::Number => f.write_str("\"number\""),
            AttributeType::Bool => f.write_str("\"bool\""),
            AttributeType::List(attr_type) => {
                f.write_fmt(format_args!("[\"list\", {}]", *attr_type))
            }
            AttributeType::Set(attr_type) => {
                f.write_fmt(format_args!("[\"set\", {}]", *attr_type))
            },
            AttributeType::Map(attr_type) => {
                f.write_fmt(format_args!("[\"map\", {}]", *attr_type))
            },
            AttributeType::Object(attr_types) => {
                f.write_str("[\"object\"")?;
                for (k, v) in attr_types {
                    f.write_fmt(format_args!(", {}", *attr_type))
                }
                f.write_fmt(format_args!("[\"list\", {}]", *attr_type))
            },
            AttributeType::Tuple(attr_types) => {
                f.write_fmt(format_args!("[\"list\", {}]", *attr_type))
            },
            AttributeType::Any => f.write_str("\"dynamic\""),
        }
    }
}
*/

/// Attribute
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Attribute {
    /// Type of the Attribute
    pub attr_type: AttributeType,
    /// Description of the Attribute
    pub description: Description,
    /// Is the attribute required
    pub required: bool,
    /// Is the attribute optional
    pub optional: bool,
    /// Is the attribute computed
    pub computed: bool,
    /// Is the attribute sensitive
    pub sensitive: bool,
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
