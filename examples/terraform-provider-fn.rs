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

use anyhow::Result;
use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use tf_provider::schema::{FunctionSchema, Parameter, Type};
use tf_provider::{
    map, serve, Block, Description, Diagnostics, Function, Provider, Schema, ValueEmpty,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Add;

#[derive(Debug, Serialize, Deserialize)]
pub struct AddArgs {
    a: i32,
    b: i32,
}

#[async_trait]
impl Function for Add {
    type Input<'a> = AddArgs;
    type Output<'a> = i32;

    fn schema(&self, _diags: &mut Diagnostics) -> Option<FunctionSchema> {
        Some(FunctionSchema {
            parameters: vec![
                Parameter {
                    name: "a".into(),
                    param_type: Type::Number,
                    description: Description::plain("First operand"),
                    ..Default::default()
                },
                Parameter {
                    name: "b".into(),
                    param_type: Type::Number,
                    description: Description::plain("Second operand"),
                    ..Default::default()
                },
            ],
            return_type: Type::Number,
            summary: "add".into(),
            description: Description::plain("Add two numbers together"),
            ..Default::default()
        })
    }

    async fn call<'a>(
        &self,
        _diags: &mut Diagnostics,
        params: Self::Input<'a>,
    ) -> Option<Self::Output<'a>> {
        Some(params.a + params.b)
    }
}

#[derive(Debug, Default, Clone)]
pub struct FnProvider;

#[async_trait]
impl Provider for FnProvider {
    type Config<'a> = ValueEmpty;
    type MetaState<'a> = ValueEmpty;

    fn schema(&self, _diags: &mut tf_provider::Diagnostics) -> Option<tf_provider::Schema> {
        Some(Schema {
            version: 1,
            block: Block {
                description: Description::plain("null"),
                ..Default::default()
            },
        })
    }

    fn get_functions(
        &self,
        _diags: &mut tf_provider::Diagnostics,
    ) -> Option<std::collections::HashMap<String, Box<dyn tf_provider::function::DynamicFunction>>>
    {
        Some(map! {
            "add" => Add,
        })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    serve("fn", FnProvider).await
}
