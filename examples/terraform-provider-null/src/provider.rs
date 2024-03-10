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

use async_trait::async_trait;

use tf_provider::{map, Block, Description, Provider, Schema, ValueEmpty};

use crate::resource::NullResource;

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
