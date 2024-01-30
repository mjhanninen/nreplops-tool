// pprint/unformatted_edn.rs
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

//! Converts the result IR into a layout problem that should produce in a
//! relatively pretty EDN output once solved.

use crate::clojure::lex::Lexeme;

use super::{
  printer::{BuildInput, Command},
  style::Style,
};

use Lexeme as L;
use Style as S;

pub fn generate_printer_input<'a, I>(
  lexemes: I,
  printer_input: &mut Vec<Command<'a>>,
) where
  I: Iterator<Item = &'a Lexeme<'a>>,
{
  for l in lexemes {
    match *l {
      L::Whitespace { source } => {
        printer_input.add_styled(S::Whitespace, source)
      }
      L::Nil { source, .. } => printer_input.add_styled(S::NilValue, source),
      L::Boolean { source, .. } => {
        printer_input.add_styled(S::BooleanValue, source)
      }
      L::Numeric { source, .. } => {
        printer_input.add_styled(S::NumberValue, source)
      }
      L::String { source, .. } => {
        printer_input.add_styled(S::StringDecoration, "\"");
        printer_input.add_styled(S::StringValue, &source[1..source.len() - 1]);
        printer_input.add_styled(S::StringDecoration, "\"");
      }
      L::SymbolicValuePrefix { source, .. } => {
        printer_input.add_styled(S::SymbolicValueDecoration, source)
      }
      L::SymbolicValue { source, .. } => {
        printer_input.add_styled(S::SymbolicValue, source)
      }
      L::Symbol {
        namespace, name, ..
      } => {
        if let Some(s) = namespace {
          printer_input.add_styled(S::SymbolNamespace, s);
          printer_input.add_styled(S::SymbolDecoration, "/");
        }
        printer_input.add_styled(S::SymbolName, name);
      }
      L::Keyword {
        alias,
        namespace,
        name,
        ..
      } => {
        printer_input
          .add_styled(S::KeywordDecoration, if alias { "::" } else { ":" });
        if let Some(s) = namespace {
          printer_input.add_styled(S::KeywordNamespace, s);
          printer_input.add_styled(S::KeywordDecoration, "/");
        }
        printer_input.add_styled(S::KeywordName, name);
      }
      L::TaggedLiteral { source, .. } => {
        printer_input.add_styled(S::TaggedLiteralDecoration, source);
      }
      L::Tag {
        namespace, name, ..
      } => {
        if let Some(s) = namespace {
          printer_input.add_styled(S::TaggedLiteralNamespace, s);
          printer_input.add_styled(S::TaggedLiteralDecoration, "/");
        }
        printer_input.add_styled(S::TaggedLiteralName, name);
      }
      L::StartList { source, .. }
      | L::EndList { source, .. }
      | L::StartVector { source, .. }
      | L::EndVector { source, .. }
      | L::StartSet { source, .. }
      | L::EndSet { source, .. }
      | L::StartMap { source, .. }
      | L::EndMap { source, .. } => {
        printer_input.add_styled(S::CollectionDelimiter, source)
      }
      ref unhandled => todo!("no rule for: {:?}", unhandled),
    }
  }
}
