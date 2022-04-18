// conn_expr/mod.rs
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

pub mod parser;

mod addr;
mod port_set;
mod resolution;

// XXX(soija) Okay, "deinception" this
#[allow(clippy::module_inception)]
mod conn_expr;

pub use self::{
    addr::{
        Addr, ConversionError as AddrConversionError,
        ParseError as AddrParseError,
    },
    conn_expr::{ConnectionExpr, Route, Routes},
    port_set::{CannotConvertToPortSetError, Port, PortSet, PortSetParseError},
    resolution::ConnectionExprSource,
};
