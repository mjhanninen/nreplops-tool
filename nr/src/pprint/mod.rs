// pprint/mod.rs
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

//
// So the different representations that the result being printed passes through
// are the following:
//
// 1. unparsed result
// 2. list of lexemes
// 3. result value AST
// 4. input structure to layout solver
// 5. input structure to printer
//
// The unparsed result (1) is what we receive from the nREPL server.  This
// is produced by Clojure's printer so it is Clojure but, importantly, very
// restricted form of Clojure.  It does not contain meta data literal and most
// of the reader macros are absent as well.
//
// The lexemes (2) are produced with our Clojure lexer and should be able to
// represent the whole language.  However, as the input result is restricted
// form of Clojure, so the produced lexemes are also a subset of known lexemes.
//
// The abstract syntax tree (3) is capable of representing only a limited subset
// of Clojure; this is effectively the EDN.  It is rich enough so that we can
// formulate the problem for the layout solver and, optionally, translate the
// data into some other form (e.g. JSON, YAML, or CSV).
//
// The input to the pretty-printing layout solver (4) is either a flat list
// or a tree structure.  I'm not sure which one it will be.  In any case it
// conssists of chunks of unbreakable texts together with styling information,
// suggested breakpoints, layout anchors, optional whitespacing, relationships
// between breakpoints and the like.  Things that the layout solver uses while
// determining the optimal layout.
//
// The input to the printer (5) consists of text fragments, style coding
// (separated from the text), line breaks, and spacing.  There is not much
// conditionality at this phase; only the decision whether to include or
// ignore the styling (coloring) when printing according to the alraedy fixed
// layout.
//
// When we are printing unformatted but colored output we can skip the phases
// (3) and (4) produce the printer input (5) directly from the lexemes (2).
//
// When we are converting the results to some other output format (e.g. JSON,
// YAML, or CSV) we follow through the same phases but do the translation
// into the alternative output format when we produce the input for the layout
// solver.
//

#![allow(unused)]

use std::io::{self, Write};

mod fragments;
mod layout_solver;
mod printer;
mod style;

use crate::clojure::{
  lex::Lexeme,
  result_ir::{self, KeywordNamespace, MapEntry, Value},
};
use layout_solver::Chunk;

#[derive(Debug)]
pub struct ClojureResultPrinter {
  pub pretty: bool,
  pub color: bool,
}

impl ClojureResultPrinter {
  pub fn new(pretty: bool, color: bool) -> Self {
    Self { pretty, color }
  }

  pub fn print(
    &self,
    writer: &mut impl Write,
    lexemes: &[Lexeme],
  ) -> io::Result<()> {
    let value = result_ir::build(lexemes).unwrap();
    let chunks = clojure_chunks(&value);
    let mut printer_input = Vec::new();
    layout_solver::solve(&chunks, &mut printer_input);
    printer::print(writer, printer_input.iter(), self.color)?;
    writeln!(writer)
  }
}

fn clojure_chunks<'a>(value: &Value<'a>) -> Box<[Chunk<'a>]> {
  let mut chunks = Vec::new();
  chunks_from_value(&mut chunks, value);
  chunks.shrink_to_fit();
  chunks.into_boxed_slice()
}

fn chunks_from_value<'a>(chunks: &mut Vec<Chunk<'a>>, value: &Value<'a>) {
  use fragments::*;
  use layout_solver::*;
  use style::Style as S;
  use Value as V;

  match value {
    V::Nil => {
      chunks.push(TextBuilder::new().add("nil", S::NilValue).build());
    }
    V::Number { literal } => {
      chunks.push(TextBuilder::new().add(*literal, S::NumberValue).build());
    }
    V::String { literal } => {
      chunks.push(TextBuilder::new().add("\"", S::StringDecoration).build());
      chunks.push(
        TextBuilder::new()
          // XXX(soija) This subrange is a hack. FIXME: Make the string value
          //            (and the corresponding string lexeme) to expose the
          //            string *content* directly.
          .add(&literal[1..literal.len() - 1], S::StringValue)
          .build(),
      );
      chunks.push(TextBuilder::new().add("\"", S::StringDecoration).build());
    }
    V::Boolean { value } => {
      chunks.push(
        TextBuilder::new()
          .add(if *value { "true" } else { "false" }, S::BooleanValue)
          .build(),
      );
    }
    V::Symbol { namespace, name } => {
      chunks.push(
        TextBuilder::new()
          .apply(|b| {
            if let Some(n) = namespace {
              b.add(*n, S::SymbolNamespace).add("/", S::SymbolDecoration)
            } else {
              b
            }
          })
          .add(*name, S::SymbolName)
          .build(),
      );
    }
    V::Keyword { namespace, name } => chunks.push(
      TextBuilder::new()
        .apply(|b| {
          use KeywordNamespace as K;
          match namespace {
            K::None => b.add(":", S::KeywordDecoration),
            K::Alias(a) => b
              .add("::", S::KeywordDecoration)
              .add(*a, S::KeywordNamespace)
              .add("/", S::KeywordDecoration),

            K::Namespace(n) => b
              .add(":", S::KeywordDecoration)
              .add(*n, S::KeywordNamespace)
              .add("/", S::KeywordDecoration),
          }
        })
        .add(*name, S::KeywordName)
        .build(),
    ),
    V::List { values } => chunks_from_value_seq(chunks, values, true, "(", ")"),
    V::Vector { values } => {
      chunks_from_value_seq(chunks, values, false, "[", "]")
    }
    V::Set { values } => {
      chunks_from_value_seq(chunks, values, false, "#{", "}")
    }
    V::Map { entries } => chunks_from_map(chunks, entries),
  }
}

fn chunks_from_value_seq<'a>(
  chunks: &mut Vec<Chunk<'a>>,
  values: &[Value<'a>],
  anchor_after_first: bool,
  opening_delim: &'static str,
  closing_delim: &'static str,
) {
  use fragments::*;
  use layout_solver::*;
  use style::Style as S;

  chunks.push(
    TextBuilder::new()
      .add(opening_delim, S::CollectionDelimiter)
      .build(),
  );

  let mut it = values.iter();

  if anchor_after_first {
    if let Some(first) = it.next() {
      chunks_from_value(chunks, first);
      chunks.push(Chunk::SoftSpace);
    }
  }

  if let Some(first) = it.next() {
    chunks.push(Chunk::PushAnchor);
    chunks_from_value(chunks, first);
    for value in it {
      chunks.push(Chunk::HardBreak);
      chunks_from_value(chunks, value);
    }
    chunks.push(Chunk::PopAnchor);
  }

  chunks.push(
    TextBuilder::new()
      .add(closing_delim, S::CollectionDelimiter)
      .build(),
  );
}

fn chunks_from_map<'a>(chunks: &mut Vec<Chunk<'a>>, entries: &[MapEntry<'a>]) {
  use fragments::*;
  use layout_solver::*;
  use style::Style as S;

  chunks.push(TextBuilder::new().add("{", S::CollectionDelimiter).build());

  let mut it = entries.iter();
  if let Some(first) = it.next() {
    chunks.push(Chunk::PushAnchor);
    chunks_from_value(chunks, &first.key);
    chunks.push(Chunk::SoftSpace);
    chunks_from_value(chunks, &first.value);
    for entry in it {
      chunks.push(Chunk::HardBreak);
      chunks_from_value(chunks, &entry.key);
      chunks.push(Chunk::SoftSpace);
      chunks_from_value(chunks, &entry.value);
    }
    chunks.push(Chunk::PopAnchor);
  }

  chunks.push(TextBuilder::new().add("}", S::CollectionDelimiter).build());
}
