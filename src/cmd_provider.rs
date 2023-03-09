use std::collections::HashMap;

use async_trait::async_trait;

use tf_provider::{map, Block, Description, Provider, Schema, ValueEmpty};

use crate::cmd_resource::CmdResource;

#[derive(Debug, Default)]
pub struct CmdProvider {}

#[async_trait]
impl Provider for CmdProvider {
    type Config = ValueEmpty;
    type MetaState = ValueEmpty;

    fn schema(&self, _diags: &mut tf_provider::Diagnostics) -> Option<tf_provider::Schema> {
        Some(Schema {
            version: 1,
            block: Block {
                description: Description::plain("cmd"),
                ..Default::default()
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
        Some(map! {
            "local_exec" => CmdResource {},
        })
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
