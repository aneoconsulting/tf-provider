pub mod attribute_path;
pub mod data_source;
pub mod diagnostics;
pub mod dynamic;
pub mod plugin;
pub mod provider;
pub mod resource;
pub mod schema;
pub mod tf6provider;
pub mod value;

mod utils;

mod tfplugin6 {
    tonic::include_proto!("tfplugin6");
}

pub use data_source::DataSource;
pub use diagnostics::Diagnostics;
pub use provider::Provider;
pub use resource::Resource;
pub use schema::Schema;
