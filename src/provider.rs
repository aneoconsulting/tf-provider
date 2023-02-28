use std::collections::HashMap;

pub mod tf {
    tonic::include_proto!("tfplugin6");
}

#[derive(Debug)]
pub struct CmdProvider {}

#[tonic::async_trait]
impl tf::provider_server::Provider for CmdProvider {
    async fn get_provider_schema(
        &self,
        _request: tonic::Request<tf::get_provider_schema::Request>,
    ) -> Result<tonic::Response<tf::get_provider_schema::Response>, tonic::Status> {
        Ok(tonic::Response::new(tf::get_provider_schema::Response {
            provider: Some(tf::Schema {
                version: 1,
                block: Some(tf::schema::Block {
                    version: 1,
                    attributes: vec![],
                    block_types: vec![],
                    description: "cmd".to_string(),
                    description_kind: tf::StringKind::Plain.into(),
                    deprecated: false,
                }),
            }),
            resource_schemas: [(
                "cmd_test".to_string(),
                tf::Schema {
                    version: 1,
                    block: Some(tf::schema::Block {
                        version: 1,
                        attributes: vec![],
                        block_types: vec![tf::schema::NestedBlock {
                            type_name: "read".to_string(),
                            block: Some(tf::schema::Block {
                                version: 1,
                                attributes: vec![],
                                block_types: vec![],
                                description: "cmd".to_string(),
                                description_kind: tf::StringKind::Plain.into(),
                                deprecated: false,
                            }),
                            nesting: tf::schema::nested_block::NestingMode::Map.into(),
                            min_items: 0,
                            max_items: 0,
                        }],
                        description: "cmd_test".to_string(),
                        description_kind: tf::StringKind::Plain.into(),
                        deprecated: false,
                    }),
                },
            )]
            .iter()
            .cloned()
            .collect(),
            data_source_schemas: HashMap::new(),
            diagnostics: vec![],
            provider_meta: Some(tf::Schema {
                version: 1,
                block: Some(tf::schema::Block {
                    version: 1,
                    attributes: vec![],
                    block_types: vec![],
                    description: "cmd".to_string(),
                    description_kind: tf::StringKind::Plain.into(),
                    deprecated: false,
                }),
            }),
            server_capabilities: Some(tf::get_provider_schema::ServerCapabilities {
                plan_destroy: false,
            }),
        }))
    }
    async fn validate_provider_config(
        &self,
        _request: tonic::Request<tf::validate_provider_config::Request>,
    ) -> Result<tonic::Response<tf::validate_provider_config::Response>, tonic::Status> {
        Ok(tonic::Response::new(
            tf::validate_provider_config::Response {
                diagnostics: vec![],
            },
        ))
    }
    async fn validate_resource_config(
        &self,
        _request: tonic::Request<tf::validate_resource_config::Request>,
    ) -> Result<tonic::Response<tf::validate_resource_config::Response>, tonic::Status> {
        Ok(tonic::Response::new(
            tf::validate_resource_config::Response {
                diagnostics: vec![],
            },
        ))
    }
    async fn validate_data_resource_config(
        &self,
        _request: tonic::Request<tf::validate_data_resource_config::Request>,
    ) -> Result<tonic::Response<tf::validate_data_resource_config::Response>, tonic::Status> {
        Ok(tonic::Response::new(
            tf::validate_data_resource_config::Response {
                diagnostics: vec![],
            },
        ))
    }
    async fn upgrade_resource_state(
        &self,
        _request: tonic::Request<tf::upgrade_resource_state::Request>,
    ) -> Result<tonic::Response<tf::upgrade_resource_state::Response>, tonic::Status> {
        unimplemented!();
    }
    /// ////// One-time initialization, called before other functions below
    async fn configure_provider(
        &self,
        _request: tonic::Request<tf::configure_provider::Request>,
    ) -> Result<tonic::Response<tf::configure_provider::Response>, tonic::Status> {
        Ok(tonic::Response::new(tf::configure_provider::Response {
            diagnostics: vec![],
        }))
    }
    /// ////// Managed Resource Lifecycle
    async fn read_resource(
        &self,
        _request: tonic::Request<tf::read_resource::Request>,
    ) -> Result<tonic::Response<tf::read_resource::Response>, tonic::Status> {
        unimplemented!();
    }
    async fn plan_resource_change(
        &self,
        request: tonic::Request<tf::plan_resource_change::Request>,
    ) -> Result<tonic::Response<tf::plan_resource_change::Response>, tonic::Status> {
        Ok(tonic::Response::new(tf::plan_resource_change::Response {
            planned_state: request.get_ref().proposed_new_state.clone(),
            requires_replace: vec![],
            planned_private: vec![],
            diagnostics: vec![],
            legacy_type_system: false,
        }))
    }
    async fn apply_resource_change(
        &self,
        _request: tonic::Request<tf::apply_resource_change::Request>,
    ) -> Result<tonic::Response<tf::apply_resource_change::Response>, tonic::Status> {
        unimplemented!();
    }
    async fn import_resource_state(
        &self,
        _request: tonic::Request<tf::import_resource_state::Request>,
    ) -> Result<tonic::Response<tf::import_resource_state::Response>, tonic::Status> {
        unimplemented!();
    }
    async fn read_data_source(
        &self,
        _request: tonic::Request<tf::read_data_source::Request>,
    ) -> Result<tonic::Response<tf::read_data_source::Response>, tonic::Status> {
        unimplemented!();
    }
    /// ////// Graceful Shutdown
    async fn stop_provider(
        &self,
        _request: tonic::Request<tf::stop_provider::Request>,
    ) -> Result<tonic::Response<tf::stop_provider::Response>, tonic::Status> {
        unimplemented!();
    }
}
