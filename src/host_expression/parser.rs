// host_expression/parser.rs
// Copyright 2022 Matti Hänninen
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

#![allow(missing_debug_implementations)]

pub use pest::{iterators::Pair, Parser};

#[derive(pest_derive::Parser)]
#[grammar = "host_expression/grammar.pest"]
pub struct HostExprLanguage;
