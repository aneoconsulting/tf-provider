use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tf_provider::{
    resource::DynamicResource,
    schema::{Block, Description},
    value::Value,
    Provider, Schema,
};

use crate::cmd_resource::CmdResource;

#[derive(Debug, Default)]
pub struct CmdProvider {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderConfig {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderMeta {}

impl Provider for CmdProvider {
    type Config = Value<ProviderConfig>;

    type MetaState = Value<ProviderMeta>;

    fn schema(&self, _diags: &mut tf_provider::Diagnostics) -> Option<tf_provider::Schema> {
        Some(Schema {
            version: 1,
            block: Block {
                version: 1,
                attributes: Default::default(),
                blocks: Default::default(),
                description: Description::plain("cmd"),
                deprecated: false,
            },
        })
    }

    fn validate(&self, _diags: &mut tf_provider::Diagnostics, _config: Self::Config) -> Option<()> {
        Some(())
    }

    fn configure(
        &self,
        _diags: &mut tf_provider::Diagnostics,
        _terraform_version: String,
        _config: Self::Config,
    ) -> Option<()> {
        Some(())
    }

    fn get_resources(
        &self,
        _diags: &mut tf_provider::Diagnostics,
    ) -> Option<std::collections::HashMap<String, Box<dyn tf_provider::resource::DynamicResource>>>
    {
        let mut resources = HashMap::<String, Box<dyn DynamicResource>>::default();
        resources.insert("cmd_test".into(), Box::new(CmdResource {}));
        Some(resources)
    }

    fn get_data_sources(
        &self,
        _diags: &mut tf_provider::Diagnostics,
    ) -> Option<
        std::collections::HashMap<String, Box<dyn tf_provider::data_source::DynamicDataSource>>,
    > {
        Some(Default::default())
    }
}