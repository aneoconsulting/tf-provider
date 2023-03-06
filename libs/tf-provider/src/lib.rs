pub mod attribute_path;
pub mod data_source;
pub mod diagnostics;
pub mod dynamic;
pub mod plugin;
pub mod provider;
pub mod resource;
pub mod result;
pub mod schema;
pub mod tfprovider;
pub mod value;

mod tfplugin6 {
    tonic::include_proto!("tfplugin6");
}
