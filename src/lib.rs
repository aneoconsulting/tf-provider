pub mod attribute_path;
pub mod data_source;
pub mod diagnostics;
pub mod plugin;
pub mod provider;
pub mod raw;
pub mod resource;
pub mod schema;
pub mod server;
pub mod tf6provider;
pub mod value;

mod utils;

mod tfplugin6 {
    tonic::include_proto!("tfplugin6");
}

pub use attribute_path::AttributePath;
pub use data_source::DataSource;
pub use diagnostics::Diagnostics;
pub use provider::Provider;
pub use resource::Resource;
pub use schema::{
    Attribute, AttributeConstraint, AttributeType, Block, Description, NestedBlock, Schema,
};
pub use server::serve;
pub use value::{
    Value, ValueAny, ValueEmpty, ValueList, ValueMap, ValueNumber, ValueSet, ValueString,
};

#[macro_export]
macro_rules! map {
    {$($key:expr => $value:expr),*} => {
        {
            let mut map = std::collections::HashMap::default();
            $(
                map.insert($key.into(), $value.into());
            )*
            map
        }
    };

    {$($key:expr => $value:expr),+ ,} => { map!{$($key => $value),+} };
}
