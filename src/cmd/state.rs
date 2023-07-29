use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use tf_provider::{
    map,
    value::{self, ValueBool},
    Attribute, AttributeConstraint, AttributeType, Block, Description, NestedBlock, Schema, Value,
    ValueList, ValueMap, ValueNumber, ValueSet, ValueString,
};

use crate::{
    connection::Connection,
    utils::{WithCmd, WithEnv, WithRead, WithSchema},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceState<'a, T>
where
    T: Connection,
{
    #[serde(borrow = "'a")]
    pub id: ValueString<'a>,
    pub inputs: ValueMap<'a, ValueString<'a>>,
    pub state: ValueMap<'a, ValueString<'a>>,
    pub read: ValueMap<'a, Value<StateRead<'a>>>,
    #[serde(with = "value::serde_as_vec")]
    pub create: Value<StateCreate<'a>>,
    #[serde(with = "value::serde_as_vec")]
    pub destroy: Value<StateDestroy<'a>>,
    pub update: ValueList<Value<StateUpdate<'a>>>,
    #[serde(with = "value::serde_as_vec")]
    pub connect: Value<T::Config<'a>>,
    pub command_concurrency: ValueNumber,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataSourceState<'a, T>
where
    T: Connection,
{
    #[serde(borrow = "'a")]
    pub inputs: ValueMap<'a, ValueString<'a>>,
    pub outputs: ValueMap<'a, ValueString<'a>>,
    pub read: ValueMap<'a, Value<StateRead<'a>>>,
    #[serde(with = "value::serde_as_vec")]
    pub connect: Value<T::Config<'a>>,
    pub command_concurrency: ValueNumber,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StateCmd<'a> {
    #[serde(borrow = "'a")]
    pub cmd: ValueString<'a>,
    pub dir: ValueString<'a>,
    pub env: ValueMap<'a, ValueString<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StateUpdate<'a> {
    #[serde(borrow = "'a")]
    #[serde(flatten)]
    pub cmd: StateCmd<'a>,
    pub triggers: ValueSet<ValueString<'a>>,
    pub reloads: ValueSet<ValueString<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StateRead<'a> {
    #[serde(borrow = "'a")]
    #[serde(flatten)]
    pub cmd: StateCmd<'a>,
    pub faillible: ValueBool,
    pub strip_trailing_newline: ValueBool,
}

pub type StateCreate<'a> = StateCmd<'a>;
pub type StateDestroy<'a> = StateCmd<'a>;

lazy_static! {
    static ref CMD_ATTRIBUTE: Attribute = Attribute {
        attr_type: AttributeType::String,
        description: Description::plain("Command to execute when reading the attribute"),
        constraint: AttributeConstraint::Required,
        ..Default::default()
    };
    static ref DIR_ATTRIBUTE: Attribute = Attribute {
        attr_type: AttributeType::String,
        description: Description::plain("Directory where the command will be executed"),
        constraint: AttributeConstraint::Optional,
        ..Default::default()
    };
    static ref ENV_ATTRIBUTE: Attribute = Attribute {
        attr_type: AttributeType::Map(AttributeType::String.into()),
        description: Description::plain("Environment used to execute the command"),
        constraint: AttributeConstraint::Optional,
        ..Default::default()
    };
    static ref READ_BLOCK: NestedBlock = NestedBlock::Map(Block {
        attributes: map! {
            "cmd" => CMD_ATTRIBUTE.clone(),
            "dir" => DIR_ATTRIBUTE.clone(),
            "env" => ENV_ATTRIBUTE.clone(),
            "faillible" => Attribute {
                attr_type: AttributeType::Bool,
                description: Description::plain(
                    "Whether a command should be a hard error",
                ),
                constraint: AttributeConstraint::Optional,
                ..Default::default()
            },
            "strip_trailing_newline" => Attribute {
                attr_type: AttributeType::Bool,
                description: Description::plain(
                    "When enabled, remove the trailing newline if present",
                ),
                constraint: AttributeConstraint::Optional,
                ..Default::default()
            },
        },
        description: Description::plain("Command to execute to get the value of the output",),
        ..Default::default()
    });
}

impl<'a, T> WithSchema for ResourceState<'a, T>
where
    T: Connection,
{
    fn schema() -> Schema {
        Schema {
            version: 1,
            block: Block {
                version: 1,
                attributes: map! {
                    "id" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("Random id for the command"),
                        constraint: AttributeConstraint::Computed,
                        ..Default::default()
                    },
                    "inputs" => Attribute {
                        attr_type: AttributeType::Map(AttributeType::String.into()),
                        description: Description::plain("Inputs to the commands"),
                        constraint: AttributeConstraint::OptionalComputed,
                        ..Default::default()
                    },
                    "state" => Attribute {
                        attr_type: AttributeType::Map(AttributeType::String.into()),
                        description: Description::plain("State of the resource"),
                        constraint: AttributeConstraint::Computed,
                        ..Default::default()
                    },
                    "command_concurrency" => Attribute {
                        attr_type: AttributeType::Number,
                        description: Description::plain("Number of conccurent commands spawned in parallel"),
                        constraint: AttributeConstraint::Optional,
                        ..Default::default()
                    },
                },
                blocks: map! {
                    "read" => READ_BLOCK.clone(),
                    "create" => NestedBlock::Optional(Block {
                        attributes: map! {
                            "cmd" => CMD_ATTRIBUTE.clone(),
                            "dir" => DIR_ATTRIBUTE.clone(),
                            "env" => ENV_ATTRIBUTE.clone(),
                        },
                        description: Description::plain(
                            "Command to execute to create the resource",
                        ),
                        ..Default::default()
                    }),
                    "destroy" => NestedBlock::Optional(Block {
                        attributes: map! {
                            "cmd" => CMD_ATTRIBUTE.clone(),
                            "dir" => DIR_ATTRIBUTE.clone(),
                            "env" => ENV_ATTRIBUTE.clone(),
                        },
                        description: Description::plain(
                            "Command to execute to destroy the resource",
                        ),
                        ..Default::default()
                    }),
                    "update" => NestedBlock::Set(Block {
                        attributes: map! {
                            "cmd" => CMD_ATTRIBUTE.clone(),
                            "dir" => DIR_ATTRIBUTE.clone(),
                            "env" => ENV_ATTRIBUTE.clone(),
                            "triggers" => Attribute {
                                attr_type: AttributeType::Set(AttributeType::String.into()),
                                description: Description::plain(
                                    "What input changes should trigger this update",
                                ),
                                constraint: AttributeConstraint::Optional,
                                ..Default::default()
                            },
                            "reloads" => Attribute {
                                attr_type: AttributeType::Set(AttributeType::String.into()),
                                description: Description::plain(
                                    "What outputs should be read again after this update",
                                ),
                                constraint: AttributeConstraint::Optional,
                                ..Default::default()
                            },
                        },
                        description: Description::plain(
                            "Command to execute when an input changes",
                        ),
                        ..Default::default()
                    }),
                    "connect" => NestedBlock::Optional(Block {
                        attributes: T::schema(),
                        description: Description::plain("Connection configuration"),
                        ..Default::default()
                    }),
                },
                description: Description::plain("Custom resource managed with local commands"),
                deprecated: false,
            },
        }
    }
}

impl<'a, T> WithSchema for DataSourceState<'a, T>
where
    T: Connection,
{
    fn schema() -> Schema {
        Schema {
            version: 1,
            block: Block {
                version: 1,
                attributes: map! {
                    "inputs" => Attribute {
                        attr_type: AttributeType::Map(AttributeType::String.into()),
                        description: Description::plain("Inputs to the commands"),
                        constraint: AttributeConstraint::OptionalComputed,
                        ..Default::default()
                    },
                    "outputs" => Attribute {
                        attr_type: AttributeType::Map(AttributeType::String.into()),
                        description: Description::plain("Outputs to the commands"),
                        constraint: AttributeConstraint::Computed,
                        ..Default::default()
                    },
                    "command_concurrency" => Attribute {
                        attr_type: AttributeType::Number,
                        description: Description::plain("Number of conccurent commands spawned in parallel"),
                        constraint: AttributeConstraint::Optional,
                        ..Default::default()
                    },
                },
                blocks: map! {
                    "read" => READ_BLOCK.clone(),
                    "connect" => NestedBlock::Optional(Block {
                        attributes: T::schema(),
                        description: Description::plain("Connection information"),
                        ..Default::default()
                    }),
                },
                description: Description::plain("Custom resource managed with local commands"),
                deprecated: false,
            },
        }
    }
}

impl<'a> WithCmd for StateCmd<'a> {
    fn cmd(&self) -> &str {
        self.cmd.as_str()
    }

    fn dir(&self) -> &str {
        self.dir.as_str()
    }
}
impl<'a> WithCmd for StateUpdate<'a> {
    fn cmd(&self) -> &str {
        self.cmd.cmd()
    }
    fn dir(&self) -> &str {
        self.cmd.dir()
    }
}
impl<'a> WithCmd for StateRead<'a> {
    fn cmd(&self) -> &str {
        self.cmd.cmd()
    }
    fn dir(&self) -> &str {
        self.cmd.dir()
    }
}
impl<'a> WithRead for StateRead<'a> {
    fn strip_trailing_newline(&self) -> bool {
        self.strip_trailing_newline.unwrap_or(true)
    }
    fn faillible(&self) -> bool {
        self.faillible.unwrap_or(false)
    }
}

impl<'a> WithEnv for StateCmd<'a> {
    type Env = ValueMap<'a, ValueString<'a>>;

    fn env(&self) -> &Self::Env {
        &self.env
    }
}
impl<'a> WithEnv for StateUpdate<'a> {
    type Env = ValueMap<'a, ValueString<'a>>;

    fn env(&self) -> &Self::Env {
        &self.cmd.env
    }
}
impl<'a> WithEnv for StateRead<'a> {
    type Env = ValueMap<'a, ValueString<'a>>;

    fn env(&self) -> &Self::Env {
        &self.cmd.env
    }
}
