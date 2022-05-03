// host_options.rs
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

use std::collections::HashMap;

use crate::conn_expr::ConnectionExpr;

// FIXME: This is a bad name. Very easy to confuse with SSH host key.
pub type HostKey = String;

pub type HostOptionsTable = HashMap<HostKey, HostOptions>;

#[derive(Debug)]
pub struct HostOptions {
    pub name: Option<String>,
    pub conn_expr: ConnectionExpr,
    pub ask_confirmation: Option<bool>,
}
