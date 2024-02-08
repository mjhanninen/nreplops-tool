// lib.rs
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

#![deny(
  future_incompatible,
  missing_debug_implementations,
  nonstandard_style,
  rust_2021_compatibility,
  unused
)]

pub mod cli;
pub mod clojure;
pub mod conn_expr;
pub mod error;
pub mod host_options;
pub mod hosts_files;
pub mod nrepl;
pub mod outputs;
pub mod pprint;
pub mod routes;
pub mod socket;
pub mod sources;
pub mod sources_old;
pub mod version;

mod bencode;
