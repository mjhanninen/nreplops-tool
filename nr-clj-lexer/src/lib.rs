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

#[derive(Clone, Copy, Debug)]
pub struct FormIx {
  /// The index of the parent form or zero, if none.
  pub parent: u32,
  /// The index of this form.
  pub ix: u32,
}

impl FormIx {
  fn root(ix: u32) -> Self {
    Self { parent: 0, ix }
  }
  fn child(&self, ix: u32) -> Self {
    Self {
      parent: self.ix,
      ix,
    }
  }
}

#[derive(Debug)]
pub enum Lexeme<'a> {
  Whitespace,
  Comment,
  Meta {
    form_ix: FormIx,
    data_ix: FormIx,
    prefix: &'a str,
  },
  Discard {
    form_ix: FormIx,
  },
  Quote {
    form_ix: FormIx,
  },
  VarQuote {
    form_ix: FormIx,
  },
  Synquote {
    form_ix: FormIx,
  },
  Unquote {
    form_ix: FormIx,
  },
  SplicingUnquote {
    form_ix: FormIx,
  },
  Nil {
    form_ix: FormIx,
  },
  Boolean {
    form_ix: FormIx,
    value: bool,
  },
  Numeric {
    form_ix: FormIx,
    literal: &'a str,
    class: NumberClass,
    value: NumericValue<'a>,
  },
  Char {
    form_ix: FormIx,
    syntax: CharSyntax,
    value: char,
  },
  StringOpen {
    form_ix: FormIx,
  },
  StringClose {
    form_ix: FormIx,
  },
  Unescaped {
    form_ix: FormIx,
    value: &'a str,
  },
  Escaped {
    form_ix: FormIx,
    code: u32,
  },
  Regex {
    form_ix: FormIx,
    value: &'a str,
  },
  Symbol {
    form_ix: FormIx,
    namespace: Option<&'a str>,
    name: &'a str,
  },
  SymbolicValue {
    form_ix: FormIx,
    value: SymbolicValue<'a>,
  },
  Keyword {
    form_ix: FormIx,
    alias: bool,
    namespace: Option<&'a str>,
    name: &'a str,
  },
  StartList {
    form_ix: FormIx,
  },
  EndList {
    form_ix: FormIx,
  },
  StartVector {
    form_ix: FormIx,
  },
  EndVector {
    form_ix: FormIx,
  },
  StartSet {
    form_ix: FormIx,
  },
  EndSet {
    form_ix: FormIx,
  },
  StartMap {
    form_ix: FormIx,
    alias: bool,
    namespace: Option<&'a str>,
  },
  EndMap {
    form_ix: FormIx,
  },
  StartAnonymousFn {
    form_ix: FormIx,
  },
  EndAnonymousFn {
    form_ix: FormIx,
  },
  StartReaderConditional {
    form_ix: FormIx,
    splicing: bool,
  },
  EndReaderConditional {
    form_ix: FormIx,
  },
  Residual(Pair<'a>),
}

#[derive(Clone, Copy, Debug)]
pub enum CharSyntax {
  Name,
  Octal,
  CodePoint,
  Simple,
}

#[derive(Clone, Copy, Debug)]
pub enum SymbolicValue<'a> {
  PosInf,
  NegInf,
  NaN,
  Other(&'a str),
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
  form_count: u32,
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

  fn next_form_ix(&mut self, parent: Option<FormIx>) -> FormIx {
    self.form_count += 1;
    parent
      .map(|p| p.child(self.form_count))
      .unwrap_or_else(|| FormIx::root(self.form_count))
  }

  fn top_level(&mut self, parent: Pair<'a>) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::form => {
          let current = self.next_form_ix(None);
          self.form(child, current);
        }
        Rule::EOI => (),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn form(&mut self, parent: Pair<'a>, current: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::quote_unquote_form => self.quote_unquote_form(child, current),
        Rule::preform => self.preforms(child, current),
        Rule::form => self.form(child, current),
        Rule::expr => self.expr(child, current),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn quote_unquote_form(&mut self, parent: Pair<'a>, parent_ix: FormIx) {
    let child_ix = self.next_form_ix(Some(parent_ix));
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::quote_unquote_prefix => self.push(match child.as_str() {
          "'" => Lexeme::Quote { form_ix: parent_ix },
          "#'" => Lexeme::VarQuote { form_ix: parent_ix },
          "`" => Lexeme::Synquote { form_ix: parent_ix },
          "~@" => Lexeme::SplicingUnquote { form_ix: parent_ix },
          "~" => Lexeme::Unquote { form_ix: parent_ix },
          _ => unreachable!("quote-unquote prefix case analysis"),
        }),
        Rule::form => self.form(child, child_ix),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn preforms(&mut self, parent: Pair<'a>, current: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::discarded_form => self.discarded_form(child),
        Rule::meta_form => self.meta_form(child, current),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn discarded_form(&mut self, parent: Pair<'a>) {
    let form_ix = self.next_form_ix(None);
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::discard_prefix => self.push(Lexeme::Discard { form_ix }),
        Rule::preform => self.preforms(child, form_ix),
        Rule::form => self.form(child, form_ix),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn meta_form(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    let data_ix = self.next_form_ix(None);
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::meta_prefix => self.push(Lexeme::Meta {
          form_ix,
          data_ix,
          prefix: child.as_str(),
        }),
        Rule::form => self.form(child, data_ix),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn expr(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::nil => self.push(Lexeme::Nil { form_ix }),
        Rule::boolean => self.push(Lexeme::Boolean {
          form_ix,
          value: child.as_str() == "true",
        }),
        Rule::number => self.number(child, form_ix),
        Rule::char => self.char(child, form_ix),
        Rule::string => self.string(child, form_ix),
        Rule::regex => self.regex(child, form_ix),
        Rule::symbolic_value => self.symbolic_value(child, form_ix),
        Rule::symbol => self.symbol(child, form_ix),
        Rule::keyword => self.keyword(child, form_ix),
        Rule::list => self.list(child, form_ix),
        Rule::vector => self.vector(child, form_ix),
        Rule::anonymous_fn => self.anonymous_fn(child, form_ix),
        Rule::set => self.set(child, form_ix),
        Rule::map => self.map(child, form_ix),
        Rule::reader_conditional => self.reader_conditional(child, form_ix),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn char(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::char_name => self.push(Lexeme::Char {
          form_ix,
          syntax: CharSyntax::Name,
          value: match child.as_str() {
            "newline" => '\n',
            "space" => ' ',
            "tab" => '\t',
            "formfeed" => '\u{0C}',
            "backspace" => '\u{08}',
            _ => unreachable!("char name case analysis"),
          },
        }),
        Rule::char_octal => self.push(Lexeme::Char {
          form_ix,
          syntax: CharSyntax::Octal,
          value: char::from_u32(
            u32::from_str_radix(child.as_str(), 8).unwrap(),
          )
          .unwrap(),
        }),
        Rule::char_code_point => self.push(Lexeme::Char {
          form_ix,
          syntax: CharSyntax::CodePoint,
          value: char::from_u32(
            u32::from_str_radix(child.as_str(), 16).unwrap(),
          )
          .unwrap(),
        }),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn number(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    let mut positive = true;
    let literal = parent.as_str();
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::sign => positive = child.as_str() == "+",
        Rule::unsigned_bigfloat => {
          self.unsigned_floats(child, form_ix, literal, true)
        }
        Rule::unsigned_float => {
          self.unsigned_floats(child, form_ix, literal, false)
        }
        Rule::unsigned_ratio => {
          self.unsigned_ratio(child, form_ix, literal, positive)
        }
        Rule::unsigned_radix_int => {
          self.unsigned_radix_int(child, form_ix, literal, positive)
        }
        Rule::unsigned_int => {
          self.unsigned_int(child, form_ix, literal, positive)
        }
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn unsigned_floats(
    &mut self,
    parent: Pair<'a>,
    form_ix: FormIx,
    literal: &'a str,
    big: bool,
  ) {
    self.push(if big {
      Lexeme::Numeric {
        form_ix,
        literal,
        class: NumberClass::BigDecimal,
        value: NumericValue::Float {
          value: &literal[..literal.len() - 1],
        },
      }
    } else {
      Lexeme::Numeric {
        form_ix,
        literal,
        class: NumberClass::Double,
        value: NumericValue::Float { value: literal },
      }
    })
  }

  fn unsigned_ratio(
    &mut self,
    parent: Pair<'a>,

    form_ix: FormIx,
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
      form_ix,
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

    form_ix: FormIx,
    literal: &'a str,
    positive: bool,
  ) {
    let mut radix = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::radix => radix = Some(child.as_str()),
        Rule::radix_digits => self.push(Lexeme::Numeric {
          form_ix,
          literal,
          class: NumberClass::Long,
          value: NumericValue::Int {
            positive,
            radix: radix.unwrap().parse::<u32>().unwrap(),
            value: child.as_str(),
          },
        }),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn unsigned_int(
    &mut self,
    parent: Pair<'a>,

    form_ix: FormIx,
    literal: &'a str,
    positive: bool,
  ) {
    let mut class = NumberClass::Long;
    let mut value = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::oct_digits => {
          value = Some(NumericValue::Int {
            positive,
            radix: 8,
            value: child.as_str(),
          })
        }
        Rule::hex_digits => {
          value = Some(NumericValue::Int {
            positive,
            radix: 16,
            value: child.as_str(),
          })
        }
        Rule::unsigned_dec => {
          value = Some(NumericValue::Int {
            positive,
            radix: 10,
            value: child.as_str(),
          })
        }
        Rule::bigint_suffix => class = NumberClass::BigInt,
        _ => self.push(Lexeme::Residual(child)),
      }
    }
    self.push(Lexeme::Numeric {
      form_ix,
      literal,
      class,
      value: value.unwrap(),
    })
  }

  fn string(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    self.push(Lexeme::StringOpen { form_ix });
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::unescaped => self.push(Lexeme::Unescaped {
          form_ix,
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
          self.push(Lexeme::Escaped { form_ix, code })
        }
        Rule::esc_octet => {
          let value = &child.as_str()[1..];
          let code = u32::from_str_radix(value, 8).unwrap();
          self.push(Lexeme::Escaped { form_ix, code })
        }
        Rule::esc_code_point => {
          let value = &child.as_str()[2..];
          let code = u32::from_str_radix(value, 16).unwrap();
          self.push(Lexeme::Escaped { form_ix, code })
        }
        _ => self.push(Lexeme::Residual(child)),
      }
    }
    self.push(Lexeme::StringClose { form_ix });
  }

  fn regex(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::regex_content => self.push(Lexeme::Regex {
          form_ix,
          value: child.as_str(),
        }),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn symbolic_value(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => (),
        Rule::WHITESPACE => (),
        Rule::unqualified_symbol => self.push(Lexeme::SymbolicValue {
          form_ix,
          value: match child.as_str() {
            "Inf" => SymbolicValue::PosInf,
            "-Inf" => SymbolicValue::NegInf,
            "NaN" => SymbolicValue::NaN,
            _ => SymbolicValue::Other(child.as_str()),
          },
        }),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn symbol(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    let mut namespace = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::namespace => namespace = Some(child.as_str()),
        Rule::qualified_symbol | Rule::unqualified_symbol => {
          self.push(Lexeme::Symbol {
            form_ix,
            namespace,
            name: child.as_str(),
          })
        }
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn keyword(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    let mut namespace = None;
    let mut alias = false;
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::keyword_prefix => alias = child.as_str() == "::",
        Rule::namespace => namespace = Some(child.as_str()),
        Rule::qualified_symbol | Rule::unqualified_symbol => {
          self.push(Lexeme::Keyword {
            form_ix,
            alias,
            namespace,
            name: child.as_str(),
          })
        }
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn list(&mut self, parent: Pair<'a>, parent_ix: FormIx) {
    self.body(
      parent,
      parent_ix,
      Lexeme::StartList { form_ix: parent_ix },
      Lexeme::EndList { form_ix: parent_ix },
    );
  }

  fn vector(&mut self, parent: Pair<'a>, parent_ix: FormIx) {
    self.body(
      parent,
      parent_ix,
      Lexeme::StartVector { form_ix: parent_ix },
      Lexeme::EndVector { form_ix: parent_ix },
    );
  }

  fn anonymous_fn(&mut self, parent: Pair<'a>, parent_ix: FormIx) {
    self.body(
      parent,
      parent_ix,
      Lexeme::StartAnonymousFn { form_ix: parent_ix },
      Lexeme::EndAnonymousFn { form_ix: parent_ix },
    );
  }

  fn set(&mut self, parent: Pair<'a>, parent_ix: FormIx) {
    self.body(
      parent,
      parent_ix,
      Lexeme::StartSet { form_ix: parent_ix },
      Lexeme::EndSet { form_ix: parent_ix },
    );
  }

  fn map(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    let mut alias = false;
    let mut namespace = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::map_qualifier => {
          for child2 in child.into_inner() {
            match child2.as_rule() {
              Rule::map_qualifier_prefix => alias = child2.as_str() == "#::",
              Rule::namespace => namespace = Some(child2.as_str()),
              _ => self.push(Lexeme::Residual(child2)),
            }
          }
        }
        Rule::unqualified_map => self.body(
          child,
          form_ix,
          Lexeme::StartMap {
            form_ix,
            alias,
            namespace,
          },
          Lexeme::EndMap { form_ix },
        ),
        Rule::discarded_form => self.discarded_form(child),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn reader_conditional(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    let mut splicing = false;
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::reader_conditional_prefix => splicing = child.as_str() == "#?@",
        Rule::reader_conditional_body => self.body(
          child,
          form_ix,
          Lexeme::StartReaderConditional { form_ix, splicing },
          Lexeme::EndMap { form_ix },
        ),
        Rule::discarded_form => self.discarded_form(child),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
  }

  fn body(
    &mut self,
    parent: Pair<'a>,
    parent_ix: FormIx,
    start_lexeme: Lexeme<'a>,
    end_lexeme: Lexeme<'a>,
  ) {
    self.push(start_lexeme);
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.push(Lexeme::Comment),
        Rule::WHITESPACE => self.push(Lexeme::Whitespace),
        Rule::form => {
          let child_ix = self.next_form_ix(Some(parent_ix));
          self.form(child, child_ix)
        }
        Rule::discarded_form => self.discarded_form(child),
        _ => self.push(Lexeme::Residual(child)),
      }
    }
    self.push(end_lexeme);
  }
}
