// clojure/pest_grammar.rs
// Copyright 2024 Matti Hänninen
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

pub use pest::Parser;
use pest_derive::Parser;

#[allow(missing_debug_implementations)]
#[derive(Parser)]
#[grammar = "clojure/grammar.pest"]
pub struct Grammar;

pub type Pair<'a> = pest::iterators::Pair<'a, Rule>;
