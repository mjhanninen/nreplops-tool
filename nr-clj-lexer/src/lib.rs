// lib.rs
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

#![deny(
  future_incompatible,
  missing_debug_implementations,
  nonstandard_style,
  rust_2021_compatibility,
  // unused
)]
#![allow(unused)]

use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

#[allow(missing_debug_implementations)]
#[derive(Parser)]
#[grammar = "clojure.pest"]
pub struct ClojurePest;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Pest error: {0}")]
  Pest(#[from] pest::error::Error<Rule>),
}

type Pairs<'a> = pest::iterators::Pairs<'a, Rule>;

#[derive(Debug)]
pub enum Lexeme<'a> {
  Residual(Pairs<'a>),
}

pub fn lex(input: &str) -> Result<Vec<Lexeme<'_>>, Error> {
  let pairs = ClojurePest::parse(Rule::top_level, input)?;
  Ok(vec![Lexeme::Residual(pairs)])
}
