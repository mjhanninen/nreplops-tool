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

use std::{iter::Peekable, rc::Rc};

use thiserror::Error;

use super::pest_grammar::*;

use Lexeme as L;
use Rule as R;
use Token as T;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Pest error: {0}")]
  Pest(#[from] pest::error::Error<Rule>),
}

pub type Ix = u32;

#[derive(Clone, Debug)]
pub struct Lexeme {
  pub parent_ix: Ix,
  pub form_ix: Ix,
  pub token: Token,
  pub source: Option<Source>,
}

#[derive(Clone, Debug)]
pub struct Source {
  pub line: u32,
  pub column: u32,
  pub str: Rc<str>,
}

#[derive(Clone, Debug)]
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
    raw_value: Box<str>,
    value: Box<[StringFragment]>,
  },
  Regex {
    raw_value: Box<str>,
  },
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
  MapQualifier {
    alias: bool,
    namespace: Option<Rc<str>>,
  },
  StartMap,
  EndMap,
  StartAnonymousFn,
  EndAnonymousFn,
  ReaderConditionalPrefix {
    splicing: bool,
  },
  StartReaderConditional,
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

#[derive(Debug)]
struct Helper {
  parent_stack: Vec<Ix>,
  form_count: u32,
  lexemes: Lexemes,
}

impl Default for Helper {
  fn default() -> Self {
    Self {
      parent_stack: vec![0],
      form_count: 0,
      lexemes: Vec::new(),
    }
  }
}

impl Helper {
  fn current_parent(&self) -> Ix {
    *self.parent_stack.last().expect("parent stack underflow")
  }

  fn push_parent(&mut self, form_ix: Ix) {
    self.parent_stack.push(form_ix);
  }

  fn pop_parent(&mut self) {
    self.parent_stack.pop().expect("parent stack underflow");
  }

  fn make_lexeme(
    &mut self,
    pair: Pair,
    token: Token,
    form_ix: Option<Ix>,
  ) -> Lexeme {
    let (l, c) = pair.line_col();
    L {
      parent_ix: self.current_parent(),
      form_ix: form_ix.unwrap_or_else(|| self.next_form_ix()),
      token,
      source: Some(Source {
        line: u32::try_from(l).expect("line number overflow"),
        column: u32::try_from(c).expect("column number overflow"),
        str: pair.as_str().into(),
      }),
    }
  }

  fn push_token(&mut self, pair: Pair, token: Token, form_ix: Option<Ix>) {
    let l = self.make_lexeme(pair, token, form_ix);
    self.push_lexeme(l);
  }

  fn push_lexeme(&mut self, lexeme: Lexeme) {
    self.lexemes.push(lexeme)
  }

  fn into_lexemes(mut self) -> Lexemes {
    self.lexemes.shrink_to_fit();
    self.lexemes
  }

  fn next_form_ix(&mut self) -> Ix {
    self.form_count += 1;
    self.form_count
  }

  fn whitespace(&mut self, pair: Pair) {
    self.push_token(pair, T::Whitespace, None)
  }

  fn comment(&mut self, pair: Pair) {
    self.push_token(pair, T::Comment, None);
  }

  fn comments_and_whitespace(&mut self, it: &mut Peekable<Pairs<'_>>) {
    while let Some(p) =
      it.next_if(|p| matches!(p.as_rule(), R::COMMENT | R::WHITESPACE))
    {
      match p.as_rule() {
        R::COMMENT => self.comment(p),
        R::WHITESPACE => self.whitespace(p),
        _ => unreachable!(),
      }
    }
  }

  fn top_level(&mut self, parent: Pair) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),

        R::form => {
          let ix = self.next_form_ix();
          self.form(child, ix)
        }

        R::EOI => (),

        _ => panic!("unexpected pair while parsing top level: {child:?}"),
      }
    }
  }

  fn form(&mut self, parent: Pair, form_ix: Ix) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),

        R::expr => self.expr(child, form_ix),
        R::quote_unquote_form => self.quote_unquote_form(child, form_ix),
        R::meta_data_form => self.meta_data_form(child, form_ix),

        R::discarded_form => {
          let ix = self.next_form_ix();
          self.discarded_form(child, ix)
        }
        // the form following one or more discarded forms
        R::form => self.form(child, form_ix),

        _ => panic!("unexpected pair while parsing form: {child:?}"),
      }
    }
  }

  fn quote_unquote_form(&mut self, parent: Pair, form_ix: Ix) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),

        R::quote_unquote_prefix => {
          let token = match child.as_str() {
            "'" => T::Quote,
            "#'" => T::VarQuote,
            "`" => T::Synquote,
            "~@" => T::SplicingUnquote,
            "~" => T::Unquote,
            _ => unreachable!("quote-unquote prefix case analysis"),
          };
          self.push_token(child, token, Some(form_ix));
          self.push_parent(form_ix)
        }

        // XXX(soija) Could have a state machine here to guard against more than
        //            one form
        R::form => {
          let ix = self.next_form_ix();
          self.form(child, ix)
        }

        _ => panic!(
          "unexpected pair while parsing quoted or unquoted form: {child:?}"
        ),
      }
    }

    self.pop_parent();
  }

  fn meta_data_form(&mut self, parent: Pair, form_ix: Ix) {
    let metaform_ix = self.next_form_ix();
    let subform_ix = self.next_form_ix();

    #[derive(PartialEq)]
    enum S {
      ExpectingMetaForm,
      ExpectingSubForm,
      Done,
    }
    let mut state = S::ExpectingMetaForm;

    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),

        R::meta_prefix => {
          self.push_token(
            child,
            T::Meta {
              subform_ix,
              metaform_ix,
            },
            Some(form_ix),
          );
          self.push_parent(form_ix);
        }

        R::form => match state {
          S::ExpectingMetaForm => {
            self.form(child, metaform_ix);
            state = S::ExpectingSubForm;
          }
          S::ExpectingSubForm => {
            self.form(child, subform_ix);
            state = S::Done;
          }
          S::Done => unreachable!("borken"),
        },

        _ => panic!("unexpected pair while parsing meta data form: {child:?}"),
      }
    }

    assert!(state == S::Done);

    self.pop_parent();
  }

  fn discarded_form(&mut self, pair: Pair, form_ix: Ix) {
    let mut it = pair.clone().into_inner().peekable();

    match it.next() {
      Some(p) if p.as_rule() == R::discard_prefix => {
        self.push_token(p, T::Discard, Some(form_ix));
        self.push_parent(form_ix);
      }
      Some(_) => {
        panic!("unexpected pair while parsing discarded form: {pair:?}")
      }
      None => panic!("missing pair while parsing discarded form: {pair:?}"),
    }

    self.comments_and_whitespace(&mut it);

    match it.next() {
      Some(p) if p.as_rule() == R::form => {
        let ix = self.next_form_ix();
        self.form(p, ix);
      }
      Some(_) => {
        panic!("unexpected pair while parsing discarded form: {pair:?}")
      }
      None => panic!("missing pair while parsing discarded form: {pair:?}"),
    }

    if it.next().is_some() {
      panic!("unexptected extra pair while parsing discarded form: {pair:?}");
    }

    self.pop_parent();
  }

  fn expr(&mut self, parent: Pair, form_ix: Ix) {
    for child in parent.into_inner() {
      match child.as_rule() {
        R::COMMENT => self.comment(child),
        R::WHITESPACE => self.whitespace(child),

        R::nil => self.push_token(child, T::Nil, Some(form_ix)),
        R::boolean => {
          let value = child.as_str() == "true";
          self.push_token(child, T::Boolean { value }, Some(form_ix))
        }
        R::number => self.number(child, form_ix),
        R::char => self.char(child, form_ix),
        R::string => self.string(child, form_ix),
        R::regex => self.regex(child, form_ix),
        R::symbolic_value => self.symbolic_value(child, form_ix),
        R::symbol => self.symbol_or_tag(child, form_ix, true),
        R::keyword => self.keyword(child, form_ix),
        R::list => self.seq_body(child, form_ix),
        R::vector => self.seq_body(child, form_ix),
        R::anonymous_fn => self.seq_body(child, form_ix),
        R::set => self.seq_body(child, form_ix),
        R::map => self.map(child, form_ix),
        R::reader_conditional => self.reader_conditional(child, form_ix),
        R::tagged_literal => self.tagged_literal(child, form_ix),

        _ => panic!("unexpected pair while parsing expression: {child:?}"),
      }
    }
  }

  fn char(&mut self, parent: Pair, form_ix: Ix) {
    let mut it = parent.clone().into_inner();

    let Some(child) = it.next() else {
      panic!("missing inner pair while parsing character: {parent:?}")
    };

    match child.as_rule() {
      R::char_name => self.push_token(
        parent,
        T::Char {
          syntax: CharSyntax::Name,
          value: match child.as_str() {
            "newline" => '\n',
            "space" => ' ',
            "tab" => '\t',
            "formfeed" => '\u{0C}',
            "backspace" => '\u{08}',
            _ => unreachable!("char name case analysis"),
          },
        },
        Some(form_ix),
      ),
      R::char_octal => self.push_token(
        parent,
        T::Char {
          syntax: CharSyntax::Octal,
          value: char::from_u32(
            u32::from_str_radix(child.as_str(), 8).unwrap(),
          )
          .unwrap(),
        },
        Some(form_ix),
      ),
      R::char_code_point => self.push_token(
        parent,
        T::Char {
          syntax: CharSyntax::CodePoint,
          value: char::from_u32(
            u32::from_str_radix(child.as_str(), 16).unwrap(),
          )
          .unwrap(),
        },
        Some(form_ix),
      ),

      _ => panic!("unexpected pair while parsing character: {child:?}"),
    }

    if let Some(extra) = it.next() {
      panic!("unexpected extra pair while parsing character: {extra:?}");
    }
  }

  fn number(&mut self, parent: Pair, form_ix: Ix) {
    let mut positive = true;
    for child in parent.clone().into_inner() {
      match child.as_rule() {
        R::sign => positive = child.as_str() == "+",
        R::unsigned_bigfloat => {
          self.unsigned_floats(parent.clone(), form_ix, true)
        }
        R::unsigned_float => {
          self.unsigned_floats(parent.clone(), form_ix, false)
        }
        R::unsigned_ratio => {
          self.unsigned_ratio(parent.clone(), child, form_ix, positive)
        }
        R::unsigned_radix_int => {
          self.unsigned_radix_int(parent.clone(), child, form_ix, positive)
        }
        R::unsigned_int => {
          self.unsigned_int(parent.clone(), child, form_ix, positive)
        }
        _ => self.push_token(
          parent.clone(),
          T::Residual {
            pair: format!("{:#?}", child).into_boxed_str(),
            ploc: ParserLoc::Number,
          },
          None,
        ),
      }
    }
  }

  fn unsigned_floats(&mut self, parent: Pair, form_ix: Ix, big: bool) {
    let l = parent.as_str();
    self.push_token(
      parent,
      if big {
        T::Numeric {
          class: NumberClass::BigDecimal,
          value: NumericValue::Float {
            value: l[..l.len() - 1].into(),
          },
        }
      } else {
        T::Numeric {
          class: NumberClass::Double,
          value: NumericValue::Float { value: l.into() },
        }
      },
      Some(form_ix),
    )
  }

  fn unsigned_ratio(
    &mut self,
    parent: Pair,
    pair: Pair,
    form_ix: Ix,
    positive: bool,
  ) {
    let mut it = pair.into_inner();

    let numerator = {
      let Some(p)= it.next() else {
        panic!("missing pair while parsing unsigned ratio: {parent:?}");
      };
      if p.as_rule() == R::numerator {
        p.as_str().into()
      } else {
        panic!("unexpected pair while parsing unsigned ratio: {parent:?}");
      }
    };

    let denominator = {
      let Some(p)= it.next() else {
        panic!("missing pair while parsing unsigned ratio: {parent:?}");
      };
      if p.as_rule() == R::denominator {
        p.as_str().into()
      } else {
        panic!("unexpected pair while parsing unsigned ratio: {parent:?}");
      }
    };

    if it.next().is_some() {
      panic!("unexpected extra pair while parsing unsigned ratio: {parent:?}");
    }

    self.push_token(
      parent,
      T::Numeric {
        class: NumberClass::Ratio,
        value: NumericValue::Fraction {
          positive,
          numerator,
          denominator,
        },
      },
      Some(form_ix),
    )
  }

  fn unsigned_radix_int(
    &mut self,
    parent: Pair,
    pair: Pair,
    form_ix: Ix,
    positive: bool,
  ) {
    let mut it = pair.into_inner();

    let radix: u32 = {
      let Some(p)= it.next() else {
        panic!("missing pair while parsing unsigned radix integer: {parent:?}");
      };
      if p.as_rule() == R::radix {
        p.as_str().parse().unwrap()
      } else {
        panic!(
          "unexpected pair while parsing unsigned radix integer: {parent:?}"
        );
      }
    };

    let value = {
      let Some(p)= it.next() else {
        panic!("missing pair while parsing unsigned radix integer: {parent:?}");
      };
      if p.as_rule() == R::radix_digits {
        p.as_str().into()
      } else {
        panic!(
          "unexpected pair while parsing unsigned radix integer: {parent:?}"
        );
      }
    };

    if it.next().is_some() {
      panic!("unexpected extra pair while parsing unsigned radix integer: {parent:?}");
    }

    self.push_token(
      parent,
      T::Numeric {
        class: NumberClass::Long,
        value: NumericValue::Int {
          positive,
          radix,
          value,
        },
      },
      Some(form_ix),
    )
  }

  fn unsigned_int(
    &mut self,
    parent: Pair,
    pair: Pair,
    form_ix: Ix,
    positive: bool,
  ) {
    let mut it = pair.into_inner();

    let value = {
      let Some(p)= it.next() else {
        panic!("missing pair while parsing unsigned integer: {parent:?}");
      };
      match p.as_rule() {
        R::oct_digits => NumericValue::Int {
          positive,
          radix: 8,
          value: p.as_str().into(),
        },
        R::hex_digits => NumericValue::Int {
          positive,
          radix: 16,
          value: p.as_str().into(),
        },
        R::unsigned_dec => NumericValue::Int {
          positive,
          radix: 10,
          value: p.as_str().into(),
        },
        _ => {
          panic!("unexpected pair while parsing unsigned integer: {parent:?}")
        }
      }
    };

    let class = match it.next() {
      Some(p) if p.as_rule() == R::bigint_suffix => NumberClass::BigInt,
      None => NumberClass::Long,
      _ => panic!("unexpected pair while parsing unsigned integer: {parent:?}"),
    };

    if it.next().is_some() {
      panic!(
        "unexpected extra pair while parsing unsigned integer: {parent:?}"
      );
    }

    self.push_token(parent, T::Numeric { class, value }, Some(form_ix));
  }

  fn string(&mut self, parent: Pair, form_ix: Ix) {
    use StringFragment as F;

    let mut fragments = Vec::new();

    for i in parent.clone().into_inner() {
      match i.as_rule() {
        R::unescaped => fragments.push(F::Unescaped {
          value: i.as_str().into(),
        }),
        R::esc_char => {
          let value = &i.as_str()[1..];
          let code = match value {
            "b" => 0x08,
            "t" => 0x09,
            "n" => 0x0A,
            "f" => 0x0C,
            "r" => 0x0D,
            "\"" => 0x22,
            "\\" => 0x5C,
            _ => panic!(
              "unexpected character escape while parsing string: {parent:?}"
            ),
          };
          fragments.push(F::Escaped { code })
        }
        R::esc_octet => {
          let value = &i.as_str()[1..];
          let code = u32::from_str_radix(value, 8).unwrap();
          fragments.push(F::Escaped { code })
        }
        R::esc_code_point => {
          let value = &i.as_str()[2..];
          let code = u32::from_str_radix(value, 16).unwrap();
          fragments.push(F::Escaped { code })
        }

        _ => panic!("unexpected pair while parsing string: {parent:?}"),
      }
    }

    fragments.shrink_to_fit();
    let literal = parent.as_str();

    self.push_token(
      parent,
      T::String {
        raw_value: literal[1..literal.len() - 1].into(),
        value: fragments.into_boxed_slice(),
      },
      Some(form_ix),
    );
  }

  fn regex(&mut self, pair: Pair, form_ix: Ix) {
    let raw_value = {
      let s = pair.as_str();
      s[2..s.len() - 1].into()
    };
    self.push_token(pair, T::Regex { raw_value }, Some(form_ix))
  }

  fn symbolic_value(&mut self, pair: Pair, form_ix: Ix) {
    use SymbolicValue as V;

    let mut it = pair.clone().into_inner().peekable();

    {
      let Some(p) = it.next() else {
        panic!("missing pair while parsing symbolic value: {pair:?}");
      };
      if p.as_rule() == R::symbolic_value_prefix {
        self.push_token(pair.clone(), T::SymbolicValuePrefix, Some(form_ix));
        // NB: We push the parent just so that the comments and whitespace
        // between the prefix and the actual symbol become children of the
        // symbolic value.
        self.push_parent(form_ix);
      } else {
        panic!("unexpected pair while parsing symbolic value: {pair:?}")
      }
    }

    self.comments_and_whitespace(&mut it);

    {
      let Some(p) = it.next() else {
        panic!("missing pair while parsing symbolic value: {pair:?}");
      };
      if p.as_rule() == R::unqualified_symbol {
        self.pop_parent();
        self.push_token(
          pair.clone(),
          T::SymbolicValue {
            value: match p.as_str() {
              "Inf" => V::PosInf,
              "-Inf" => V::NegInf,
              "NaN" => V::NaN,
              _ => V::Other(p.as_str().into()),
            },
          },
          Some(form_ix),
        )
      } else {
        panic!("unexpected pair while parsing symbolic value: {pair:?}")
      }
    }

    if it.next().is_some() {
      panic!("unexpected extra pair while parsing symbolic value: {pair:?}")
    }
  }

  fn symbol_or_tag(&mut self, pair: Pair, form_ix: Ix, is_symbol: bool) {
    let mut it = pair.clone().into_inner().peekable();

    let namespace = it
      .next_if(|p| p.as_rule() == R::namespace)
      .map(|p| p.as_str().into());

    let Some(p) = it.next() else {
      panic!("missing pair while parsing symbol or tar: {pair:?}");
    };

    let token = match p.as_rule() {
      R::qualified_symbol | R::unqualified_symbol => {
        if is_symbol {
          T::Symbol {
            namespace,
            name: p.as_str().into(),
          }
        } else {
          T::Tag {
            namespace,
            name: p.as_str().into(),
          }
        }
      }

      _ => panic!("unexpected pair while parsing symbol or tag: {pair:?}"),
    };

    if it.next().is_some() {
      panic!("unexptected extra pair while parsing symbol or tag: {pair:?}");
    }

    self.push_token(pair, token, Some(form_ix))
  }

  fn keyword(&mut self, pair: Pair, form_ix: Ix) {
    let mut it = pair.clone().into_inner().peekable();

    let alias = match it.next() {
      Some(p) if p.as_rule() == R::keyword_prefix => p.as_str() == "::",
      None => panic!("missing pair while parsing keyword: {pair:?}"),
      _ => panic!("unexpected pair while parsing keyword: {pair:?}"),
    };

    let namespace = it
      .next_if(|p| p.as_rule() == R::namespace)
      .map(|p| p.as_str().into());

    let Some(p) = it.next() else {
      panic!("missing pair while parsing keyword: {pair:?}");
    };

    let token = match p.as_rule() {
      R::qualified_symbol | R::unqualified_keyword => T::Keyword {
        alias,
        namespace,
        name: p.as_str().into(),
      },
      _ => panic!("unexpected pair while parsing keyword: {pair:?}"),
    };

    if it.next().is_some() {
      panic!("unexptected extra pair while parsing keyword: {pair:?}");
    }

    self.push_token(pair, token, Some(form_ix));
  }

  fn map(&mut self, pair: Pair, form_ix: Ix) {
    let mut it = pair.clone().into_inner().peekable();

    let body_form_ix =
      if let Some(p) = it.next_if(|p| p.as_rule() == R::map_qualifier) {
        let mut it2 = p.clone().into_inner();

        let alias = match it2.next() {
          Some(p2) if p2.as_rule() == R::map_qualifier_prefix => {
            p2.as_str() == "#::"
          }
          Some(_) => {
            panic!("unexpected pair while parsing map qualifier: {pair:?}")
          }
          None => panic!("missing pair while parsing map qualifier: {pair:?}"),
        };

        let namespace = match it2.next() {
          Some(p2) if p2.as_rule() == R::namespace => Some(p2.as_str().into()),
          Some(_) => {
            panic!("unexpected pair while parsing map qualifier: {pair:?}")
          }
          None => None,
        };

        if it2.next().is_some() {
          panic!("unexpected extra pair while parsing map qualifire: {pair:?}")
        }

        self.push_token(p, T::MapQualifier { alias, namespace }, Some(form_ix));

        self.push_parent(form_ix);

        self.next_form_ix()
      } else {
        form_ix
      };

    if form_ix != body_form_ix {
      self.comments_and_whitespace(&mut it);
    }

    match it.next() {
      Some(p) if p.as_rule() == R::unqualified_map => {
        self.seq_body(p, body_form_ix)
      }
      Some(_) => panic!("unexpected pair while parsing map: {pair:?}"),
      None => panic!("missing pair while parsing map: {pair:?}"),
    }

    if it.next().is_some() {
      panic!("unexpected extra pair while parsing map: {pair:?}");
    }

    if body_form_ix != form_ix {
      self.pop_parent();
    }
  }

  fn reader_conditional(&mut self, pair: Pair, form_ix: Ix) {
    let mut it = pair.clone().into_inner().peekable();

    match it.next() {
      Some(p) if p.as_rule() == R::reader_conditional_prefix => {
        let t = T::ReaderConditionalPrefix {
          splicing: p.as_str() == "#?@",
        };
        self.push_token(p, t, Some(form_ix));
      }
      Some(_) => {
        panic!("unexpected pair while parsing reader conditional: {pair:?}")
      }
      None => panic!("missing pair while parsing reader conditional: {pair:?}"),
    };

    self.push_parent(form_ix);

    self.comments_and_whitespace(&mut it);

    match it.next() {
      Some(p) if p.as_rule() == R::reader_conditional_body => {
        let ix = self.next_form_ix();
        self.seq_body(p, ix);
      }
      Some(_) => {
        panic!("unexpected pair while parsing reader conditional: {pair:?}")
      }
      None => panic!("missing pair while parsing reader conditional: {pair:?}"),
    }

    if it.next().is_some() {
      panic!(
        "unexpected extra pair while parsing reader conditional: {pair:?}"
      );
    }

    self.pop_parent();
  }

  fn seq_body(&mut self, pair: Pair, form_ix: Ix) {
    let mut it = pair.clone().into_inner().peekable();

    {
      let Some(p) = it.next() else {
        panic!("missing pair while parsing sequence body: {pair:?}");
      };
      let t = match p.as_rule() {
        R::list_start => T::StartList,
        R::vector_start => T::StartVector,
        R::anonymous_fn_start => T::StartAnonymousFn,
        R::set_start => T::StartSet,
        R::map_start => T::StartMap,
        _ => panic!("unexpected pair while parsing sequence body: {pair:?}"),
      };
      self.push_token(p, t, Some(form_ix));
      self.push_parent(form_ix);
    }

    loop {
      self.comments_and_whitespace(&mut it);

      if let Some(p) =
        it.next_if(|p| matches!(p.as_rule(), R::form | R::discarded_form))
      {
        match p.as_rule() {
          R::form => {
            let ix = self.next_form_ix();
            self.form(p, ix);
          }
          R::discarded_form => {
            let ix = self.next_form_ix();
            self.discarded_form(p, ix);
          }
          _ => unreachable!(),
        }
      } else {
        break;
      }
    }

    {
      let Some(p) = it.next() else {
        panic!("missing pair while parsing sequence body: {pair:?}");
      };
      let t = match p.as_rule() {
        R::list_end => T::EndList,
        R::vector_end => T::EndVector,
        R::anonymous_fn_end => T::EndAnonymousFn,
        R::set_end => T::EndSet,
        R::map_end => T::EndMap,
        _ => panic!("unexpected pair while parsing sequence body: {pair:?}"),
      };
      self.pop_parent();
      self.push_token(p, t, Some(form_ix));
    }

    if it.next().is_some() {
      panic!("unexpected extra pair while parsing sequence body: {pair:?}");
    }
  }

  fn tagged_literal(&mut self, pair: Pair, form_ix: Ix) {
    let tag_ix = self.next_form_ix();
    let arg_ix = self.next_form_ix();

    self.push_token(
      pair.clone(),
      T::TaggedLiteral { tag_ix, arg_ix },
      Some(form_ix),
    );
    self.push_parent(form_ix);

    let mut it = pair.clone().into_inner().peekable();

    self.comments_and_whitespace(&mut it);

    match it.next() {
      Some(p) if p.as_rule() == R::symbol => {
        self.symbol_or_tag(p, tag_ix, false)
      }
      Some(_) => {
        panic!("unexpected pair while parsing tagged literal: {pair:?}")
      }
      None => panic!("missing pair while parsing tagged literal: {pair:?}"),
    }

    self.comments_and_whitespace(&mut it);

    match it.next() {
      Some(p) if p.as_rule() == R::form => self.form(p, arg_ix),
      Some(_) => {
        panic!("unexpected pair while parsing tagged literal: {pair:?}")
      }
      None => panic!("missing pair while parsing tagged literal: {pair:?}"),
    }

    if it.next().is_some() {
      panic!("unexpected extra pair while parsing tagged literal: {pair:?}");
    }

    self.pop_parent();
  }
}
