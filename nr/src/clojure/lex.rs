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

pub type Ix = u32;

#[derive(Debug)]
pub enum Lexeme {
  /// Whitespace
  Whitespace {
    /// The index of the parent form within which the whitespace occurs
    parent_ix: Ix,
    source: Rc<str>,
  },
  /// Comment
  Comment {
    /// The index of the parent form within which the comment occurs
    parent_ix: Ix,
    /// The original source for the comment line.
    ///
    /// Effectively the end of the line starting from the comment marker up to
    /// but excluding the line break.  Note that in addition to `;` the comment
    /// marker can be `#!`.
    source: Rc<str>,
  },
  /// Meta data prefix (`^` or `#^`)
  Meta {
    /// The index of the parent form containing this.
    parent_ix: Ix,
    /// The index of this form whole composite form.
    form_ix: Ix,
    /// The index of the form containing the meta data.
    metaform_ix: Ix,
    /// The index of the form to which the meta data is attached.
    subform_ix: Ix,
    /// The original source for the meta data prefix.  Effectively either `"^"`
    /// or `"#^"`.
    source: Rc<str>,
  },
  Discard {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  Quote {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  VarQuote {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  Synquote {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  Unquote {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  SplicingUnquote {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  Nil {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  Boolean {
    parent_ix: Ix,
    form_ix: Ix,
    value: bool,
    source: Rc<str>,
  },
  Numeric {
    parent_ix: Ix,
    form_ix: Ix,
    class: NumberClass,
    value: NumericValue,
    source: Rc<str>,
  },
  Char {
    parent_ix: Ix,
    form_ix: Ix,
    syntax: CharSyntax,
    value: char,
    source: Rc<str>,
  },
  String {
    parent_ix: Ix,
    form_ix: Ix,
    value: Box<[StringFragment]>,
    source: Rc<str>,
  },
  Regex {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  SymbolicValuePrefix {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  SymbolicValue {
    parent_ix: Ix,
    form_ix: Ix,
    value: SymbolicValue,
    source: Rc<str>,
  },
  Symbol {
    parent_ix: Ix,
    form_ix: Ix,
    namespace: Option<Rc<str>>,
    name: Rc<str>,
    source: Rc<str>,
  },
  /// The tag of a tagged literal
  ///
  /// This is essentially an unqualified or qualified symbol.  However, this
  /// is kept distinct from an ordinary symbol in order to give this little bit
  /// of contextual information.  This is convenient, for example, when doing
  /// syntax coloring.
  Tag {
    parent_ix: Ix,
    form_ix: Ix,
    namespace: Option<Rc<str>>,
    name: Rc<str>,
    source: Rc<str>,
  },
  Keyword {
    parent_ix: Ix,
    form_ix: Ix,
    alias: bool,
    namespace: Option<Rc<str>>,
    name: Rc<str>,
    source: Rc<str>,
  },
  StartList {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  EndList {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  StartVector {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  EndVector {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  StartSet {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  EndSet {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  MapQualifier {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  StartMap {
    parent_ix: Ix,
    form_ix: Ix,
    alias: bool,
    namespace: Option<Rc<str>>,
    source: Rc<str>,
  },
  EndMap {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  StartAnonymousFn {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  EndAnonymousFn {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  ReaderConditionalPrefix {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  StartReaderConditional {
    parent_ix: Ix,
    form_ix: Ix,
    splicing: bool,
    source: Rc<str>,
  },
  EndReaderConditional {
    parent_ix: Ix,
    form_ix: Ix,
    source: Rc<str>,
  },
  TaggedLiteral {
    parent_ix: Ix,
    form_ix: Ix,
    tag_ix: Ix,
    arg_ix: Ix,
    source: Rc<str>,
  },
  Residual {
    pair: Box<str>,
    ploc: ParserLoc,
  },
}

#[derive(Debug)]
pub enum Token {
  Whitespace,
  Comment,
  Meta {
    metaform_ix: Ix,
    subform_ix: Ix,
  },
  Discard,
  Quote,
  VarQuote,
  Synquote,
  Unquote,
  SplicingUnquote,
  Nil,
  Boolean {
    value: bool,
  },
  Numeric {
    class: NumberClass,
    value: NumericValue,
  },
  Char {
    syntax: CharSyntax,
    value: char,
  },
  String {
    value: Box<[StringFragment]>,
  },
  Regex,
  SymbolicValuePrefix,
  SymbolicValue {
    value: SymbolicValue,
  },
  Symbol {
    namespace: Option<Rc<str>>,
    name: Rc<str>,
  },
  /// The tag of a tagged literal
  ///
  /// This is essentially an unqualified or qualified symbol.  However, this
  /// is kept distinct from an ordinary symbol in order to give this little bit
  /// of contextual information.  This is convenient, for example, when doing
  /// syntax coloring.
  Tag {
    namespace: Option<Rc<str>>,
    name: Rc<str>,
  },
  Keyword {
    alias: bool,
    namespace: Option<Rc<str>>,
    name: Rc<str>,
  },
  StartList,
  EndList,
  StartVector,
  EndVector,
  StartSet,
  EndSet,
  MapQualifier,
  StartMap {
    alias: bool,
    namespace: Option<Rc<str>>,
  },
  EndMap,
  StartAnonymousFn,
  EndAnonymousFn,
  ReaderConditionalPrefix,
  StartReaderConditional {
    splicing: bool,
  },
  EndReaderConditional,
  TaggedLiteral {
    tag_ix: Ix,
    arg_ix: Ix,
  },
  Residual {
    pair: Box<str>,
    ploc: ParserLoc,
  },
}

impl Lexeme {
  pub fn form_ix(&self) -> Ix {
    match self {
      L::Whitespace { form_ix, .. }
      | L::Comment { form_ix, .. }
      | L::Meta { form_ix, .. }
      | L::Discard { form_ix, .. }
      | L::Quote { form_ix, .. }
      | L::VarQuote { form_ix, .. }
      | L::Synquote { form_ix, .. }
      | L::Unquote { form_ix, .. }
      | L::SplicingUnquote { form_ix, .. }
      | L::Nil { form_ix, .. }
      | L::Boolean { form_ix, .. }
      | L::Numeric { form_ix, .. }
      | L::Char { form_ix, .. }
      | L::String { form_ix, .. }
      | L::Regex { form_ix, .. }
      | L::SymbolicValuePrefix { form_ix, .. }
      | L::SymbolicValue { form_ix, .. }
      | L::Symbol { form_ix, .. }
      | L::Tag { form_ix, .. }
      | L::Keyword { form_ix, .. }
      | L::StartList { form_ix, .. }
      | L::EndList { form_ix, .. }
      | L::StartVector { form_ix, .. }
      | L::EndVector { form_ix, .. }
      | L::StartSet { form_ix, .. }
      | L::EndSet { form_ix, .. }
      | L::MapQualifier { form_ix, .. }
      | L::StartMap { form_ix, .. }
      | L::EndMap { form_ix, .. }
      | L::StartAnonymousFn { form_ix, .. }
      | L::EndAnonymousFn { form_ix, .. }
      | L::ReaderConditionalPrefix { form_ix, .. }
      | L::StartReaderConditional { form_ix, .. }
      | L::EndReaderConditional { form_ix, .. }
      | L::TaggedLiteral { form_ix, .. } => *form_ix,
      L::Residual { .. } => panic!("form of residual lexeme queried"),
    }
  }

  pub fn parent_ix(&self) -> Ix {
    match self {
      L::Whitespace { parent_ix, .. }
      | L::Comment { parent_ix, .. }
      | L::Meta { parent_ix, .. }
      | L::Discard { parent_ix, .. }
      | L::Quote { parent_ix, .. }
      | L::VarQuote { parent_ix, .. }
      | L::Synquote { parent_ix, .. }
      | L::Unquote { parent_ix, .. }
      | L::SplicingUnquote { parent_ix, .. }
      | L::Nil { parent_ix, .. }
      | L::Boolean { parent_ix, .. }
      | L::Numeric { parent_ix, .. }
      | L::Char { parent_ix, .. }
      | L::String { parent_ix, .. }
      | L::Regex { parent_ix, .. }
      | L::SymbolicValuePrefix { parent_ix, .. }
      | L::SymbolicValue { parent_ix, .. }
      | L::Symbol { parent_ix, .. }
      | L::Tag { parent_ix, .. }
      | L::Keyword { parent_ix, .. }
      | L::StartList { parent_ix, .. }
      | L::EndList { parent_ix, .. }
      | L::StartVector { parent_ix, .. }
      | L::EndVector { parent_ix, .. }
      | L::StartSet { parent_ix, .. }
      | L::EndSet { parent_ix, .. }
      | L::MapQualifier { parent_ix, .. }
      | L::StartMap { parent_ix, .. }
      | L::EndMap { parent_ix, .. }
      | L::StartAnonymousFn { parent_ix, .. }
      | L::EndAnonymousFn { parent_ix, .. }
      | L::ReaderConditionalPrefix { parent_ix, .. }
      | L::StartReaderConditional { parent_ix, .. }
      | L::EndReaderConditional { parent_ix, .. }
      | L::TaggedLiteral { parent_ix, .. } => *parent_ix,
      L::Residual { .. } => panic!("parent of residual lexeme queried"),
    }
  }
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

  fn next_form_ix(&mut self) -> Ix {
    self.form_count += 1;
    return self.form_count;
  }

  fn whitespace(&mut self, pair: Pair, parent_ix: Ix) {
    self.push(L::Whitespace {
      parent_ix,
      source: pair.as_str().into(),
    });
  }

  fn comment(&mut self, pair: Pair, parent_ix: Ix) {
    self.push(L::Comment {
      parent_ix,
      source: pair.as_str().into(),
    });
  }

  fn top_level(&mut self, parent: Pair) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child, 0),
        R::WHITESPACE => self.whitespace(child, 0),

        R::form => self.form(child, 0, self.next_form_ix()),

        R::EOI => (),
        _ => self.push(L::Residual {
          pair: format!("{:?}", child).into_boxed_str(),
          ploc: ParserLoc::TopLevel,
        }),
      }
    }
  }

  fn form(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child, parent_ix),
        R::WHITESPACE => self.whitespace(child, parent_ix),

        R::quote_unquote_form => {
          self.quote_unquote_form(child, parent_ix, form_ix)
        }
        R::meta_data_form => self.meta_data_form(child, parent_ix, form_ix),
        R::discarded_form => {
          self.discarded_form(child, parent_ix, self.next_form_ix())
        }

        R::form => self.form(child, parent_ix, form_ix),
        R::expr => self.expr(child, parent_ix, form_ix),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::Form,
        }),
      }
    }
  }

  fn quote_unquote_form(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    for child in parent.into_inner() {
      match child.as_rule() {
        // Leading whitespace and comments were consumed before colling this
        // method.  Hence, these are between the prefix and the quoted form itself.
        R::COMMENT => self.comment(child, form_ix),
        R::WHITESPACE => self.whitespace(child, form_ix),

        R::quote_unquote_prefix => self.push(match child.as_str() {
          "'" => L::Quote {
            parent_ix,
            form_ix,
            source: child.as_str().into(),
          },
          "#'" => L::VarQuote {
            parent_ix,
            form_ix,
            source: child.as_str().into(),
          },
          "`" => L::Synquote {
            parent_ix,
            form_ix,
            source: child.as_str().into(),
          },
          "~@" => L::SplicingUnquote {
            parent_ix,
            form_ix,
            source: child.as_str().into(),
          },
          "~" => L::Unquote {
            parent_ix,
            form_ix,
            source: child.as_str().into(),
          },
          _ => unreachable!("quote-unquote prefix case analysis"),
        }),

        // XXX(soija) Could have a state machine here to guard against more than
        //            one form
        R::form => self.form(child, form_ix, self.next_form_ix()),

        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::QuoteUnquoteForm,
        }),
      }
    }
  }

  fn meta_data_form(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    let metaform_ix = self.next_form_ix();
    let subform_ix = self.next_form_ix();

    enum S {
      WaitingMetaForm,
      WaitingSubForm,
      Done,
    }
    let mut state = S::WaitingMetaForm;

    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child, form_ix),
        R::WHITESPACE => self.whitespace(child, form_ix),

        R::meta_prefix => self.push(L::Meta {
          parent_ix,
          form_ix,
          subform_ix,
          metaform_ix,
          source: child.as_str().into(),
        }),

        R::form => match state {
          S::WaitingMetaForm => {
            self.form(child, form_ix, metaform_ix);
            state = S::WaitingSubForm;
          }
          S::WaitingSubForm => {
            self.form(child, form_ix, subform_ix);
            state = S::Done;
          }
          S::Done => unreachable!("borken"),
        },

        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::MetaForm,
        }),
      }
    }
  }

  fn discarded_form(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child, form_ix),
        R::WHITESPACE => self.whitespace(child, form_ix),

        R::discard_prefix => self.push(L::Discard {
          parent_ix,
          form_ix,
          source: child.as_str().into(),
        }),
        R::form => self.form(child, form_ix, self.next_form_ix()),

        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::DiscardedForm,
        }),
      }
    }
  }

  fn expr(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child, form_ix),
        R::WHITESPACE => self.whitespace(child, form_ix),

        R::nil => self.push(L::Nil {
          parent_ix,
          form_ix,
          source: child.as_str().into(),
        }),
        R::boolean => self.push(L::Boolean {
          parent_ix,
          form_ix,
          value: child.as_str() == "true",
          source: child.as_str().into(),
        }),
        R::number => self.number(child, parent_ix, form_ix),
        R::char => self.char(child, parent_ix, form_ix),
        R::string => self.string(child, parent_ix, form_ix),
        R::regex => self.regex(child, parent_ix, form_ix),
        R::symbolic_value => self.symbolic_value(child, parent_ix, form_ix),
        R::symbol => self.symbol(child, parent_ix, form_ix),
        R::keyword => self.keyword(child, parent_ix, form_ix),
        R::list => self.list(child, parent_ix, form_ix),
        R::vector => self.vector(child, parent_ix, form_ix),
        R::anonymous_fn => self.anonymous_fn(child, parent_ix, form_ix),
        R::set => self.set(child, parent_ix, form_ix),
        R::map => self.map(child, parent_ix, form_ix),
        R::reader_conditional => {
          self.reader_conditional(child, parent_ix, form_ix)
        }
        R::tagged_literal => self.tagged_literal(child, parent_ix, form_ix),

        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::Expr,
        }),
      }
    }
  }

  fn char(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    let source: Rc<str> = parent.as_str().to_string().into();
    // XXX(soija) This should be refactored so that we match the character
    //            literal only once and the assert that there are no remaining
    //            pairs left.  Among other things it would get rid of cloning
    //            `source`.
    for child in parent.into_inner() {
      match child.as_rule() {
        R::char_name => self.push(L::Char {
          parent_ix,
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
          parent_ix,
          form_ix,
          syntax: CharSyntax::Octal,
          value: char::from_u32(
            u32::from_str_radix(child.as_str(), 8).unwrap(),
          )
          .unwrap(),
          source: source.clone(),
        }),
        R::char_code_point => self.push(L::Char {
          parent_ix,
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

  fn number(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    let mut positive = true;
    let literal = parent.as_str();
    for child in parent.into_inner() {
      match child.as_rule() {
        R::sign => positive = child.as_str() == "+",
        R::unsigned_bigfloat => {
          self.unsigned_floats(child, parent_ix, form_ix, literal, true)
        }
        R::unsigned_float => {
          self.unsigned_floats(child, parent_ix, form_ix, literal, false)
        }
        R::unsigned_ratio => {
          self.unsigned_ratio(child, parent_ix, form_ix, literal, positive)
        }
        R::unsigned_radix_int => {
          self.unsigned_radix_int(child, parent_ix, form_ix, literal, positive)
        }
        R::unsigned_int => {
          self.unsigned_int(child, parent_ix, form_ix, literal, positive)
        }
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
    parent_ix: Ix,
    form_ix: Ix,
    literal: &str,
    big: bool,
  ) {
    self.push(if big {
      L::Numeric {
        parent_ix,
        form_ix,
        class: NumberClass::BigDecimal,
        value: NumericValue::Float {
          value: literal[..literal.len() - 1].into(),
        },
        source: literal.into(),
      }
    } else {
      L::Numeric {
        parent_ix,
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
    parent_ix: Ix,
    form_ix: Ix,
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
      parent_ix,
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
    parent_ix: Ix,
    form_ix: Ix,
    literal: &str,
    positive: bool,
  ) {
    let mut radix = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        R::radix => radix = Some(child.as_str()),
        R::radix_digits => self.push(L::Numeric {
          parent_ix,
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
    parent_ix: Ix,
    form_ix: Ix,
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
      parent_ix,
      form_ix,
      source: literal.into(),
      class,
      value: value.unwrap(),
    })
  }

  fn string(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
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
      parent_ix,
      form_ix,
      source: literal.into(),
      value: fragments.into_boxed_slice(),
    });
  }

  fn regex(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::regex_content => self.push(L::Regex {
          parent_ix,
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

  fn symbolic_value(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child, form_ix),
        R::WHITESPACE => self.whitespace(child, form_ix),

        R::symbolic_value_prefix => self.push(L::SymbolicValuePrefix {
          parent_ix,
          form_ix,
          source: child.as_str().into(),
        }),
        R::unqualified_symbol => self.push(L::SymbolicValue {
          parent_ix,
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

  fn symbol(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    let source: Rc<str> = parent.as_str().to_string().into();
    let mut namespace = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        R::namespace => namespace = Some(child.as_str().into()),
        R::qualified_symbol | R::unqualified_symbol => self.push(L::Symbol {
          parent_ix,
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

  fn tag(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    let source: Rc<str> = parent.as_str().to_string().into();
    let mut namespace = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        R::namespace => namespace = Some(child.as_str().into()),
        R::qualified_symbol | R::unqualified_symbol => self.push(L::Tag {
          parent_ix,
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

  fn keyword(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    let source: Rc<str> = parent.as_str().to_string().into();
    let mut namespace = None;
    let mut alias = false;
    for child in parent.into_inner() {
      match child.as_rule() {
        R::keyword_prefix => alias = child.as_str() == "::",
        R::namespace => namespace = Some(child.as_str().into()),
        R::qualified_symbol | R::unqualified_keyword => self.push(L::Keyword {
          parent_ix,
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

  fn list(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    let source = parent.as_str();
    self.body(
      parent,
      form_ix,
      L::StartList {
        parent_ix,
        form_ix,
        source: source[..1].into(),
      },
      L::EndList {
        parent_ix,
        form_ix,
        source: source[source.len() - 1..].into(),
      },
    );
  }

  fn vector(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    let source = parent.as_str();
    self.body(
      parent,
      form_ix,
      L::StartVector {
        parent_ix,
        form_ix,
        source: source[..1].into(),
      },
      L::EndVector {
        parent_ix,
        form_ix,
        source: source[source.len() - 1..].into(),
      },
    );
  }

  fn anonymous_fn(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    let source = parent.as_str();
    self.body(
      parent,
      form_ix,
      L::StartAnonymousFn {
        parent_ix,
        form_ix,
        source: source[..2].into(),
      },
      L::EndAnonymousFn {
        parent_ix,
        form_ix,
        source: source[source.len() - 1..].into(),
      },
    );
  }

  fn set(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    let source = parent.as_str();
    self.body(
      parent,
      form_ix,
      L::StartSet {
        parent_ix,
        form_ix,
        source: source[..2].into(),
      },
      L::EndSet {
        parent_ix,
        form_ix,
        source: source[source.len() - 1..].into(),
      },
    );
  }

  fn map(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    let mut alias = false;
    let mut namespace = None;
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child, parent_ix),
        R::WHITESPACE => self.whitespace(child, parent_ix),

        R::map_qualifier => {
          self.push(L::MapQualifier {
            parent_ix,
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
              parent_ix,
              form_ix,
              alias,
              namespace: namespace.clone(),
              source: source[..1].into(),
            },
            L::EndMap {
              parent_ix,
              form_ix,
              source: source[source.len() - 1..].into(),
            },
          )
        }

        // R::discarded_form => {
        //   self.discarded_form(child, form_ix, self.next_form_ix())
        // }
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::Map,
        }),
      }
    }
  }

  fn reader_conditional(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    let mut splicing = false;
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child, form_ix),
        R::WHITESPACE => self.whitespace(child, form_ix),

        R::reader_conditional_prefix => splicing = child.as_str() == "#?@",
        R::reader_conditional_body => {
          let source = child.as_str();
          self.body(
            child,
            form_ix,
            L::StartReaderConditional {
              parent_ix,
              form_ix,
              splicing,
              source: source[1..].into(),
            },
            L::EndReaderConditional {
              parent_ix,
              form_ix,
              source: source[source.len() - 1..].into(),
            },
          )
        }

        // R::discarded_form => self.discarded_form(child),
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
    parent_ix: Ix,
    start_lexeme: Lexeme,
    end_lexeme: Lexeme,
  ) {
    self.push(start_lexeme);
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child, parent_ix),
        R::WHITESPACE => self.whitespace(child, parent_ix),

        R::form => self.form(child, parent_ix, self.next_form_ix()),
        R::discarded_form => {
          self.discarded_form(child, parent_ix, self.next_form_ix())
        }

        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::Body,
        }),
      }
    }
    self.push(end_lexeme);
  }

  fn tagged_literal(&mut self, parent: Pair, parent_ix: Ix, form_ix: Ix) {
    let tag_ix = self.next_form_ix();
    let arg_ix = self.next_form_ix();

    self.push(L::TaggedLiteral {
      parent_ix,
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
        R::COMMENT => self.comment(child, form_ix),
        R::WHITESPACE => self.whitespace(child, form_ix),

        R::tagged_literal_tag => {
          for child2 in child.into_inner() {
            match child2.as_rule() {
              // R::COMMENT => self.comment(child2),
              // R::WHITESPACE => self.whitespace(child2),
              // R::preform => self.preforms(child2, tag_ix),
              R::symbol => self.tag(child2, form_ix, tag_ix),

              _ => self.push(L::Residual {
                pair: format!("{:#?}", child2).into_boxed_str(),
                ploc: ParserLoc::TaggedLiteralTag,
              }),
            }
          }
        }

        R::form => self.form(child, form_ix, arg_ix),
        _ => self.push(L::Residual {
          pair: format!("{:#?}", child).into_boxed_str(),
          ploc: ParserLoc::TaggedLiteralValue,
        }),
      }
    }
  }
}
