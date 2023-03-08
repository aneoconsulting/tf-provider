use std::collections::HashMap;

use async_trait::async_trait;

use tf_provider::{
    schema::{Block, Description},
    EmptyValue, Provider, Schema,
};

use crate::cmd_resource::CmdResource;

#[derive(Debug, Default)]
pub struct CmdProvider {}

#[async_trait]
impl Provider for CmdProvider {
    type Config = EmptyValue;
    type MetaState = EmptyValue;

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

    async fn validate(
        &self,
        _diags: &mut tf_provider::Diagnostics,
        _config: Self::Config,
    ) -> Option<()> {
        Some(())
    }

    async fn configure(
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
        let mut resources = HashMap::default();
        resources.insert("test".into(), Box::new(CmdResource {}) as _);
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
