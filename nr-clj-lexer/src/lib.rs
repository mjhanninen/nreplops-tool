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
  Numeric {
    expr_ix: usize,
    literal: &'a str,
    class: NumberClass,
    value: NumericValue<'a>,
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
  BogusMap {
    expr_ix: usize,
  },
  Residual(Pair<'a>),
}

#[derive(Clone, Copy, Debug)]
pub enum NumericValue<'a> {
  Int {
    positive: bool,
    radix: u32,
    value: &'a str,
  },
  Float {
    value: &'a str,
  },
  Fraction {
    positive: bool,
    numerator: &'a str,
    denominator: &'a str,
  },
}

/// Number class as recognized by Clojure
#[derive(Clone, Copy, Debug)]
pub enum NumberClass {
  Long,
  Double,
  BigInt,
  BigDecimal,
  Ratio,
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
        Rule::number => self.number(child, expr_ix),
        Rule::string => self.string(child, expr_ix),
        Rule::symbol => self.symbol(child, expr_ix),
        Rule::keyword => self.keyword(child, expr_ix),
        Rule::bogus_map => self.push(Lexeme::BogusMap { expr_ix }),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn number(&mut self, parent: Pair<'a>, expr_ix: usize) {
    let mut positive = true;
    let literal = parent.as_str();
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::sign => positive = child.as_str() == "+",
        Rule::unsigned_bigfloat => {
          self.unsigned_floats(child, expr_ix, literal, true)
        }
        Rule::unsigned_float => {
          self.unsigned_floats(child, expr_ix, literal, false)
        }
        Rule::unsigned_ratio => {
          self.unsigned_ratio(child, expr_ix, literal, positive)
        }
        Rule::unsigned_radix_int => {
          self.unsigned_radix_int(child, expr_ix, literal, positive)
        }
        Rule::unsigned_bigint => {
          self.unsigned_ints(child, expr_ix, literal, true, positive)
        }
        Rule::unsigned_int => {
          self.unsigned_ints(child, expr_ix, literal, false, positive)
        }
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn unsigned_floats(
    &mut self,
    parent: Pair<'a>,
    expr_ix: usize,
    literal: &'a str,
    big: bool,
  ) {
    self.push(if big {
      Lexeme::Numeric {
        expr_ix,
        literal,
        class: NumberClass::BigDecimal,
        value: NumericValue::Float {
          value: &literal[..literal.len() - 1],
        },
      }
    } else {
      Lexeme::Numeric {
        expr_ix,
        literal,
        class: NumberClass::Double,
        value: NumericValue::Float { value: literal },
      }
    })
  }

  fn unsigned_ratio(
    &mut self,
    parent: Pair<'a>,
    expr_ix: usize,
    literal: &'a str,
    positive: bool,
  ) {
    let mut numerator = None;
    let mut denominator = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::numerator => numerator = Some(child.as_str()),
        Rule::denominator => denominator = Some(child.as_str()),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
    self.push(Lexeme::Numeric {
      expr_ix,
      literal,
      class: NumberClass::Ratio,
      value: NumericValue::Fraction {
        positive,
        numerator: numerator.unwrap(),
        denominator: denominator.unwrap(),
      },
    })
  }
  fn unsigned_radix_int(
    &mut self,
    parent: Pair<'a>,
    expr_ix: usize,
    literal: &'a str,
    positive: bool,
  ) {
    let mut radix = None;
    let mut digits = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::radix => radix = Some(child.as_str()),
        Rule::radix_digits => digits = Some(child.as_str()),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
    self.push(Lexeme::Numeric {
      expr_ix,
      literal,
      class: NumberClass::Long,
      value: NumericValue::Int {
        positive,
        radix: radix.unwrap().parse::<u32>().unwrap(),
        value: digits.unwrap(),
      },
    })
  }

  fn unsigned_ints(
    &mut self,
    parent: Pair<'a>,
    expr_ix: usize,
    literal: &'a str,
    big: bool,
    positive: bool,
  ) {
    let class = if big {
      NumberClass::BigInt
    } else {
      NumberClass::Long
    };
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::oct_digits => self.push(Lexeme::Numeric {
          expr_ix,
          literal,
          class,
          value: NumericValue::Int {
            positive,
            radix: 8,
            value: child.as_str(),
          },
        }),
        Rule::hex_digits => self.push(Lexeme::Numeric {
          expr_ix,
          literal,
          class,
          value: NumericValue::Int {
            positive,
            radix: 16,
            value: child.as_str(),
          },
        }),
        Rule::unsigned_dec => self.push(Lexeme::Numeric {
          expr_ix,
          literal,
          class,
          value: NumericValue::Int {
            positive,
            radix: 10,
            value: child.as_str(),
          },
        }),
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
}
