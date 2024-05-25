// This file is part of the tf-provider project
//
// Copyright (C) ANEO, 2024-2024. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License")
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::borrow::Cow;

use anyhow::Result;
use async_trait::async_trait;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use tf_provider::{
    map, serve, Attribute, AttributeConstraint, AttributePath, AttributeType, Block, Description,
    Diagnostics, Provider, Resource, Schema, ValueEmpty, ValueMap, ValueString,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NullState<'a> {
    #[serde(borrow = "'a")]
    pub id: ValueString<'a>,
    pub triggers: ValueMap<'a, ValueString<'a>>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NullResource;

#[async_trait]
impl Resource for NullResource {
    type State<'a> = NullState<'a>;
    type PrivateState<'a> = ValueEmpty;
    type ProviderMetaState<'a> = ValueEmpty;

    fn schema(&self, _diags: &mut Diagnostics) -> Option<Schema> {
        Some(Schema {
            version: 1,
            block: Block {
                version: 1,
                description: Description::plain("Pouet"),
                attributes: map! {
                  "id" => Attribute {
                    attr_type: AttributeType::String,
                    description: Description::plain("ID"),
                    constraint: AttributeConstraint::Computed,
                    ..Default::default()
                  },
                  "triggers" => Attribute {
                    attr_type: AttributeType::Map(AttributeType::String.into()),
                    description: Description::plain("ID"),
                    constraint: AttributeConstraint::Computed,
                    ..Default::default()
                  },
                },
                ..Default::default()
            },
        })
    }

    async fn validate<'a>(&self, _diags: &mut Diagnostics, _config: Self::State<'a>) -> Option<()> {
        Some(())
    }

    async fn read<'a>(
        &self,
        _diags: &mut Diagnostics,
        state: Self::State<'a>,
        private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(Self::State<'a>, Self::PrivateState<'a>)> {
        Some((state, private_state))
    }

    async fn plan_create<'a>(
        &self,
        _diags: &mut Diagnostics,
        proposed_state: Self::State<'a>,
        _config_state: Self::State<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(Self::State<'a>, Self::PrivateState<'a>)> {
        let mut state = proposed_state;
        state.id = ValueString::Unknown;

        Some((state, Default::default()))
    }
    async fn plan_update<'a>(
        &self,
        _diags: &mut Diagnostics,
        prior_state: Self::State<'a>,
        proposed_state: Self::State<'a>,
        _config_state: Self::State<'a>,
        prior_private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(
        Self::State<'a>,
        Self::PrivateState<'a>,
        Vec<tf_provider::attribute_path::AttributePath>,
    )> {
        let mut trigger_replace = Vec::new();
        if proposed_state.triggers != prior_state.triggers {
            trigger_replace.push(AttributePath::new("root"));
        }

        Some((proposed_state, prior_private_state, trigger_replace))
    }

    async fn plan_destroy<'a>(
        &self,
        _diags: &mut Diagnostics,
        _prior_state: Self::State<'a>,
        _prior_private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<()> {
        Some(())
    }

    async fn create<'a>(
        &self,
        _diags: &mut Diagnostics,
        planned_state: Self::State<'a>,
        _config_state: Self::State<'a>,
        private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(Self::State<'a>, Self::PrivateState<'a>)> {
        let id = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect::<Cow<'a, str>>();

        let mut state = planned_state;
        state.id = ValueString::from(id);

        Some((state, private_state))
    }
    async fn update<'a>(
        &self,
        _diags: &mut Diagnostics,
        _prior_state: Self::State<'a>,
        planned_state: Self::State<'a>,
        _config_state: Self::State<'a>,
        private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(Self::State<'a>, Self::PrivateState<'a>)> {
        Some((planned_state, private_state))
    }
    async fn destroy<'a>(
        &self,
        _diags: &mut Diagnostics,
        _state: Self::State<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<()> {
        Some(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct NullProvider;

#[async_trait]
impl Provider for NullProvider {
    type Config<'a> = ValueEmpty;
    type MetaState<'a> = ValueEmpty;

    fn schema(&self, _diags: &mut tf_provider::Diagnostics) -> Option<tf_provider::Schema> {
        Some(Schema {
            version: 1,
            block: Block {
                description: Description::plain("null"),
                ..Default::default()
            },
        })
    }

    async fn validate<'a>(
        &self,
        _diags: &mut tf_provider::Diagnostics,
        _config: Self::Config<'a>,
    ) -> Option<()> {
        Some(())
    }

    async fn configure<'a>(
        &self,
        _diags: &mut tf_provider::Diagnostics,
        _terraform_version: String,
        _config: Self::Config<'a>,
    ) -> Option<()> {
        Some(())
    }

    fn get_resources(
        &self,
        _diags: &mut tf_provider::Diagnostics,
    ) -> Option<std::collections::HashMap<String, Box<dyn tf_provider::resource::DynamicResource>>>
    {
        Some(map! {
            "resource" => NullResource,
        })
    }

    fn get_data_sources(
        &self,
        _diags: &mut tf_provider::Diagnostics,
    ) -> Option<
        std::collections::HashMap<String, Box<dyn tf_provider::data_source::DynamicDataSource>>,
    > {
        Some(map! {})
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    serve("null", NullProvider).await
}
