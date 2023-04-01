use async_trait::async_trait;

use tf_provider::{map, Block, Description, Provider, Schema, ValueEmpty};

use crate::{
    cmd_exec::{CmdExecDataSource, CmdExecResource},
    cmd_file::{CmdFileDataSource, CmdFileResource},
    connection::{local::ConnectionLocal, ssh::ConnectionSsh},
};

#[derive(Debug, Default, Clone)]
pub struct CmdProvider {}

#[async_trait]
impl Provider for CmdProvider {
    type Config<'a> = ValueEmpty;
    type MetaState<'a> = ValueEmpty;

    fn schema(&self, _diags: &mut tf_provider::Diagnostics) -> Option<tf_provider::Schema> {
        Some(Schema {
            version: 1,
            block: Block {
                description: Description::plain("cmd"),
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
            "local_exec" => CmdExecResource::new(ConnectionLocal::default()),
            "ssh_exec"   => CmdExecResource::new(ConnectionSsh::default()),
            "local_file" => CmdFileResource::new(false, ConnectionLocal::default()),
            "ssh_file"   => CmdFileResource::new(false, ConnectionSsh::default()),
            "local_sensitive_file" => CmdFileResource::new(true, ConnectionLocal::default()),
            "ssh_sensitive_file"   => CmdFileResource::new(true, ConnectionSsh::default()),
        })
    }

    fn get_data_sources(
        &self,
        _diags: &mut tf_provider::Diagnostics,
    ) -> Option<
        std::collections::HashMap<String, Box<dyn tf_provider::data_source::DynamicDataSource>>,
    > {
        Some(map! {
            "local_exec" => CmdExecDataSource::new(ConnectionLocal::default()),
            "ssh_exec"   => CmdExecDataSource::new(ConnectionSsh::default()),
            "local_file" => CmdFileDataSource::new(false, ConnectionLocal::default()),
            "ssh_file"   => CmdFileDataSource::new(false, ConnectionSsh::default()),
            "local_sensitive_file" => CmdFileDataSource::new(true, ConnectionLocal::default()),
            "ssh_sensitive_file"   => CmdFileDataSource::new(true, ConnectionSsh::default()),
        })
    }
}
