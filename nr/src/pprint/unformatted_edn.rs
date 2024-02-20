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

use crate::clojure::lex::{Lexeme, Token};

use super::{
  printer::{BuildInput, Command},
  style::Style,
};

use Style as S;
use Token as T;

pub fn generate_printer_input<'a, I>(
  lexemes: I,
  printer_input: &mut Vec<Command>,
) where
  I: Iterator<Item = &'a Lexeme>,
{
  let mut last_var_quote: Option<u32> = None;

  for l in lexemes {
    match &l.token {
      T::Whitespace => {
        // XXX(soija) FIXME: Try first the original source and, if not present,
        //            use just single blank space.
        printer_input
          .add_styled(S::Whitespace, l.source.as_ref().unwrap().str.clone())
      }
      T::Nil => printer_input.add_styled(S::NilValue, " "),
      T::Boolean { value } => printer_input
        .add_styled(S::BooleanValue, if *value { "true" } else { "false" }),
      T::Numeric { .. } => {
        // XXX(soija) FIXME: Try first the original source and, if not present,
        //            use token to derive representation.
        printer_input
          .add_styled(S::NumberValue, l.source.as_ref().unwrap().str.clone())
      }
      T::String { raw_value, .. } => {
        printer_input.add_styled(S::StringDecoration, "\"");
        printer_input.add_styled(S::StringValue, raw_value.clone());
        printer_input.add_styled(S::StringDecoration, "\"");
      }
      T::SymbolicValuePrefix => {
        printer_input.add_styled(S::SymbolicValueDecoration, "##")
      }
      T::SymbolicValue { .. } => {
        // XXX(soija) FIXME: Try first the original source and, if not present,
        //            use token to derive representation.
        printer_input
          .add_styled(S::SymbolicValue, l.source.as_ref().unwrap().str.clone())
      }
      T::Symbol { namespace, name } => {
        let is_var_quoted =
          last_var_quote.map(|ix| ix == l.parent_ix).unwrap_or(false);
        if is_var_quoted {
          if let Some(ns) = namespace {
            printer_input.add_styled(S::VarQuoteNamespace, ns.clone());
            printer_input.add_styled(S::VarQuoteDecoration, "/");
          }
          printer_input.add_styled(S::VarQuoteName, name.clone());
        } else {
          if let Some(ns) = namespace {
            printer_input.add_styled(S::SymbolNamespace, ns.clone());
            printer_input.add_styled(S::SymbolDecoration, "/");
          }
          printer_input.add_styled(S::SymbolName, name.clone());
        }
      }
      T::Keyword {
        alias,
        namespace,
        name,
      } => {
        printer_input
          .add_styled(S::KeywordDecoration, if *alias { "::" } else { ":" });
        if let Some(ns) = namespace {
          printer_input.add_styled(S::KeywordNamespace, ns.clone());
          printer_input.add_styled(S::KeywordDecoration, "/");
        }
        printer_input.add_styled(S::KeywordName, name.clone());
      }
      T::VarQuote => {
        printer_input.add_styled(S::VarQuoteDecoration, "#'");
        last_var_quote = Some(l.form_ix);
      }
      T::TaggedLiteral { .. } => {
        printer_input.add_styled(S::TaggedLiteralDecoration, "#");
      }
      T::Tag { namespace, name } => {
        if let Some(ns) = namespace {
          printer_input.add_styled(S::TaggedLiteralNamespace, ns.clone());
          printer_input.add_styled(S::TaggedLiteralDecoration, "/");
        }
        printer_input.add_styled(S::TaggedLiteralName, name.clone());
      }

      T::StartList => printer_input.add_styled(S::CollectionDelimiter, "("),
      T::EndList => printer_input.add_styled(S::CollectionDelimiter, ")"),
      T::StartVector => printer_input.add_styled(S::CollectionDelimiter, "["),
      T::EndVector => printer_input.add_styled(S::CollectionDelimiter, "]"),
      T::StartSet => printer_input.add_styled(S::CollectionDelimiter, "#{"),
      T::EndSet => printer_input.add_styled(S::CollectionDelimiter, "}"),
      T::StartMap => printer_input.add_styled(S::CollectionDelimiter, "{"),
      T::EndMap => printer_input.add_styled(S::CollectionDelimiter, "}"),

      _ => todo!("no rule for: {l:?}"),
    }
  }
}
