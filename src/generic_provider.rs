use async_trait::async_trait;

use tf_provider::{map, Block, Description, Provider, Schema, ValueEmpty};

use crate::{
    cmd::{GenericCmdDataSource, GenericCmdResource},
    connection::{local::ConnectionLocal, ssh::ConnectionSsh},
    file::{GenericFileDataSource, GenericFileResource},
};

#[derive(Debug, Default, Clone)]
pub struct GenericProvider {}

#[async_trait]
impl Provider for GenericProvider {
    type Config<'a> = ValueEmpty;
    type MetaState<'a> = ValueEmpty;

    fn schema(&self, _diags: &mut tf_provider::Diagnostics) -> Option<tf_provider::Schema> {
        Some(Schema {
            version: 1,
            block: Block {
                description: Description::plain("generic"),
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
            "local_cmd" => GenericCmdResource::new(ConnectionLocal::default()),
            "ssh_cmd"   => GenericCmdResource::new(ConnectionSsh::default()),
            "local_file" => GenericFileResource::new(false, ConnectionLocal::default()),
            "ssh_file"   => GenericFileResource::new(false, ConnectionSsh::default()),
            "local_sensitive_file" => GenericFileResource::new(true, ConnectionLocal::default()),
            "ssh_sensitive_file"   => GenericFileResource::new(true, ConnectionSsh::default()),
        })
    }

    fn get_data_sources(
        &self,
        _diags: &mut tf_provider::Diagnostics,
    ) -> Option<
        std::collections::HashMap<String, Box<dyn tf_provider::data_source::DynamicDataSource>>,
    > {
        Some(map! {
            "local_cmd" => GenericCmdDataSource::new(ConnectionLocal::default()),
            "ssh_cmd"   => GenericCmdDataSource::new(ConnectionSsh::default()),
            "local_file" => GenericFileDataSource::new(false, ConnectionLocal::default()),
            "ssh_file"   => GenericFileDataSource::new(false, ConnectionSsh::default()),
            "local_sensitive_file" => GenericFileDataSource::new(true, ConnectionLocal::default()),
            "ssh_sensitive_file"   => GenericFileDataSource::new(true, ConnectionSsh::default()),
        })
    }
}
