// lex.rs
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

use thiserror::Error;

use super::pest_grammar::*;

use Lexeme as L;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Pest error: {0}")]
  Pest(#[from] pest::error::Error<Rule>),
}

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
  Whitespace {
    source: &'a str,
  },
  Comment {
    source: &'a str,
  },
  Meta {
    form_ix: FormIx,
    data_ix: FormIx,
    source: &'a str,
  },
  Discard {
    form_ix: FormIx,
    source: &'a str,
  },
  Quote {
    form_ix: FormIx,
    source: &'a str,
  },
  VarQuote {
    form_ix: FormIx,
    source: &'a str,
  },
  Synquote {
    form_ix: FormIx,
    source: &'a str,
  },
  Unquote {
    form_ix: FormIx,
    source: &'a str,
  },
  SplicingUnquote {
    form_ix: FormIx,
    source: &'a str,
  },
  Nil {
    form_ix: FormIx,
    source: &'a str,
  },
  Boolean {
    form_ix: FormIx,
    value: bool,
    source: &'a str,
  },
  Numeric {
    form_ix: FormIx,
    class: NumberClass,
    value: NumericValue<'a>,
    source: &'a str,
  },
  Char {
    form_ix: FormIx,
    syntax: CharSyntax,
    value: char,
    source: &'a str,
  },
  String {
    form_ix: FormIx,
    value: Box<[StringFragment<'a>]>,
    source: &'a str,
  },
  Regex {
    form_ix: FormIx,
    source: &'a str,
  },
  Symbol {
    form_ix: FormIx,
    namespace: Option<&'a str>,
    name: &'a str,
    source: &'a str,
  },
  SymbolicValue {
    form_ix: FormIx,
    value: SymbolicValue<'a>,
    source: &'a str,
  },
  Keyword {
    form_ix: FormIx,
    alias: bool,
    namespace: Option<&'a str>,
    name: &'a str,
    source: &'a str,
  },
  StartList {
    form_ix: FormIx,
    source: &'a str,
  },
  EndList {
    form_ix: FormIx,
    source: &'a str,
  },
  StartVector {
    form_ix: FormIx,
    source: &'a str,
  },
  EndVector {
    form_ix: FormIx,
    source: &'a str,
  },
  StartSet {
    form_ix: FormIx,
    source: &'a str,
  },
  EndSet {
    form_ix: FormIx,
    source: &'a str,
  },
  MapQualifier {
    form_ix: FormIx,
    source: &'a str,
  },
  StartMap {
    form_ix: FormIx,
    alias: bool,
    namespace: Option<&'a str>,
    source: &'a str,
  },
  EndMap {
    form_ix: FormIx,
    source: &'a str,
  },
  StartAnonymousFn {
    form_ix: FormIx,
    source: &'a str,
  },
  EndAnonymousFn {
    form_ix: FormIx,
    source: &'a str,
  },
  ReaderConditionalPrefix {
    form_ix: FormIx,
    source: &'a str,
  },
  StartReaderConditional {
    form_ix: FormIx,
    splicing: bool,
    source: &'a str,
  },
  EndReaderConditional {
    form_ix: FormIx,
    source: &'a str,
  },
  TaggedLiteral {
    form_ix: FormIx,
    tag_ix: FormIx,
    arg_ix: FormIx,
    source: &'a str,
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

#[derive(Clone, Copy, Debug)]
pub enum StringFragment<'a> {
  Unescaped { value: &'a str },
  Escaped { code: u32 },
}

type Lexemes<'a> = Vec<Lexeme<'a>>;

#[allow(clippy::result_large_err)]
pub fn lex(input: &str) -> Result<Lexemes, Error> {
  let mut helper = Helper::default();
  let mut pairs = Grammar::parse(Rule::top_level, input)?;
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

  fn whitespace(&mut self, pair: Pair<'a>) {
    self.push(L::Whitespace {
      source: pair.as_str(),
    });
  }

  fn comment(&mut self, pair: Pair<'a>) {
    self.push(L::Comment {
      source: pair.as_str(),
    });
  }

  fn top_level(&mut self, parent: Pair<'a>) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.comment(child),
        Rule::WHITESPACE => self.whitespace(child),
        Rule::form => {
          let current = self.next_form_ix(None);
          self.form(child, current);
        }
        Rule::EOI => (),
        _ => self.push(L::Residual(child)),
      }
    }
  }

  fn form(&mut self, parent: Pair<'a>, current: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.comment(child),
        Rule::WHITESPACE => self.whitespace(child),
        Rule::quote_unquote_form => self.quote_unquote_form(child, current),
        Rule::preform => self.preforms(child, current),
        Rule::form => self.form(child, current),
        Rule::expr => self.expr(child, current),
        _ => self.push(L::Residual(child)),
      }
    }
  }

  fn quote_unquote_form(&mut self, parent: Pair<'a>, parent_ix: FormIx) {
    let child_ix = self.next_form_ix(Some(parent_ix));
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.comment(child),
        Rule::WHITESPACE => self.whitespace(child),
        Rule::quote_unquote_prefix => self.push(match child.as_str() {
          "'" => L::Quote {
            form_ix: parent_ix,
            source: child.as_str(),
          },
          "#'" => L::VarQuote {
            form_ix: parent_ix,
            source: child.as_str(),
          },
          "`" => L::Synquote {
            form_ix: parent_ix,
            source: child.as_str(),
          },
          "~@" => L::SplicingUnquote {
            form_ix: parent_ix,
            source: child.as_str(),
          },
          "~" => L::Unquote {
            form_ix: parent_ix,
            source: child.as_str(),
          },
          _ => unreachable!("quote-unquote prefix case analysis"),
        }),
        Rule::form => self.form(child, child_ix),
        _ => self.push(L::Residual(child)),
      }
    }
  }

  fn preforms(&mut self, parent: Pair<'a>, current: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.comment(child),
        Rule::WHITESPACE => self.whitespace(child),
        Rule::discarded_form => self.discarded_form(child),
        Rule::meta_form => self.meta_form(child, current),
        _ => self.push(L::Residual(child)),
      }
    }
  }

  fn discarded_form(&mut self, parent: Pair<'a>) {
    let form_ix = self.next_form_ix(None);
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.comment(child),
        Rule::WHITESPACE => self.whitespace(child),
        Rule::discard_prefix => self.push(L::Discard {
          form_ix,
          source: child.as_str(),
        }),
        Rule::preform => self.preforms(child, form_ix),
        Rule::form => self.form(child, form_ix),
        _ => self.push(L::Residual(child)),
      }
    }
  }

  fn meta_form(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    let data_ix = self.next_form_ix(None);
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.comment(child),
        Rule::WHITESPACE => self.whitespace(child),
        Rule::meta_prefix => self.push(L::Meta {
          form_ix,
          data_ix,
          source: child.as_str(),
        }),
        Rule::form => self.form(child, data_ix),
        _ => self.push(L::Residual(child)),
      }
    }
  }

  fn expr(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.comment(child),
        Rule::WHITESPACE => self.whitespace(child),
        Rule::nil => self.push(L::Nil {
          form_ix,
          source: child.as_str(),
        }),
        Rule::boolean => self.push(L::Boolean {
          form_ix,
          value: child.as_str() == "true",
          source: child.as_str(),
        }),
        Rule::number => self.number(child, form_ix),
        Rule::char => self.char(child, form_ix),
        Rule::string => self.string(child, form_ix),
        Rule::regex => self.regex(child, form_ix),
        // XXX(soija) FIXME: Capture the `##` prefix for the symbolic value.
        Rule::symbolic_value => self.symbolic_value(child, form_ix),
        Rule::symbol => self.symbol(child, form_ix),
        Rule::keyword => self.keyword(child, form_ix),
        Rule::list => self.list(child, form_ix),
        Rule::vector => self.vector(child, form_ix),
        Rule::anonymous_fn => self.anonymous_fn(child, form_ix),
        Rule::set => self.set(child, form_ix),
        Rule::map => self.map(child, form_ix),
        Rule::reader_conditional => self.reader_conditional(child, form_ix),
        Rule::tagged_literal => self.tagged_literal(child, form_ix),
        _ => self.push(L::Residual(child)),
      }
    }
  }

  fn char(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    let source = parent.as_str();
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::char_name => self.push(L::Char {
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
          source,
        }),
        Rule::char_octal => self.push(L::Char {
          form_ix,
          syntax: CharSyntax::Octal,
          value: char::from_u32(
            u32::from_str_radix(child.as_str(), 8).unwrap(),
          )
          .unwrap(),
          source,
        }),
        Rule::char_code_point => self.push(L::Char {
          form_ix,
          syntax: CharSyntax::CodePoint,
          value: char::from_u32(
            u32::from_str_radix(child.as_str(), 16).unwrap(),
          )
          .unwrap(),
          source,
        }),
        _ => self.push(L::Residual(child)),
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
        _ => self.push(L::Residual(child)),
      }
    }
  }

  fn unsigned_floats(
    &mut self,
    _parent: Pair<'a>,
    form_ix: FormIx,
    literal: &'a str,
    big: bool,
  ) {
    self.push(if big {
      L::Numeric {
        form_ix,
        class: NumberClass::BigDecimal,
        value: NumericValue::Float {
          value: &literal[..literal.len() - 1],
        },
        source: literal,
      }
    } else {
      L::Numeric {
        form_ix,
        class: NumberClass::Double,
        value: NumericValue::Float { value: literal },
        source: literal,
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
        _ => self.push(L::Residual(child)),
      }
    }
    self.push(L::Numeric {
      form_ix,
      source: literal,
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
        Rule::radix_digits => self.push(L::Numeric {
          form_ix,
          source: literal,
          class: NumberClass::Long,
          value: NumericValue::Int {
            positive,
            radix: radix.unwrap().parse::<u32>().unwrap(),
            value: child.as_str(),
          },
        }),
        _ => self.push(L::Residual(child)),
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
        _ => self.push(L::Residual(child)),
      }
    }
    self.push(L::Numeric {
      form_ix,
      source: literal,
      class,
      value: value.unwrap(),
    })
  }

  fn string(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    let mut fragments = Vec::new();
    let literal = parent.as_str();
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::unescaped => fragments.push(StringFragment::Unescaped {
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
          fragments.push(StringFragment::Escaped { code })
        }
        Rule::esc_octet => {
          let value = &child.as_str()[1..];
          let code = u32::from_str_radix(value, 8).unwrap();
          fragments.push(StringFragment::Escaped { code })
        }
        Rule::esc_code_point => {
          let value = &child.as_str()[2..];
          let code = u32::from_str_radix(value, 16).unwrap();
          fragments.push(StringFragment::Escaped { code })
        }
        _ => self.push(L::Residual(child)),
      }
    }
    fragments.shrink_to_fit();
    self.push(L::String {
      form_ix,
      source: literal,
      value: fragments.into_boxed_slice(),
    });
  }

  fn regex(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::regex_content => self.push(L::Regex {
          form_ix,
          source: child.as_str(),
        }),
        _ => self.push(L::Residual(child)),
      }
    }
  }

  fn symbolic_value(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.comment(child),
        Rule::WHITESPACE => self.whitespace(child),
        Rule::unqualified_symbol => self.push(L::SymbolicValue {
          form_ix,
          value: match child.as_str() {
            "Inf" => SymbolicValue::PosInf,
            "-Inf" => SymbolicValue::NegInf,
            "NaN" => SymbolicValue::NaN,
            _ => SymbolicValue::Other(child.as_str()),
          },
          source: child.as_str(),
        }),
        _ => self.push(L::Residual(child)),
      }
    }
  }

  fn symbol(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    let source = parent.as_str();
    let mut namespace = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::namespace => namespace = Some(child.as_str()),
        Rule::qualified_symbol | Rule::unqualified_symbol => {
          self.push(L::Symbol {
            form_ix,
            namespace,
            name: child.as_str(),
            source,
          })
        }
        _ => self.push(L::Residual(child)),
      }
    }
  }

  fn keyword(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    let source = parent.as_str();
    let mut namespace = None;
    let mut alias = false;
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::keyword_prefix => alias = child.as_str() == "::",
        Rule::namespace => namespace = Some(child.as_str()),
        Rule::qualified_symbol | Rule::unqualified_symbol => {
          self.push(L::Keyword {
            form_ix,
            alias,
            namespace,
            name: child.as_str(),
            source,
          })
        }
        _ => self.push(L::Residual(child)),
      }
    }
  }

  fn list(&mut self, parent: Pair<'a>, parent_ix: FormIx) {
    let source = parent.as_str();
    self.body(
      parent,
      parent_ix,
      L::StartList {
        form_ix: parent_ix,
        source: &source[..1],
      },
      L::EndList {
        form_ix: parent_ix,
        source: &source[source.len() - 1..],
      },
    );
  }

  fn vector(&mut self, parent: Pair<'a>, parent_ix: FormIx) {
    let source = parent.as_str();
    self.body(
      parent,
      parent_ix,
      L::StartVector {
        form_ix: parent_ix,
        source: &source[..1],
      },
      L::EndVector {
        form_ix: parent_ix,
        source: &source[source.len() - 1..],
      },
    );
  }

  fn anonymous_fn(&mut self, parent: Pair<'a>, parent_ix: FormIx) {
    let source = parent.as_str();
    self.body(
      parent,
      parent_ix,
      L::StartAnonymousFn {
        form_ix: parent_ix,
        source: &source[..2],
      },
      L::EndAnonymousFn {
        form_ix: parent_ix,
        source: &source[source.len() - 1..],
      },
    );
  }

  fn set(&mut self, parent: Pair<'a>, parent_ix: FormIx) {
    let source = parent.as_str();
    self.body(
      parent,
      parent_ix,
      L::StartSet {
        form_ix: parent_ix,
        source: &source[..2],
      },
      L::EndSet {
        form_ix: parent_ix,
        source: &source[source.len() - 1..],
      },
    );
  }

  fn map(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    let mut alias = false;
    let mut namespace = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.comment(child),
        Rule::WHITESPACE => self.whitespace(child),
        Rule::map_qualifier => {
          self.push(L::MapQualifier {
            form_ix,
            source: child.as_str(),
          });
          for child2 in child.into_inner() {
            match child2.as_rule() {
              Rule::map_qualifier_prefix => alias = child2.as_str() == "#::",
              Rule::namespace => namespace = Some(child2.as_str()),
              _ => self.push(L::Residual(child2)),
            }
          }
        }
        Rule::unqualified_map => {
          let source = child.as_str();
          self.body(
            child,
            form_ix,
            L::StartMap {
              form_ix,
              alias,
              namespace,
              source: &source[..1],
            },
            L::EndMap {
              form_ix,
              source: &source[source.len() - 1..],
            },
          )
        }
        Rule::discarded_form => self.discarded_form(child),
        _ => self.push(L::Residual(child)),
      }
    }
  }

  fn reader_conditional(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    let mut splicing = false;
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.comment(child),
        Rule::WHITESPACE => self.whitespace(child),
        Rule::reader_conditional_prefix => splicing = child.as_str() == "#?@",
        Rule::reader_conditional_body => {
          let source = child.as_str();
          self.body(
            child,
            form_ix,
            L::StartReaderConditional {
              form_ix,
              splicing,
              source: &source[1..],
            },
            L::EndReaderConditional {
              form_ix,
              source: &source[source.len() - 1..],
            },
          )
        }
        Rule::discarded_form => self.discarded_form(child),
        _ => self.push(L::Residual(child)),
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
        Rule::COMMENT => self.comment(child),
        Rule::WHITESPACE => self.whitespace(child),
        Rule::form => {
          let child_ix = self.next_form_ix(Some(parent_ix));
          self.form(child, child_ix)
        }
        Rule::discarded_form => self.discarded_form(child),
        _ => self.push(L::Residual(child)),
      }
    }
    self.push(end_lexeme);
  }

  fn tagged_literal(&mut self, parent: Pair<'a>, form_ix: FormIx) {
    let tag_ix = self.next_form_ix(Some(form_ix));
    let arg_ix = self.next_form_ix(Some(form_ix));
    self.push(L::TaggedLiteral {
      form_ix,
      tag_ix,
      arg_ix,
      source: parent.as_str(),
    });
    for child in parent.into_inner() {
      match child.as_rule() {
        Rule::COMMENT => self.comment(child),
        Rule::WHITESPACE => self.whitespace(child),
        Rule::tagged_literal_tag => {
          for child2 in child.into_inner() {
            match child2.as_rule() {
              Rule::COMMENT => self.comment(child2),
              Rule::WHITESPACE => self.whitespace(child2),
              Rule::preform => self.preforms(child2, tag_ix),
              Rule::symbol => self.symbol(child2, tag_ix),
              _ => self.push(L::Residual(child2)),
            }
          }
        }
        Rule::form => self.form(child, arg_ix),
        _ => self.push(L::Residual(child)),
      }
    }
  }
}
