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

use std::collections::HashMap;

use crate::data_source::DynamicDataSource;
use crate::diagnostics::Diagnostics;
use crate::raw::RawValue;
use crate::resource::DynamicResource;
use crate::schema::Schema;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Trait for implementing a provider
#[async_trait]
pub trait Provider: Send + Sync + 'static {
    /// Configuration of the provider
    type Config<'a>: Serialize + Deserialize<'a> + Send;
    /// State of the provider metadata
    type MetaState<'a>: Serialize + Deserialize<'a> + Send;

    /// Get the schema of the provider
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema>;

    /// Validate the configuration of the provider
    async fn validate<'a>(&self, diags: &mut Diagnostics, config: Self::Config<'a>) -> Option<()>;

    /// Configure the provider
    async fn configure<'a>(
        &self,
        diags: &mut Diagnostics,
        terraform_version: String,
        config: Self::Config<'a>,
    ) -> Option<()>;

    /// Get the schema for the provider metadata (defaults to empty)
    fn meta_schema(&self, diags: &mut Diagnostics) -> Option<Schema> {
        _ = diags;
        Some(Schema {
            version: 1,
            block: Default::default(),
        })
    }

    /// Get the resources of the provider
    fn get_resources(
        &self,
        diags: &mut Diagnostics,
    ) -> Option<HashMap<String, Box<dyn DynamicResource>>>;

    /// Get the data sources of the provider
    fn get_data_sources(
        &self,
        diags: &mut Diagnostics,
    ) -> Option<HashMap<String, Box<dyn DynamicDataSource>>>;
}

#[async_trait]
pub trait DynamicProvider: Send + Sync + 'static {
    /// Get the schema of the provider
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema>;

    /// Validate the configuration of the provider
    async fn validate(&self, diags: &mut Diagnostics, config: RawValue) -> Option<()>;

    /// Configure the provider
    async fn configure(
        &self,
        diags: &mut Diagnostics,
        terraform_version: String,
        config: RawValue,
    ) -> Option<()>;

    /// Get the schema for the provider metadata (defaults to empty)
    fn meta_schema(&self, diags: &mut Diagnostics) -> Option<Schema> {
        _ = diags;
        Some(Schema {
            version: 1,
            block: Default::default(),
        })
    }

    /// Get the resources of the provider
    fn get_resources(
        &self,
        diags: &mut Diagnostics,
    ) -> Option<HashMap<String, Box<dyn DynamicResource>>>;

    /// Get the data sources of the provider
    fn get_data_sources(
        &self,
        diags: &mut Diagnostics,
    ) -> Option<HashMap<String, Box<dyn DynamicDataSource>>>;
}

#[async_trait]
impl<T: Provider> DynamicProvider for T {
    /// Get the schema of the provider
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema> {
        <T as Provider>::schema(self, diags)
    }

    /// Validate the configuration of the provider
    async fn validate(&self, diags: &mut Diagnostics, config: RawValue) -> Option<()> {
        let config = config.deserialize(diags)?;
        <T as Provider>::validate(self, diags, config).await
    }

    /// Configure the provider
    async fn configure(
        &self,
        diags: &mut Diagnostics,
        terraform_version: String,
        config: RawValue,
    ) -> Option<()> {
        let config = config.deserialize(diags)?;
        <T as Provider>::configure(self, diags, terraform_version, config).await
    }

    /// Get the schema for the provider metadata (defaults to empty)
    fn meta_schema(&self, diags: &mut Diagnostics) -> Option<Schema> {
        <T as Provider>::meta_schema(self, diags)
    }

    /// Get the resources of the provider
    fn get_resources(
        &self,
        diags: &mut Diagnostics,
    ) -> Option<HashMap<String, Box<dyn DynamicResource>>> {
        <T as Provider>::get_resources(self, diags)
    }

    /// Get the data sources of the provider
    fn get_data_sources(
        &self,
        diags: &mut Diagnostics,
    ) -> Option<HashMap<String, Box<dyn DynamicDataSource>>> {
        <T as Provider>::get_data_sources(self, diags)
    }
}

impl<T: Provider + 'static> From<T> for Box<dyn DynamicProvider> {
    fn from(value: T) -> Self {
        Box::new(value)
    }
}
