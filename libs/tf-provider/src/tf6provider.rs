use tonic::codegen::http::request;

use crate::diagnostics::Diagnostics;
use crate::provider::DynamicProvider;
use crate::{resource, tfplugin6 as tf};

#[tonic::async_trait]
impl<T: DynamicProvider> tf::provider_server::Provider for T {
    async fn get_provider_schema(
        &self,
        _request: tonic::Request<tf::get_provider_schema::Request>,
    ) -> Result<tonic::Response<tf::get_provider_schema::Response>, tonic::Status> {
        let mut diags = Diagnostics::default();
        let schema = self
            .schema()
            .map(|schema| schema.into())
            .into_option(&mut diags);
        let meta_schema = self
            .meta_schema()
            .map(|schema| schema.into())
            .into_option(&mut diags);
        let resources = self
            .get_resources()
            .and_then(|resources| {
                crate::result::Result::combine(resources.into_iter().map(|(name, resource)| {
                    resource
                        .schema()
                        .map(|schema| (name.clone(), schema.into()))
                }))
            })
            .into_option(&mut diags)
            .unwrap_or_default();
        let data_sources = self
            .get_data_sources()
            .and_then(|data_sources| {
                crate::result::Result::combine(data_sources.into_iter().map(
                    |(name, data_source)| {
                        data_source
                            .schema()
                            .map(|schema| (name.clone(), schema.into()))
                    },
                ))
            })
            .into_option(&mut diags)
            .unwrap_or_default();

        Ok(tonic::Response::new(tf::get_provider_schema::Response {
            provider: schema,
            resource_schemas: resources,
            data_source_schemas: data_sources,
            diagnostics: diags.into(),
            provider_meta: meta_schema,
            server_capabilities: None,
        }))
    }
    async fn validate_provider_config(
        &self,
        request: tonic::Request<tf::validate_provider_config::Request>,
    ) -> Result<tonic::Response<tf::validate_provider_config::Response>, tonic::Status> {
        let request = request.get_ref();
        let mut diags = Diagnostics::default();

        if let Some(config) = request.config {
            _ = self.validate(config.into()).into_option(&mut diags);
        }
        Ok(tonic::Response::new(
            tf::validate_provider_config::Response {
                diagnostics: diags.into(),
            },
        ))
    }
    async fn validate_resource_config(
        &self,
        request: tonic::Request<tf::validate_resource_config::Request>,
    ) -> Result<tonic::Response<tf::validate_resource_config::Response>, tonic::Status> {
        let request = request.get_ref();
        let mut diags = Diagnostics::default();

        if let Some(resources) = self.get_resources().into_option(&mut diags) {
            if let Some(resource) = resources.get(&request.type_name) {
                if let Some(config) = request.config {
                    _ = resource.validate(config.into()).into_option(&mut diags);
                }
            } else {
                diags.root_error_short(format!("Could not find resource {}", request.type_name));
            }
        }

        Ok(tonic::Response::new(
            tf::validate_resource_config::Response {
                diagnostics: diags.into(),
            },
        ))
    }
    async fn validate_data_resource_config(
        &self,
        request: tonic::Request<tf::validate_data_resource_config::Request>,
    ) -> Result<tonic::Response<tf::validate_data_resource_config::Response>, tonic::Status> {
        let request = request.get_ref();
        let mut diags = Diagnostics::default();

        if let Some(data_sources) = self.get_data_sources().into_option(&mut diags) {
            if let Some(data_source) = data_sources.get(&request.type_name) {
                if let Some(config) = request.config {
                    _ = data_source.validate(config.into()).into_option(&mut diags);
                }
            } else {
                diags.root_error_short(format!("Could not find data source {}", request.type_name));
            }
        }

        Ok(tonic::Response::new(
            tf::validate_data_resource_config::Response {
                diagnostics: diags.into(),
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
        let request = request.get_ref();
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct Block {}
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct State {
            read: HashMap<Value<String>, Value<Block>>,
        }
        let config = match from_dynamic::<State>(&request.config) {
            Ok(val) => val,
            Err(err) => {
                return Ok(tonic::Response::new(tf::plan_resource_change::Response {
                    planned_state: None,
                    requires_replace: vec![],
                    planned_private: vec![],
                    diagnostics: vec![tf::Diagnostic {
                        severity: tf::diagnostic::Severity::Error.into(),
                        summary: err.to_string().into(),
                        detail: err.to_string().into(),
                        attribute: None,
                    }],
                    legacy_type_system: false,
                }))
            }
        };
        eprintln!("POUET");
        eprintln!("State: {:?}", config);
        Ok(tonic::Response::new(tf::plan_resource_change::Response {
            planned_state: request.proposed_new_state.clone(),
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
