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
type Pair<'a> = pest::iterators::Pair<'a, Rule>;

#[derive(Debug)]
pub enum Lexeme<'a> {
  Whitespace,
  Comment,
  Meta {
    expr_ix: usize,
    meta_ix: usize,
    prefix: &'a str,
  },
  Discard {
    expr_ix: usize,
  },
  Symbol {
    expr_ix: usize,
    namespace: Option<&'a str>,
    name: &'a str,
  },
  Keyword {
    expr_ix: usize,
    alias: bool,
    namespace: Option<&'a str>,
    name: &'a str,
  },
  StringOpen {
    expr_ix: usize,
  },
  StringClose {
    expr_ix: usize,
  },
  Unescaped {
    expr_ix: usize,
    value: &'a str,
  },
  Escaped {
    expr_ix: usize,
    code: u32,
  },
  BogusMap {
    expr_ix: usize,
  },
  Residual(Pair<'a>),
}

type Lexemes<'a> = Vec<Lexeme<'a>>;

pub fn lex(input: &str) -> Result<Lexemes, Error> {
  let mut helper = Helper::default();
  let mut pairs = ClojurePest::parse(Rule::top_level, input)?;
  let Some(top_level_pair) = pairs.next() else {
    panic!("at least one top-level");
  };
  if pairs.next().is_some() {
    panic!("at most one top-level");
  }
  helper.top_level(top_level_pair);
  Ok(helper.into_lexemes())
}

#[derive(Debug, Default)]
struct Helper<'a> {
  expr_count: usize,
  lexemes: Lexemes<'a>,
}

impl<'a> Helper<'a> {
  fn push(&mut self, lexeme: Lexeme<'a>) {
    self.lexemes.push(lexeme)
  }

  fn into_lexemes(mut self) -> Lexemes<'a> {
    self.lexemes.shrink_to_fit();
    self.lexemes
  }

  fn next_expr_ix(&mut self) -> usize {
    self.expr_count += 1;
    self.expr_count
  }

  fn top_level(&mut self, parent: Pair<'a>) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::form => {
          let current = self.next_expr_ix();
          self.form(child, current);
        }
        Rule::EOI => (),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn form(&mut self, parent: Pair<'a>, current: usize) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::preform => self.preforms(child, current),
        Rule::expr => self.expr(child, current),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn preforms(&mut self, parent: Pair<'a>, current: usize) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::discard_expr => self.discard_expr(child),
        Rule::meta_expr => self.meta_expr(child, current),
        Rule::expr => self.expr(child, current),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn discard_expr(&mut self, parent: Pair<'a>) {
    let expr_ix = self.next_expr_ix();
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::discard_prefix => self.push(Lexeme::Discard { expr_ix }),
        Rule::preform => self.preforms(child, expr_ix),
        Rule::expr => self.expr(child, expr_ix),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn meta_expr(&mut self, parent: Pair<'a>, expr_ix: usize) {
    let meta_ix = self.next_expr_ix();
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::meta_prefix => self.push(Lexeme::Meta {
          expr_ix,
          meta_ix,
          prefix: child.as_str(),
        }),
        Rule::form => self.form(child, meta_ix),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn expr(&mut self, parent: Pair<'a>, expr_ix: usize) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::symbol => self.symbol(child, expr_ix),
        Rule::keyword => self.keyword(child, expr_ix),
        Rule::string => self.string(child, expr_ix),
        Rule::bogus_map => self.push(Lexeme::BogusMap { expr_ix }),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn symbol(&mut self, parent: Pair<'a>, expr_ix: usize) {
    let mut namespace = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::namespace => namespace = Some(child.as_str()),
        Rule::qualified_symbol | Rule::unqualified_symbol => {
          self.push(Lexeme::Symbol {
            expr_ix,
            namespace,
            name: child.as_str(),
          })
        }
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn keyword(&mut self, parent: Pair<'a>, expr_ix: usize) {
    let mut namespace = None;
    let mut alias = false;
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::keyword_prefix => alias = child.as_str() == "::",
        Rule::namespace => namespace = Some(child.as_str()),
        Rule::qualified_symbol | Rule::unqualified_symbol => {
          self.push(Lexeme::Keyword {
            expr_ix,
            alias,
            namespace,
            name: child.as_str(),
          })
        }
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn string(&mut self, parent: Pair<'a>, expr_ix: usize) {
    self.push(Lexeme::StringOpen { expr_ix });
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::unescaped => self.push(Lexeme::Unescaped {
          expr_ix,
          value: child.as_str(),
        }),
        Rule::esc_char => {
          let value = &child.as_str()[1..];
          let code = match value {
            "b" => 0x08,
            "t" => 0x09,
            "n" => 0x0A,
            "f" => 0x0C,
            "r" => 0x0D,
            "\"" => 0x22,
            "\\" => 0x5C,
            e => unreachable!("inexhaustive: {}", e),
          };
          self.push(Lexeme::Escaped { expr_ix, code })
        }
        Rule::esc_octet => {
          let value = &child.as_str()[1..];
          let code = u32::from_str_radix(value, 8).unwrap();
          self.push(Lexeme::Escaped { expr_ix, code })
        }
        Rule::esc_code_point => {
          let value = &child.as_str()[2..];
          let code = u32::from_str_radix(value, 16).unwrap();
          self.push(Lexeme::Escaped { expr_ix, code })
        }
        _ => self.push(Lexeme::Residual(child)),
      }
    }
    self.push(Lexeme::StringClose { expr_ix });
  }
}
