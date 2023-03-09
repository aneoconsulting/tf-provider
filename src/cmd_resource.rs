use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use tf_provider::{
    Attribute, AttributeConstraint, Block, Description, EmptyValue, NestedBlock, Resource, Schema,
    Value,
};

pub struct CmdResource {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State {
    pub dummy: Value<i64>,
    pub dummy2: Value<i64>,
    pub read: HashMap<String, EmptyValue>,
}

#[async_trait]
impl Resource for CmdResource {
    type State = Value<State>;
    type PrivateState = EmptyValue;
    type ProviderMetaState = EmptyValue;

    fn schema(&self, _diags: &mut tf_provider::Diagnostics) -> Option<Schema> {
        Some(Schema {
            version: 1,
            block: Block {
                version: 1,
                attributes: [
                    (
                        "dummy".to_string(),
                        Attribute {
                            attr_type: tf_provider::schema::AttributeType::Number,
                            description: Description::plain("dummy"),
                            constraint: AttributeConstraint::OptionalComputed,
                            sensitive: false,
                            deprecated: false,
                        },
                    ),
                    (
                        "dummy2".to_string(),
                        Attribute {
                            attr_type: tf_provider::schema::AttributeType::Number,
                            description: Description::plain("dummy2"),
                            constraint: AttributeConstraint::Optional,
                            sensitive: false,
                            deprecated: false,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                blocks: [(
                    "read".to_string(),
                    NestedBlock::Map(Block {
                        version: 1,
                        attributes: Default::default(),
                        blocks: Default::default(),
                        description: Description::plain("read"),
                        deprecated: false,
                    }),
                )]
                .into_iter()
                .collect(),
                description: Description::plain("cmd_test"),
                deprecated: false,
            },
        })
    }

    async fn validate(
        &self,
        _diags: &mut tf_provider::Diagnostics,
        _config: Self::State,
    ) -> Option<()> {
        Some(())
    }

    async fn read(
        &self,
        _diags: &mut tf_provider::Diagnostics,
        state: Self::State,
        private_state: Self::PrivateState,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)> {
        Some((state, private_state))
    }

    async fn plan(
        &self,
        _diags: &mut tf_provider::Diagnostics,
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
        _diags: &mut tf_provider::Diagnostics,
        _prior_state: Self::State,
        planned_state: Self::State,
        _config_state: Self::State,
        planned_private_state: Self::PrivateState,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)> {
        Some((planned_state, planned_private_state))
    }
}
