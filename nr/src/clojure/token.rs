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

use std::rc::Rc;

use super::lex::{
  CharSyntax, NumberClass, NumericValue, StringFragment, SymbolicValue,
};

#[derive(Clone, Debug)]
pub enum TokenTree {
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
    meta: Box<TokenTree>,
    form: Box<TokenTree>,
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
