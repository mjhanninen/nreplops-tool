// clojure/token.rs
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

#![allow(unused)]

use std::convert::From;
use std::iter::{Filter, Peekable};
use std::rc::Rc;

use crate::error::Error;

use super::lex::{
  CharSyntax, Ix, Lexeme, NumberClass, NumericValue, Source, StringFragment,
  SymbolicValue, Token,
};

use Token as T;

#[derive(Clone, Debug)]
pub struct TokenTree {
  pub value: TokenTreeValue,
  pub source: Option<Source>,
}

#[derive(Clone, Debug)]
pub enum TokenTreeValue {
  // Simple values
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
    raw_value: Rc<str>,
    value: Rc<[StringFragment]>,
  },
  Regex {
    raw_value: Rc<str>,
  },
  SymbolicValue {
    value: SymbolicValue,
  },
  Symbol {
    namespace: Option<Rc<str>>,
    name: Rc<str>,
  },
  Keyword {
    alias: bool,
    namespace: Option<Rc<str>>,
    name: Rc<str>,
  },

  // Compound values
  Meta {
    /// The index to the subform that gives the meta-form
    meta_ix: usize,
    /// The index to the subform that gives the actual form
    form_ix: usize,
    /// Subforms two of which are actually meaningful
    forms: Box<TokenTree>,
  },
  Discard {
    form: Box<TokenTree>,
  },
  Quote {
    form: Box<TokenTree>,
  },
  VarQuote {
    form: Box<TokenTree>,
  },
  Synquote {
    form: Box<TokenTree>,
  },
  Unquote {
    splicing: bool,
    form: Box<TokenTree>,
  },
  TaggedLiteral {
    namespace: Option<Rc<str>>,
    name: Rc<str>,
    form: Box<TokenTree>,
  },
  AnonymousFn {
    forms: Box<[TokenTree]>,
  },
  List {
    forms: Box<[TokenTree]>,
  },
  Vector {
    forms: Box<[TokenTree]>,
  },
  Set {
    forms: Box<[TokenTree]>,
  },
  Map {
    qualifier: MapQualifier,
    forms: Box<[TokenTree]>,
  },
  ReaderConditional {
    splicing: bool,
    forms: Box<[TokenTree]>,
  },
}

#[derive(Clone, Debug)]
pub enum MapQualifier {
  Unqualified,
  Alias(Rc<str>),
  Namespace(Rc<str>),
}

// Pre-filter hack
//
// XXX(soija) It is so unsatisfactory that Rust's type system's deficiencies
//            force us to use either this or `Box<dyn Iterator<Item = Lexeme>>`.

#[inline(always)]
fn keep_lexeme(lexeme: &Lexeme) -> bool {
  !matches!(lexeme.token, Token::Whitespace | Token::Comment)
}

pub struct Prefilter<I>
where
  I: Iterator<Item = Lexeme>,
{
  iter: I,
}

impl<I> Iterator for Prefilter<I>
where
  I: Iterator<Item = Lexeme>,
{
  type Item = Lexeme;

  fn next(&mut self) -> Option<Self::Item> {
    while let Some(lexeme) = self.iter.next() {
      if keep_lexeme(&lexeme) {
        return Some(lexeme);
      }
    }
    None
  }
}

//
// Converting lexeme iterator into token tree iterator
//

pub struct TokenTrees<I>
where
  I: Iterator<Item = Lexeme>,
{
  iter: Peekable<Prefilter<I>>,
}

impl<I> From<I> for TokenTrees<I>
where
  I: Iterator<Item = Lexeme>,
{
  fn from(value: I) -> TokenTrees<I> {
    TokenTrees {
      iter: Prefilter { iter: value }.peekable(),
    }
  }
}

impl<I> Iterator for TokenTrees<I>
where
  I: Iterator<Item = Lexeme>,
{
  type Item = Result<TokenTree, Error>;

  fn next(&mut self) -> Option<Self::Item> {
    while self.iter.peek().is_some() {
      match TokenTreeBuilder::new(&mut self.iter).try_build() {
        Ok(Some(tt)) => return Some(Ok(tt)),
        Err(e) => return Some(Err(e)),
        _ => (),
      }
    }
    None
  }
}

struct TokenTreeBuilder<'a, I>
where
  I: Iterator<Item = Lexeme>,
{
  lexemes: &'a mut Peekable<Prefilter<I>>,
  stack: Vec<TokenTree>,
}

impl<'a, I> TokenTreeBuilder<'a, I>
where
  I: Iterator<Item = Lexeme>,
{
  fn new(lexemes: &'a mut Peekable<Prefilter<I>>) -> Self {
    Self {
      lexemes,
      stack: Vec::new(),
    }
  }

  fn try_build(mut self) -> Result<Option<TokenTree>, Error> {
    debug_assert!(self.lexemes.peek().is_some());
    self.collect_form(0);
    debug_assert!(self.stack.len() <= 1);
    Ok(self.stack.pop())
  }

  #[inline(always)]
  fn peek(&mut self) -> Option<&Lexeme> {
    self.lexemes.peek()
  }

  #[inline(always)]
  fn current(&mut self) -> &Lexeme {
    self.lexemes.peek().unwrap()
  }

  #[inline(always)]
  fn pop(&mut self) -> Lexeme {
    debug_assert!(self.lexemes.peek().is_some());
    unsafe { self.lexemes.next().unwrap_unchecked() }
  }

  fn collect_form(&mut self, parent_ix: Ix) {
    debug_assert!(self.lexemes.peek().is_some());

    use Action as A;

    #[derive(Debug)]
    enum Action {
      Discard,
      CollectJustThis,
      CollectCountedChildren(usize),
      CollectDelimitedChildren,
    }

    let action = {
      let lexeme = dbg!(self.current());

      debug_assert_eq!(lexeme.parent_ix, parent_ix);

      match dbg!(&lexeme.token) {
        T::Discard => A::Discard,

        T::StartList
        | T::StartVector
        | T::StartSet
        | T::StartMap
        | T::StartAnonymousFn
        | T::StartReaderConditional => A::CollectDelimitedChildren,

        (T::Quote
        | T::VarQuote
        | T::Synquote
        | T::Unquote
        | T::SplicingUnquote) => A::CollectCountedChildren(1),

        T::Meta { .. } | T::TaggedLiteral { .. } => {
          A::CollectCountedChildren(2)
        }

        T::Numeric { .. }
        | T::Char { .. }
        | T::String { .. }
        | T::Symbol { .. }
        | T::Keyword { .. }
        | T::Tag { .. } => A::CollectJustThis,

        (T::EndList
        | T::EndVector
        | T::EndSet
        | T::EndMap
        | T::EndAnonymousFn
        | T::EndReaderConditional) => {
          panic!("unexpected end lexeme while collecting next form: {lexeme:?}")
        }
        _ => panic!("unexpected lexeme while collecting next form: {lexeme:?}"),
      }
    };

    match dbg!(action) {
      A::Discard => discard_recursively(lexemes),
      A::CollectJustThis => collector.collect_lexeme(lexemes.next().unwrap()),
      A::CollectCountedChildren(n) => {
        let lexeme = lexemes.next().unwrap();
        let form_ix = lexeme.form_ix;
        collector.collect_lexeme(lexeme);
        for _ in 0..n {
          collect_form_recursively(form_ix, lexemes, collector)?;
        }
      }
      A::CollectDelimitedChildren => {
        let start_lexeme = lexemes.next().unwrap();
        let form_ix = start_lexeme.form_ix;
        collector.collect_lexeme(start_lexeme);
        loop {
          let next = lexemes.peek().unwrap();
          match next.token {
            (T::EndList
            | T::EndVector
            | T::EndSet
            | T::EndMap
            | T::EndAnonymousFn
            | T::EndReaderConditional)
              if next.form_ix == form_ix =>
            {
              collector.collect_lexeme(lexemes.next().unwrap());
              break;
            }
            _ => collect_form_recursively(form_ix, lexemes, collector)?,
          }
        }
      }
    }

    Ok(())
  }
}

// #[derive(Default)]
// struct FragmentCollector {
//   unfinished: Vec<Lexeme>,
//   fragments: Vec<Fragment>,
// }

// impl FragmentCollector {
//   fn new() -> Self {
//     Self::default()
//   }

//   fn collect_lexeme(&mut self, lexeme: Lexeme) {
//     self.unfinished.push(lexeme);
//   }

//   fn is_empty(&self) -> bool {
//     self.fragments.is_empty() && self.unfinished.is_empty()
//   }

//   fn build(mut self) -> Box<[Fragment]> {
//     if !self.unfinished.is_empty() {
//       self
//         .fragments
//         .push(Fragment::Lexemes(self.unfinished.into_boxed_slice()));
//     }
//     self.fragments.into()
//   }
// }

// fn discard_recursively<I>(lexemes: &mut Peekable<I>)
// where
//   I: Iterator<Item = Lexeme>,
// {
//   let lexeme = lexemes.next().unwrap();

//   #[allow(clippy::enum_variant_names)]
//   enum Discard {
//     JustThis,
//     CountedChildren(usize),
//     DelimitedChildren,
//   }

//   let action = match lexeme.token {
//     (T::Nil
//     | T::Boolean { .. }
//     | T::Numeric { .. }
//     | T::Char { .. }
//     | T::String { .. }
//     | T::Regex { .. }
//     | T::SymbolicValue { .. }
//     | T::Symbol { .. }
//     | T::Keyword { .. }
//     | T::Tag { .. }) => Discard::JustThis,

//     (T::Discard
//     | T::Quote
//     | T::VarQuote
//     | T::Synquote
//     | T::Unquote
//     | T::SplicingUnquote) => Discard::CountedChildren(1),

//     T::Meta { .. } => Discard::CountedChildren(2),

//     (T::StartList
//     | T::StartVector
//     | T::StartSet
//     | T::StartMap
//     | T::StartAnonymousFn
//     | T::StartReaderConditional) => Discard::DelimitedChildren,

//     _ => panic!("unexpected lexeme while discarding: lexeme = {lexeme:?}",),
//   };

//   match action {
//     Discard::JustThis => {}

//     Discard::CountedChildren(n) => {
//       for _ in 0..n {
//         let child = lexemes.peek().unwrap();
//         if child.parent_ix == lexeme.form_ix {
//           discard_recursively(lexemes);
//         } else {
//           panic!(
//             "unexpected lexeme while discarding child: parent = {lexeme:?}, child {child:?}"
//           );
//         }
//       }
//     }

//     Discard::DelimitedChildren => loop {
//       let child_or_end = lexemes.peek().unwrap();
//       match child_or_end.token {
//           (T::EndList
//           | T::EndVector
//           | T::EndSet
//           | T::EndMap
//           | T::EndAnonymousFn
//           | T::EndReaderConditional)
//             if child_or_end.form_ix == lexeme.form_ix => {
//             lexemes.next().unwrap();
//             break;
//           },
//           _ if child_or_end.parent_ix == lexeme.form_ix => discard_recursively( lexemes),
//           _ => panic!("unexpected lexeme while discarding delimited children: parent = {lexeme:?}, child = {child_or_end:?}"),
//         }
//     },
//   }
// }
