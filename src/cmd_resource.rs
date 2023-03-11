use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use tf_provider::{
    map, Attribute, AttributeConstraint, AttributeType, Block, Description, Diagnostics,
    NestedBlock, Resource, Schema, Value, ValueEmpty, ValueMap, ValueString,
};

use crate::connection::Connection;

pub struct CmdResource {
    pub connection: Box<dyn Connection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State {
    pub id: ValueString,
    pub inputs: ValueMap<ValueString>,
    pub state: ValueMap<ValueString>,
    pub read: HashMap<String, Value<StateRead>>,
    pub create: Value<StateCreate>,
    pub destroy: Value<StateDestroy>,
    pub update: Vec<StateUpdate>,
    pub connection: ValueEmpty,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateCmd {
    pub cmd: ValueString,
    pub env: ValueMap<ValueString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateUpdate {
    pub cmd: ValueString,
    pub env: ValueMap<ValueString>,
    pub triggers: ValueMap<ValueString>,
    pub reloads: ValueMap<ValueString>,
}

pub type StateRead = StateCmd;
pub type StateCreate = StateCmd;
pub type StateDestroy = StateCmd;

#[async_trait]
impl Resource for CmdResource {
    type State = Value<State>;
    type PrivateState = ValueEmpty;
    type ProviderMetaState = ValueEmpty;

    fn schema(&self, _diags: &mut Diagnostics) -> Option<Schema> {
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
        Some(Schema {
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
                    "create" => NestedBlock::Group(Block {
                        attributes: map! {
                            "cmd" => cmd_attribute.clone(),
                            "env" => env_attribute.clone(),
                        },
                        description: Description::plain(
                            "Command to execute to create the resource",
                        ),
                        ..Default::default()
                    }),
                    "destroy" => NestedBlock::Group(Block {
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
                    "connection" => NestedBlock::Group(Block {
                        description: Description::plain("Connection information"),
                        ..Default::default()
                    }),
                },
                description: Description::plain("Custom resource managed with local commands"),
                deprecated: false,
            },
        })
    }

    async fn validate(&self, _diags: &mut Diagnostics, _config: Self::State) -> Option<()> {
        Some(())
    }

    async fn read(
        &self,
        _diags: &mut Diagnostics,
        state: Self::State,
        private_state: Self::PrivateState,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)> {
        Some((state, private_state))
    }

    async fn plan(
        &self,
        _diags: &mut Diagnostics,
        _prior_state: Self::State,
        proposed_state: Self::State,
        _config_state: Self::State,
        prior_private_state: Self::PrivateState,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(
        Self::State,
        Self::PrivateState,
        Vec<tf_provider::attribute_path::AttributePath>,
    )> {
        Some((proposed_state, prior_private_state, vec![]))
        //Some((State::default().into(), prior_private_state, vec![]))
    }

    async fn apply(
        &self,
        _diags: &mut Diagnostics,
        _prior_state: Self::State,
        planned_state: Self::State,
        _config_state: Self::State,
        planned_private_state: Self::PrivateState,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)> {
        Some((planned_state, planned_private_state))
    }
}
