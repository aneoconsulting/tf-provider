use std::sync::Arc;

use crate::diagnostics::Diagnostics;
use crate::dynamic::DynamicValue;
use crate::server::Server;
use crate::tfplugin6 as tf;
use crate::tfplugin6::get_provider_schema::ServerCapabilities;
use crate::utils::{CollectDiagnostics, MapInto, OptionExpand};

#[tonic::async_trait]
impl tf::provider_server::Provider for Arc<Server> {
    async fn get_provider_schema(
        &self,
        _request: tonic::Request<tf::get_provider_schema::Request>,
    ) -> Result<tonic::Response<tf::get_provider_schema::Response>, tonic::Status> {
        let schema = self.schema.clone().map_into();
        let meta_schema = self.meta_schema.clone().map_into();
        let resources = self
            .resources
            .iter()
            .map(|(name, (_, schema))| (name.clone(), schema.clone().into()))
            .collect();
        let data_sources = self
            .data_sources
            .iter()
            .map(|(name, (_, schema))| (name.clone(), schema.clone().into()))
            .collect();

        Ok(tonic::Response::new(tf::get_provider_schema::Response {
            provider: schema,
            resource_schemas: resources,
            data_source_schemas: data_sources,
            diagnostics: self.init_diags.clone().into(),
            provider_meta: meta_schema,
            server_capabilities: Some(ServerCapabilities { plan_destroy: true }),
        }))
    }
    async fn validate_provider_config(
        &self,
        request: tonic::Request<tf::validate_provider_config::Request>,
    ) -> Result<tonic::Response<tf::validate_provider_config::Response>, tonic::Status> {
        let request = request.into_inner();
        let mut diags = Diagnostics::default();

        _ = if let Some(config) = request.config {
            self.provider.validate(&mut diags, config.into()).await
        } else {
            None
        }
        .ok_or_else(|| diags.internal_error());

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
        let request = request.into_inner();
        let mut diags = Diagnostics::default();

        _ = if let (Some(resource), Some(config)) = (
            self.get_resource(&mut diags, &request.type_name),
            request.config,
        ) {
            resource.validate(&mut diags, config.into()).await
        } else {
            None
        }
        .ok_or_else(|| diags.internal_error());

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
        let request = request.into_inner();
        let mut diags = Diagnostics::default();

        _ = if let (Some(data_source), Some(config)) = (
            self.get_data_source(&mut diags, &request.type_name),
            request.config,
        ) {
            data_source.validate(&mut diags, config.into()).await
        } else {
            None
        }
        .ok_or_else(|| diags.internal_error());

        Ok(tonic::Response::new(
            tf::validate_data_resource_config::Response {
                diagnostics: diags.into(),
            },
        ))
    }
    async fn upgrade_resource_state(
        &self,
        request: tonic::Request<tf::upgrade_resource_state::Request>,
    ) -> Result<tonic::Response<tf::upgrade_resource_state::Response>, tonic::Status> {
        let request = request.into_inner();
        let mut diags = Diagnostics::default();

        let upgraded_state = if let (Some((resource, schema)), Some(raw_state)) =
            (self.resources.get(&request.type_name), request.raw_state)
        {
            if raw_state.json.is_empty() {
                diags.root_error_short("Upgrading from a legacy state is not supported");
                None
            } else {
                let json = DynamicValue::Json(raw_state.json);
                if request.version == schema.version {
                    Some(json)
                } else {
                    resource.upgrade(&mut diags, request.version, json).await
                }
            }
        } else {
            None
        }
        .collect_diagnostics(&mut diags);

        Ok(tonic::Response::new(tf::upgrade_resource_state::Response {
            diagnostics: diags.into(),
            upgraded_state: upgraded_state.map_into(),
        }))
    }
    /// ////// One-time initialization, called before other functions below
    async fn configure_provider(
        &self,
        request: tonic::Request<tf::configure_provider::Request>,
    ) -> Result<tonic::Response<tf::configure_provider::Response>, tonic::Status> {
        let request = request.into_inner();
        let mut diags = Diagnostics::default();

        if let Some(config) = request.config {
            if self
                .provider
                .configure(&mut diags, request.terraform_version, config.into())
                .await
                .is_none()
            {
                diags.internal_error();
            }
        }

        Ok(tonic::Response::new(tf::configure_provider::Response {
            diagnostics: diags.into(),
        }))
    }
    /// ////// Managed Resource Lifecycle
    async fn read_resource(
        &self,
        request: tonic::Request<tf::read_resource::Request>,
    ) -> Result<tonic::Response<tf::read_resource::Response>, tonic::Status> {
        let request = request.into_inner();
        let mut diags = Diagnostics::default();

        let (state, private_state) =
            if let (Some(resource), Some(state), Some(provider_meta_state)) = (
                self.get_resource(&mut diags, &request.type_name),
                request.current_state,
                request.provider_meta,
            ) {
                resource
                    .read(
                        &mut diags,
                        state.into(),
                        request.private,
                        provider_meta_state.into(),
                    )
                    .await
            } else {
                None
            }
            .collect_diagnostics(&mut diags)
            .expand();

        Ok(tonic::Response::new(tf::read_resource::Response {
            new_state: state.map_into(),
            diagnostics: diags.into(),
            private: private_state.unwrap_or_default(),
        }))
    }
    async fn plan_resource_change(
        &self,
        request: tonic::Request<tf::plan_resource_change::Request>,
    ) -> Result<tonic::Response<tf::plan_resource_change::Response>, tonic::Status> {
        let request = request.into_inner();
        let mut diags = Diagnostics::default();

        let (state, private_state, triggers) = if let (
            Some(resource),
            Some(prior_state),
            Some(proposed_state),
            Some(config_state),
            Some(provider_meta_state),
        ) = (
            self.get_resource(&mut diags, &request.type_name),
            request.prior_state,
            request.proposed_new_state,
            request.config,
            request.provider_meta,
        ) {
            resource
                .plan(
                    &mut diags,
                    prior_state.into(),
                    proposed_state.into(),
                    config_state.into(),
                    request.prior_private,
                    provider_meta_state.into(),
                )
                .await
        } else {
            None
        }
        .map(|(state, private_state, triggers)| {
            (
                state.into(),
                private_state,
                triggers.into_iter().map(|attr| attr.into()).collect(),
            )
        })
        .collect_diagnostics(&mut diags)
        .expand();

        Ok(tonic::Response::new(tf::plan_resource_change::Response {
            planned_state: state,
            requires_replace: triggers.unwrap_or_default(),
            planned_private: private_state.unwrap_or_default(),
            diagnostics: diags.into(),
            legacy_type_system: false,
        }))
    }
    async fn apply_resource_change(
        &self,
        request: tonic::Request<tf::apply_resource_change::Request>,
    ) -> Result<tonic::Response<tf::apply_resource_change::Response>, tonic::Status> {
        let request = request.into_inner();
        let mut diags = Diagnostics::default();

        let (state, private_state) = if let (
            Some(resource),
            Some(prior_state),
            Some(planned_state),
            Some(config_state),
            Some(provider_meta_state),
        ) = (
            self.get_resource(&mut diags, &request.type_name),
            request.prior_state,
            request.planned_state,
            request.config,
            request.provider_meta,
        ) {
            resource
                .apply(
                    &mut diags,
                    prior_state.into(),
                    planned_state.into(),
                    config_state.into(),
                    request.planned_private,
                    provider_meta_state.into(),
                )
                .await
        } else {
            None
        }
        .collect_diagnostics(&mut diags)
        .expand();

        Ok(tonic::Response::new(tf::apply_resource_change::Response {
            new_state: state.map_into(),
            private: private_state.unwrap_or_default(),
            diagnostics: diags.into(),
            legacy_type_system: false,
        }))
    }
    async fn import_resource_state(
        &self,
        request: tonic::Request<tf::import_resource_state::Request>,
    ) -> Result<tonic::Response<tf::import_resource_state::Response>, tonic::Status> {
        let request = request.into_inner();
        let mut diags = Diagnostics::default();

        let imported = if let Some(resource) = self.get_resource(&mut diags, &request.type_name) {
            resource.import(&mut diags, request.id).await
        } else {
            None
        }
        .collect_diagnostics(&mut diags)
        .map(
            |(state, private_state)| tf::import_resource_state::ImportedResource {
                type_name: request.type_name,
                state: Some(state.into()),
                private: private_state,
            },
        );

        Ok(tonic::Response::new(tf::import_resource_state::Response {
            imported_resources: imported.into_iter().collect(),
            diagnostics: diags.into(),
        }))
    }
    async fn read_data_source(
        &self,
        request: tonic::Request<tf::read_data_source::Request>,
    ) -> Result<tonic::Response<tf::read_data_source::Response>, tonic::Status> {
        let request = request.into_inner();
        let mut diags = Diagnostics::default();

        let state = if let (Some(data_source), Some(config), Some(provider_meta_state)) = (
            self.get_data_source(&mut diags, &request.type_name),
            request.config,
            request.provider_meta,
        ) {
            data_source
                .read(&mut diags, config.into(), provider_meta_state.into())
                .await
        } else {
            None
        }
        .collect_diagnostics(&mut diags);

        Ok(tonic::Response::new(tf::read_data_source::Response {
            state: state.map_into(),
            diagnostics: diags.into(),
        }))
    }
    /// ////// Graceful Shutdown
    async fn stop_provider(
        &self,
        _request: tonic::Request<tf::stop_provider::Request>,
    ) -> Result<tonic::Response<tf::stop_provider::Response>, tonic::Status> {
        self.cancellation_token.cancel();

        Ok(tonic::Response::new(tf::stop_provider::Response {
            error: "".to_string(),
        }))
    }
}
