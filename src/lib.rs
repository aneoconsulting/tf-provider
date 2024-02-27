// This file is part of the tf-provider project
//
// Copyright (C) ANEO, 2024-2024. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License")
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
