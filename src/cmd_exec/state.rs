use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use tf_provider::{
    map, value, Attribute, AttributeConstraint, AttributeType, Block, Description, NestedBlock,
    Schema, Value, ValueList, ValueMap, ValueNumber, ValueSet, ValueString,
};

use crate::{
    connection::Connection,
    utils::{WithCmd, WithEnv, WithSchema},
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

pub type StateRead<'a> = StateCmd<'a>;
pub type StateCreate<'a> = StateCmd<'a>;
pub type StateDestroy<'a> = StateCmd<'a>;

impl<'a, T> WithSchema for ResourceState<'a, T>
where
    T: Connection,
{
    fn schema() -> Schema {
        let cmd_attribute = Attribute {
            attr_type: AttributeType::String,
            description: Description::plain("Command to execute when reading the attribute"),
            constraint: AttributeConstraint::Required,
            ..Default::default()
        };
        let env_attribute = Attribute {
            attr_type: AttributeType::Map(AttributeType::String.into()),
            description: Description::plain("Environment used to execute the command"),
            constraint: AttributeConstraint::Optional,
            ..Default::default()
        };
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
                    "read" => NestedBlock::Map(Block {
                        attributes: map! {
                            "cmd" => cmd_attribute.clone(),
                            "env" => env_attribute.clone(),
                        },
                        description: Description::plain(
                            "Command to execute to get the value of the output",
                        ),
                        ..Default::default()
                    }),
                    "create" => NestedBlock::Optional(Block {
                        attributes: map! {
                            "cmd" => cmd_attribute.clone(),
                            "env" => env_attribute.clone(),
                        },
                        description: Description::plain(
                            "Command to execute to create the resource",
                        ),
                        ..Default::default()
                    }),
                    "destroy" => NestedBlock::Optional(Block {
                        attributes: map! {
                            "cmd" => cmd_attribute.clone(),
                            "env" => env_attribute.clone(),
                        },
                        description: Description::plain(
                            "Command to execute to destroy the resource",
                        ),
                        ..Default::default()
                    }),
                    "update" => NestedBlock::Set(Block {
                        attributes: map! {
                            "cmd" => cmd_attribute.clone(),
                            "env" => env_attribute.clone(),
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
                    "read" => NestedBlock::Map(Block {
                        attributes: map! {
                            "cmd" => Attribute {
                                attr_type: AttributeType::String,
                                description: Description::plain("Command to execute when reading the attribute"),
                                constraint: AttributeConstraint::Required,
                                ..Default::default()
                            },
                            "env" => Attribute {
                                attr_type: AttributeType::Map(AttributeType::String.into()),
                                description: Description::plain("Environment used to execute the command"),
                                constraint: AttributeConstraint::Optional,
                                ..Default::default()
                            },
                        },
                        description: Description::plain(
                            "Command to execute to get the value of the output",
                        ),
                        ..Default::default()
                    }),
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
}
impl<'a> WithCmd for StateUpdate<'a> {
    fn cmd(&self) -> &str {
        self.cmd.cmd()
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
