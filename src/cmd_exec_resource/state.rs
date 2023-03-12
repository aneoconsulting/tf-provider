use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use tf_provider::{
    map, value, Attribute, AttributeConstraint, AttributeType, Block, Description, NestedBlock,
    Schema, Value, ValueList, ValueMap, ValueString,
};

use crate::{connection::Connection, utils::WithSchema};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State<T>
where
    T: Connection,
    T: Serialize,
    T: for<'a> Deserialize<'a>,
{
    pub id: ValueString,
    pub inputs: ValueMap<ValueString>,
    pub state: ValueMap<ValueString>,
    pub read: ValueMap<Value<StateRead>>,
    #[serde(with = "value::serde_as_vec")]
    pub create: Value<StateCreate>,
    #[serde(with = "value::serde_as_vec")]
    pub destroy: Value<StateDestroy>,
    pub update: ValueList<Value<StateUpdate>>,
    #[serde(with = "value::serde_as_vec")]
    pub connection: Value<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateCmd {
    pub cmd: ValueString,
    pub env: ValueMap<ValueString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateUpdate {
    #[serde(flatten)]
    pub cmd: StateCmd,
    pub triggers: ValueMap<ValueString>,
    pub reloads: ValueMap<ValueString>,
}

pub type StateRead = StateCmd;
pub type StateCreate = StateCmd;
pub type StateDestroy = StateCmd;

impl<T> WithSchema for State<T>
where
    T: Connection,
    T: Serialize,
    T: for<'a> Deserialize<'a>,
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
                        description: Description::plain("Execute command locally"),
                        constraint: AttributeConstraint::OptionalComputed,
                        ..Default::default()
                    },
                    "state" => Attribute {
                        attr_type: AttributeType::Map(AttributeType::String.into()),
                        description: Description::plain("State of the resource"),
                        constraint: AttributeConstraint::Computed,
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
                                attr_type: AttributeType::Map(AttributeType::String.into()),
                                description: Description::plain(
                                    "What input changes should trigger this update",
                                ),
                                constraint: AttributeConstraint::Optional,
                                ..Default::default()
                            },
                            "reloads" => Attribute {
                                attr_type: AttributeType::Map(AttributeType::String.into()),
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
                    "connection" => NestedBlock::Optional(Block {
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
