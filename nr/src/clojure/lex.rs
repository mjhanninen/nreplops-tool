// clojure/lex.rs
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

use std::rc::Rc;

use thiserror::Error;

use super::pest_grammar::*;

use Lexeme as L;
use Rule as R;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Pest error: {0}")]
  Pest(#[from] pest::error::Error<Rule>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
pub enum Lexeme {
  /// Whitespace
  Whitespace {
    source: Rc<str>,
  },
  /// Comment
  Comment {
    /// The original source for the comment line.
    ///
    /// Effectively the end of the line starting from the comment marker up to
    /// but excluding the line break.  Note that in addition to `;` the comment
    /// marker can be `#!`.
    source: Rc<str>,
  },
  /// Meta data prefix (`^` or `#^`)
  Meta {
    /// The form to which the meta data is attached.  In essence, the form that
    /// owns the meta data.
    form_ix: FormIx,
    /// The form which contains the meta data itself.
    data_ix: FormIx,
    /// The original source for the meta data prefix.  Effectively either `"^"`
    /// or `"#^"`.
    source: Rc<str>,
  },
  Discard {
    form_ix: FormIx,
    source: Rc<str>,
  },
  Quote {
    form_ix: FormIx,
    source: Rc<str>,
  },
  VarQuote {
    form_ix: FormIx,
    source: Rc<str>,
  },
  Synquote {
    form_ix: FormIx,
    source: Rc<str>,
  },
  Unquote {
    form_ix: FormIx,
    source: Rc<str>,
  },
  SplicingUnquote {
    form_ix: FormIx,
    source: Rc<str>,
  },
  Nil {
    form_ix: FormIx,
    source: Rc<str>,
  },
  Boolean {
    form_ix: FormIx,
    value: bool,
    source: Rc<str>,
  },
  Numeric {
    form_ix: FormIx,
    class: NumberClass,
    value: NumericValue,
    source: Rc<str>,
  },
  Char {
    form_ix: FormIx,
    syntax: CharSyntax,
    value: char,
    source: Rc<str>,
  },
  String {
    form_ix: FormIx,
    value: Box<[StringFragment]>,
    source: Rc<str>,
  },
  Regex {
    form_ix: FormIx,
    source: Rc<str>,
  },
  SymbolicValuePrefix {
    form_ix: FormIx,
    source: Rc<str>,
  },
  SymbolicValue {
    form_ix: FormIx,
    value: SymbolicValue,
    source: Rc<str>,
  },
  Symbol {
    form_ix: FormIx,
    namespace: Option<Rc<str>>,
    name: Rc<str>,
    source: Rc<str>,
  },
  Tag {
    form_ix: FormIx,
    namespace: Option<Rc<str>>,
    name: Rc<str>,
    source: Rc<str>,
  },
  Keyword {
    form_ix: FormIx,
    alias: bool,
    namespace: Option<Rc<str>>,
    name: Rc<str>,
    source: Rc<str>,
  },
  StartList {
    form_ix: FormIx,
    source: Rc<str>,
  },
  EndList {
    form_ix: FormIx,
    source: Rc<str>,
  },
  StartVector {
    form_ix: FormIx,
    source: Rc<str>,
  },
  EndVector {
    form_ix: FormIx,
    source: Rc<str>,
  },
  StartSet {
    form_ix: FormIx,
    source: Rc<str>,
  },
  EndSet {
    form_ix: FormIx,
    source: Rc<str>,
  },
  MapQualifier {
    form_ix: FormIx,
    source: Rc<str>,
  },
  StartMap {
    form_ix: FormIx,
    alias: bool,
    namespace: Option<Rc<str>>,
    source: Rc<str>,
  },
  EndMap {
    form_ix: FormIx,
    source: Rc<str>,
  },
  StartAnonymousFn {
    form_ix: FormIx,
    source: Rc<str>,
  },
  EndAnonymousFn {
    form_ix: FormIx,
    source: Rc<str>,
  },
  ReaderConditionalPrefix {
    form_ix: FormIx,
    source: Rc<str>,
  },
  StartReaderConditional {
    form_ix: FormIx,
    splicing: bool,
    source: Rc<str>,
  },
  EndReaderConditional {
    form_ix: FormIx,
    source: Rc<str>,
  },
  TaggedLiteral {
    form_ix: FormIx,
    tag_ix: FormIx,
    arg_ix: FormIx,
    source: Rc<str>,
  },
  Residual {
    pair: Box<str>,
    ploc: ParserLoc,
  },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParserLoc {
  TopLevel,
  Form,
  QuoteUnquoteForm,
  Preforms,
  DiscardedForm,
  MetaForm,
  Expr,
  Char,
  Number,
  UnsignedRatio,
  UnsignedRadixInt,
  UnsignedInt,
  String,
  Regex,
  SymbolicValue,
  Symbol,
  Tag,
  Keyword,
  Map,
  MapQualifier,
  ReaderConditional,
  Body,
  TaggedLiteralTag,
  TaggedLiteralValue,
}

#[derive(Clone, Copy, Debug)]
pub enum CharSyntax {
  Name,
  Octal,
  CodePoint,
  Simple,
}

#[derive(Clone, Debug)]
pub enum SymbolicValue {
  PosInf,
  NegInf,
  NaN,
  Other(Rc<str>),
}

#[derive(Clone, Debug)]
pub enum NumericValue {
  Int {
    positive: bool,
    radix: u32,
    value: Rc<str>,
  },
  Float {
    value: Rc<str>,
  },
  Fraction {
    positive: bool,
    numerator: Rc<str>,
    denominator: Rc<str>,
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

#[derive(Clone, Debug)]
pub enum StringFragment {
  Unescaped { value: Rc<str> },
  Escaped { code: u32 },
}

type Lexemes = Vec<Lexeme>;

#[allow(clippy::result_large_err)]
pub fn lex(input: &str) -> Result<Lexemes, Error> {
  let mut helper = Helper::default();
  let mut pairs = Grammar::parse(R::top_level, input)?;
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
struct Helper {
  form_count: u32,
  lexemes: Lexemes,
}

impl Helper {
  fn push(&mut self, lexeme: Lexeme) {
    self.lexemes.push(lexeme)
  }

  fn into_lexemes(mut self) -> Lexemes {
    self.lexemes.shrink_to_fit();
    self.lexemes
  }

  fn next_form_ix(&mut self, parent: Option<FormIx>) -> FormIx {
    self.form_count += 1;
    parent
      .map(|p| p.child(self.form_count))
      .unwrap_or_else(|| FormIx::root(self.form_count))
  }

  fn whitespace(&mut self, pair: Pair) {
    self.push(L::Whitespace {
      source: pair.as_str().into(),
    });
  }

  fn comment(&mut self, pair: Pair) {
    self.push(L::Comment {
      source: pair.as_str().into(),
    });
  }

  fn top_level(&mut self, parent: Pair) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),
        R::form => {
          let current = self.next_form_ix(None);
          self.form(child, current);
        }
        R::EOI => (),
        _ => self.push(L::Residual {
          pair: format!("{:?}", child).into_boxed_str(),
          ploc: ParserLoc::TopLevel,
        }),
      }
    }
  }

  fn form(&mut self, parent: Pair, current: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),
        R::quote_unquote_form => self.quote_unquote_form(child, current),
        R::preform => self.preforms(child, current),
        R::form => self.form(child, current),
        R::expr => self.expr(child, current),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::Form,
        }),
      }
    }
  }

  fn quote_unquote_form(&mut self, parent: Pair, parent_ix: FormIx) {
    let child_ix = self.next_form_ix(Some(parent_ix));
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),
        R::quote_unquote_prefix => self.push(match child.as_str() {
          "'" => L::Quote {
            form_ix: parent_ix,
            source: child.as_str().into(),
          },
          "#'" => L::VarQuote {
            form_ix: parent_ix,
            source: child.as_str().into(),
          },
          "`" => L::Synquote {
            form_ix: parent_ix,
            source: child.as_str().into(),
          },
          "~@" => L::SplicingUnquote {
            form_ix: parent_ix,
            source: child.as_str().into(),
          },
          "~" => L::Unquote {
            form_ix: parent_ix,
            source: child.as_str().into(),
          },
          _ => unreachable!("quote-unquote prefix case analysis"),
        }),
        R::form => self.form(child, child_ix),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::QuoteUnquoteForm,
        }),
      }
    }
  }

  fn preforms(&mut self, parent: Pair, current: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),
        R::discarded_form => self.discarded_form(child),
        R::meta_form => self.meta_form(child, current),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::Preforms,
        }),
      }
    }
  }

  fn discarded_form(&mut self, parent: Pair) {
    let form_ix = self.next_form_ix(None);
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),
        R::discard_prefix => self.push(L::Discard {
          form_ix,
          source: child.as_str().into(),
        }),
        R::preform => self.preforms(child, form_ix),
        R::form => self.form(child, form_ix),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::DiscardedForm,
        }),
      }
    }
  }

  fn meta_form(&mut self, parent: Pair, form_ix: FormIx) {
    let data_ix = self.next_form_ix(None);
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),
        R::meta_prefix => self.push(L::Meta {
          form_ix,
          data_ix,
          source: child.as_str().into(),
        }),
        R::form => self.form(child, data_ix),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::MetaForm,
        }),
      }
    }
  }

  fn expr(&mut self, parent: Pair, form_ix: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),
        R::nil => self.push(L::Nil {
          form_ix,
          source: child.as_str().into(),
        }),
        R::boolean => self.push(L::Boolean {
          form_ix,
          value: child.as_str() == "true",
          source: child.as_str().into(),
        }),
        R::number => self.number(child, form_ix),
        R::char => self.char(child, form_ix),
        R::string => self.string(child, form_ix),
        R::regex => self.regex(child, form_ix),
        R::symbolic_value => self.symbolic_value(child, form_ix),
        R::symbol => self.symbol(child, form_ix),
        R::keyword => self.keyword(child, form_ix),
        R::list => self.list(child, form_ix),
        R::vector => self.vector(child, form_ix),
        R::anonymous_fn => self.anonymous_fn(child, form_ix),
        R::set => self.set(child, form_ix),
        R::map => self.map(child, form_ix),
        R::reader_conditional => self.reader_conditional(child, form_ix),
        R::tagged_literal => self.tagged_literal(child, form_ix),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::Expr,
        }),
      }
    }
  }

  fn char(&mut self, parent: Pair, form_ix: FormIx) {
    let source: Rc<str> = parent.as_str().to_string().into();
    // XXX(soija) This should be refactored so that we match the character
    //            literal only once and the assert that there are no remaining
    //            pairs left.  Among other things it would get rid of cloning
    //            `source`.
    for child in parent.into_inner() {
      match child.as_rule() {
        R::char_name => self.push(L::Char {
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
          source: source.clone(),
        }),
        R::char_octal => self.push(L::Char {
          form_ix,
          syntax: CharSyntax::Octal,
          value: char::from_u32(
            u32::from_str_radix(child.as_str(), 8).unwrap(),
          )
          .unwrap(),
          source: source.clone(),
        }),
        R::char_code_point => self.push(L::Char {
          form_ix,
          syntax: CharSyntax::CodePoint,
          value: char::from_u32(
            u32::from_str_radix(child.as_str(), 16).unwrap(),
          )
          .unwrap(),
          source: source.clone(),
        }),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::Char,
        }),
      }
    }
  }

  fn number(&mut self, parent: Pair, form_ix: FormIx) {
    let mut positive = true;
    let literal = parent.as_str();
    for child in parent.into_inner() {
      match child.as_rule() {
        R::sign => positive = child.as_str() == "+",
        R::unsigned_bigfloat => {
          self.unsigned_floats(child, form_ix, literal, true)
        }
        R::unsigned_float => {
          self.unsigned_floats(child, form_ix, literal, false)
        }
        R::unsigned_ratio => {
          self.unsigned_ratio(child, form_ix, literal, positive)
        }
        R::unsigned_radix_int => {
          self.unsigned_radix_int(child, form_ix, literal, positive)
        }
        R::unsigned_int => self.unsigned_int(child, form_ix, literal, positive),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::Number,
        }),
      }
    }
  }

  fn unsigned_floats(
    &mut self,
    _parent: Pair,
    form_ix: FormIx,
    literal: &str,
    big: bool,
  ) {
    self.push(if big {
      L::Numeric {
        form_ix,
        class: NumberClass::BigDecimal,
        value: NumericValue::Float {
          value: literal[..literal.len() - 1].into(),
        },
        source: literal.into(),
      }
    } else {
      L::Numeric {
        form_ix,
        class: NumberClass::Double,
        value: NumericValue::Float {
          value: literal.into(),
        },
        source: literal.into(),
      }
    })
  }

  fn unsigned_ratio(
    &mut self,
    parent: Pair,
    form_ix: FormIx,
    literal: &str,
    positive: bool,
  ) {
    let mut numerator = None;
    let mut denominator = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        R::numerator => numerator = Some(child.as_str()),
        R::denominator => denominator = Some(child.as_str()),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::UnsignedRatio,
        }),
      }
    }
    self.push(L::Numeric {
      form_ix,
      source: literal.into(),
      class: NumberClass::Ratio,
      value: NumericValue::Fraction {
        positive,
        numerator: numerator.unwrap().into(),
        denominator: denominator.unwrap().into(),
      },
    })
  }
  fn unsigned_radix_int(
    &mut self,
    parent: Pair,
    form_ix: FormIx,
    literal: &str,
    positive: bool,
  ) {
    let mut radix = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        R::radix => radix = Some(child.as_str()),
        R::radix_digits => self.push(L::Numeric {
          form_ix,
          source: literal.into(),
          class: NumberClass::Long,
          value: NumericValue::Int {
            positive,
            radix: radix.unwrap().parse::<u32>().unwrap(),
            value: child.as_str().into(),
          },
        }),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::UnsignedRadixInt,
        }),
      }
    }
  }

  fn unsigned_int(
    &mut self,
    parent: Pair,
    form_ix: FormIx,
    literal: &str,
    positive: bool,
  ) {
    let mut class = NumberClass::Long;
    let mut value = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        R::oct_digits => {
          value = Some(NumericValue::Int {
            positive,
            radix: 8,
            value: child.as_str().into(),
          })
        }
        R::hex_digits => {
          value = Some(NumericValue::Int {
            positive,
            radix: 16,
            value: child.as_str().into(),
          })
        }
        R::unsigned_dec => {
          value = Some(NumericValue::Int {
            positive,
            radix: 10,
            value: child.as_str().into(),
          })
        }
        R::bigint_suffix => class = NumberClass::BigInt,
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::UnsignedInt,
        }),
      }
    }
    self.push(L::Numeric {
      form_ix,
      source: literal.into(),
      class,
      value: value.unwrap(),
    })
  }

  fn string(&mut self, parent: Pair, form_ix: FormIx) {
    let mut fragments = Vec::new();
    let literal = parent.as_str();
    for child in parent.into_inner() {
      match child.as_rule() {
        R::unescaped => fragments.push(StringFragment::Unescaped {
          value: child.as_str().into(),
        }),
        R::esc_char => {
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
        R::esc_octet => {
          let value = &child.as_str()[1..];
          let code = u32::from_str_radix(value, 8).unwrap();
          fragments.push(StringFragment::Escaped { code })
        }
        R::esc_code_point => {
          let value = &child.as_str()[2..];
          let code = u32::from_str_radix(value, 16).unwrap();
          fragments.push(StringFragment::Escaped { code })
        }
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::String,
        }),
      }
    }
    fragments.shrink_to_fit();
    self.push(L::String {
      form_ix,
      source: literal.into(),
      value: fragments.into_boxed_slice(),
    });
  }

  fn regex(&mut self, parent: Pair, form_ix: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::regex_content => self.push(L::Regex {
          form_ix,
          source: child.as_str().into(),
        }),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::Regex,
        }),
      }
    }
  }

  fn symbolic_value(&mut self, parent: Pair, form_ix: FormIx) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),
        R::symbolic_value_prefix => self.push(L::SymbolicValuePrefix {
          form_ix,
          source: child.as_str().into(),
        }),
        R::unqualified_symbol => self.push(L::SymbolicValue {
          form_ix,
          value: match child.as_str() {
            "Inf" => SymbolicValue::PosInf,
            "-Inf" => SymbolicValue::NegInf,
            "NaN" => SymbolicValue::NaN,
            _ => SymbolicValue::Other(child.as_str().into()),
          },
          source: child.as_str().into(),
        }),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::SymbolicValue,
        }),
      }
    }
  }

  fn symbol(&mut self, parent: Pair, form_ix: FormIx) {
    let source: Rc<str> = parent.as_str().to_string().into();
    let mut namespace = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        R::namespace => namespace = Some(child.as_str().into()),
        R::qualified_symbol | R::unqualified_symbol => self.push(L::Symbol {
          form_ix,
          namespace: namespace.clone(),
          name: child.as_str().into(),
          source: source.clone(),
        }),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::Symbol,
        }),
      }
    }
  }

  fn tag(&mut self, parent: Pair, form_ix: FormIx) {
    let source: Rc<str> = parent.as_str().to_string().into();
    let mut namespace = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        R::namespace => namespace = Some(child.as_str().into()),
        R::qualified_symbol | R::unqualified_symbol => self.push(L::Tag {
          form_ix,
          namespace: namespace.clone(),
          name: child.as_str().into(),
          source: source.clone(),
        }),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::Tag,
        }),
      }
    }
  }

  fn keyword(&mut self, parent: Pair, form_ix: FormIx) {
    let source: Rc<str> = parent.as_str().to_string().into();
    let mut namespace = None;
    let mut alias = false;
    for child in parent.into_inner() {
      match child.as_rule() {
        R::keyword_prefix => alias = child.as_str() == "::",
        R::namespace => namespace = Some(child.as_str().into()),
        R::qualified_symbol | R::unqualified_keyword => self.push(L::Keyword {
          form_ix,
          alias,
          namespace: namespace.clone(),
          name: child.as_str().into(),
          source: source.clone(),
        }),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::Keyword,
        }),
      }
    }
  }

  fn list(&mut self, parent: Pair, parent_ix: FormIx) {
    let source = parent.as_str();
    self.body(
      parent,
      parent_ix,
      L::StartList {
        form_ix: parent_ix,
        source: source[..1].into(),
      },
      L::EndList {
        form_ix: parent_ix,
        source: source[source.len() - 1..].into(),
      },
    );
  }

  fn vector(&mut self, parent: Pair, parent_ix: FormIx) {
    let source = parent.as_str();
    self.body(
      parent,
      parent_ix,
      L::StartVector {
        form_ix: parent_ix,
        source: source[..1].into(),
      },
      L::EndVector {
        form_ix: parent_ix,
        source: source[source.len() - 1..].into(),
      },
    );
  }

  fn anonymous_fn(&mut self, parent: Pair, parent_ix: FormIx) {
    let source = parent.as_str();
    self.body(
      parent,
      parent_ix,
      L::StartAnonymousFn {
        form_ix: parent_ix,
        source: source[..2].into(),
      },
      L::EndAnonymousFn {
        form_ix: parent_ix,
        source: source[source.len() - 1..].into(),
      },
    );
  }

  fn set(&mut self, parent: Pair, parent_ix: FormIx) {
    let source = parent.as_str();
    self.body(
      parent,
      parent_ix,
      L::StartSet {
        form_ix: parent_ix,
        source: source[..2].into(),
      },
      L::EndSet {
        form_ix: parent_ix,
        source: source[source.len() - 1..].into(),
      },
    );
  }

  fn map(&mut self, parent: Pair, form_ix: FormIx) {
    let mut alias = false;
    let mut namespace = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),
        R::map_qualifier => {
          self.push(L::MapQualifier {
            form_ix,
            source: child.as_str().into(),
          });
          for child2 in child.into_inner() {
            match child2.as_rule() {
              R::map_qualifier_prefix => alias = child2.as_str() == "#::",
              R::namespace => namespace = Some(child2.as_str().into()),
              _ => self.push(L::Residual {
                pair: format!("{:#?}", child2).into_boxed_str(),
                ploc: ParserLoc::MapQualifier,
              }),
            }
          }
        }
        R::unqualified_map => {
          let source = child.as_str();
          self.body(
            child,
            form_ix,
            L::StartMap {
              form_ix,
              alias,
              namespace: namespace.clone(),
              source: source[..1].into(),
            },
            L::EndMap {
              form_ix,
              source: source[source.len() - 1..].into(),
            },
          )
        }
        R::discarded_form => self.discarded_form(child),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::Map,
        }),
      }
    }
  }

  fn reader_conditional(&mut self, parent: Pair, form_ix: FormIx) {
    let mut splicing = false;
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),
        R::reader_conditional_prefix => splicing = child.as_str() == "#?@",
        R::reader_conditional_body => {
          let source = child.as_str();
          self.body(
            child,
            form_ix,
            L::StartReaderConditional {
              form_ix,
              splicing,
              source: source[1..].into(),
            },
            L::EndReaderConditional {
              form_ix,
              source: source[source.len() - 1..].into(),
            },
          )
        }
        R::discarded_form => self.discarded_form(child),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::ReaderConditional,
        }),
      }
    }
  }

  fn body(
    &mut self,
    parent: Pair,
    parent_ix: FormIx,
    start_lexeme: Lexeme,
    end_lexeme: Lexeme,
  ) {
    self.push(start_lexeme);
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),
        R::form => {
          let child_ix = self.next_form_ix(Some(parent_ix));
          self.form(child, child_ix)
        }
        R::discarded_form => self.discarded_form(child),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::Body,
        }),
      }
    }
    self.push(end_lexeme);
  }

  fn tagged_literal(&mut self, parent: Pair, form_ix: FormIx) {
    let tag_ix = self.next_form_ix(Some(form_ix));
    let arg_ix = self.next_form_ix(Some(form_ix));
    self.push(L::TaggedLiteral {
      form_ix,
      tag_ix,
      arg_ix,
      // XXX(soija) Aw, this is a hack.  We want to capture only the prefix as
      //            the rest of the tagged literal is captured be the following
      //            lexemes.
      source: parent.as_str()[..1].into(),
    });
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),
        R::tagged_literal_tag => {
          for child2 in child.into_inner() {
            match child2.as_rule() {
              R::COMMENT => self.comment(child2),
              R::WHITESPACE => self.whitespace(child2),
              R::preform => self.preforms(child2, tag_ix),
              R::symbol => self.tag(child2, tag_ix),
              _ => self.push(L::Residual {
                pair: format!("{:#?}", child2).into_boxed_str(),
                ploc: ParserLoc::TaggedLiteralTag,
              }),
            }
          }
        }
        R::form => self.form(child, arg_ix),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::TaggedLiteralValue,
        }),
      }
    }
  }
}
