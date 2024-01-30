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

use std::io::{self, Write};

mod fragments;
mod layout_solver;
mod printer;
mod style;

use crate::clojure::{
  lex::Lexeme,
  result_ir::{self, MapEntry, Value},
};
use layout_solver::{ChunkBuilder, Chunks};

#[derive(Debug)]
pub struct ClojureResultPrinter {
  pub pretty: bool,
  pub color: bool,
  pub width: u16,
}

impl ClojureResultPrinter {
  pub fn new(pretty: bool, color: bool, width: u16) -> Self {
    Self {
      pretty,
      color,
      width,
    }
  }

  pub fn print(
    &self,
    writer: &mut impl Write,
    lexemes: &[Lexeme],
  ) -> io::Result<()> {
    let mut printer_input = Vec::new();
    if self.pretty {
      let value = result_ir::build(lexemes).unwrap();
      let chunks = clojure_chunks(&value);
      layout_solver::solve(&chunks, &mut printer_input);
    } else {
      unformatted_layout(lexemes.iter(), &mut printer_input);
    }
    printer::print(writer, printer_input.iter(), self.color)?;
    writeln!(writer)
  }
}

fn unformatted_layout<'a, I>(
  lexemes: I,
  printer_input: &mut Vec<printer::Command<'a>>,
) where
  I: Iterator<Item = &'a Lexeme<'a>>,
{
  use printer::BuildInput;
  use style::Style as S;
  use Lexeme as L;

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

/// The width of a rigid value, i.e. a value that in no circumstance can be
/// split or combined to occupy a narrower or wider horizontal span.
//
// XXX(soija) In fact, we could call this a natural width and some values
//            (lists, maps, and the like) just don't have it.
fn rigid_width(value: &Value) -> Option<usize> {
  use Value as V;
  match value {
    V::Nil => Some(3),
    V::Number { literal } => Some(literal.len()),
    V::String { literal } => Some(literal.len()),
    V::Boolean { value } => Some(if *value { 4 } else { 5 }),
    V::SymbolicValue { literal } => Some(literal.len() + 2),
    // XXX(soija) This could be done by taking the rigid width of contained
    //            value and, if any, add the prerix width.  However this means
    //            that we could not use the breakpoint before the contained
    //            value that the literal offers.
    V::TaggedLiteral { .. } => None,
    V::Keyword {
      namespace,
      name,
      alias,
    } => Some(
      if *alias { 2 } else { 1 }
        + namespace.map(|s| s.len() + 1).unwrap_or(0)
        + name.len(),
    ),
    V::Symbol { namespace, name } => Some(if let Some(n) = namespace {
      n.len() + 1 + name.len()
    } else {
      name.len()
    }),
    _ => None,
  }
}

fn clojure_chunks<'a>(value: &Value<'a>) -> Chunks<'a> {
  let mut builder = ChunkBuilder::new();
  chunks_from_value(&mut builder, value);
  builder.build()
}

fn chunks_from_value<'a>(builder: &mut ChunkBuilder<'a>, value: &Value<'a>) {
  use layout_solver::*;
  use style::Style as S;
  use Value as V;

  match value {
    V::Nil => {
      builder.add_text(TextBuilder::new().add("nil", S::NilValue));
    }
    V::Boolean { value } => {
      builder.add_text(
        TextBuilder::new()
          .add(if *value { "true" } else { "false" }, S::BooleanValue),
      );
    }
    V::Number { literal } => {
      builder.add_text(TextBuilder::new().add(*literal, S::NumberValue));
    }
    V::String { literal } => {
      builder.add_text(TextBuilder::new().add("\"", S::StringDecoration));
      builder.add_text(
        TextBuilder::new()
          // XXX(soija) This subrange is a hack. FIXME: Make the string value
          //            (and the corresponding string lexeme) to expose the
          //            string *content* directly.
          .add(&literal[1..literal.len() - 1], S::StringValue),
      );
      builder.add_text(TextBuilder::new().add("\"", S::StringDecoration));
    }
    V::SymbolicValue { literal } => {
      builder
        .add_text(TextBuilder::new().add("##", S::SymbolicValueDecoration));
      builder.add_text(TextBuilder::new().add(*literal, S::SymbolicValue));
    }
    V::Symbol { namespace, name } => {
      builder.add_text(
        TextBuilder::new()
          .apply(|b| {
            if let Some(n) = namespace {
              b.add(*n, S::SymbolNamespace).add("/", S::SymbolDecoration)
            } else {
              b
            }
          })
          .add(*name, S::SymbolName),
      );
    }
    V::Keyword {
      namespace,
      name,
      alias,
    } => builder.add_text(
      TextBuilder::new()
        .add(if *alias { "::" } else { ":" }, S::KeywordDecoration)
        .apply(|b| {
          if let Some(n) = namespace {
            b.add(*n, S::KeywordNamespace)
              .add("/", S::KeywordDecoration)
          } else {
            b
          }
        })
        .add(*name, S::KeywordName),
    ),
    V::TaggedLiteral {
      namespace,
      name,
      value,
    } => {
      builder.add_text(
        TextBuilder::new()
          .add("#", S::TaggedLiteralDecoration)
          .apply(|b| {
            if let Some(s) = namespace {
              b.add(*s, S::TaggedLiteralNamespace)
                .add("/", S::TaggedLiteralDecoration)
            } else {
              b
            }
          })
          .add(*name, S::TaggedLiteralName),
      );
      builder.add_soft_space();
      chunks_from_value(builder, value);
    }
    V::List { values } => {
      chunks_from_value_seq(builder, values, true, "(", ")")
    }
    V::Vector { values } => {
      chunks_from_value_seq(builder, values, false, "[", "]")
    }
    V::Set { values } => {
      chunks_from_value_seq(builder, values, false, "#{", "}")
    }
    V::Map { entries } => chunks_from_map(builder, entries),
  }
}

fn chunks_from_value_seq<'a>(
  builder: &mut ChunkBuilder<'a>,
  values: &[Value<'a>],
  anchor_after_first: bool,
  opening_delim: &'static str,
  closing_delim: &'static str,
) {
  use layout_solver::*;
  use style::Style as S;

  let only_simple_values = values.iter().all(|v| {
    !matches!(
      v,
      Value::List { .. }
        | Value::Vector { .. }
        | Value::Set { .. }
        | Value::Map { .. }
    )
  });

  builder
    .add_text(TextBuilder::new().add(opening_delim, S::CollectionDelimiter));

  let mut it = values.iter();

  if anchor_after_first {
    if let Some(first) = it.next() {
      chunks_from_value(builder, first);
      builder.add_soft_space();
    }
  }

  if let Some(first) = it.next() {
    let anchor = builder.set_anchor();
    chunks_from_value(builder, first);

    for value in it {
      if only_simple_values {
        builder.add_soft_space();
      } else {
        builder.break_hard(anchor);
      }
      chunks_from_value(builder, value);
    }
  }

  builder
    .add_text(TextBuilder::new().add(closing_delim, S::CollectionDelimiter));
}

fn chunks_from_map<'a>(
  builder: &mut ChunkBuilder<'a>,
  entries: &[MapEntry<'a>],
) {
  use layout_solver::*;
  use style::Style as S;

  builder.add_text(TextBuilder::new().add("{", S::CollectionDelimiter));

  let key_width = entries
    .iter()
    .fold(Some(0), |acc, e| Some(acc?.max(rigid_width(&e.key)?)));

  let mut it = entries.iter();
  if let Some(first) = it.next() {
    let anchor = builder.set_anchor();

    if let Some(key_width) = key_width {
      let value_anchor = builder.set_relative_anchor(
        anchor,
        1 + i16::try_from(key_width).expect("too large keys"),
      );

      chunks_from_value(builder, &first.key);

      builder.jump_to(value_anchor);
      chunks_from_value(builder, &first.value);

      for entry in it {
        builder.break_hard(anchor);
        chunks_from_value(builder, &entry.key);

        builder.jump_to(value_anchor);
        chunks_from_value(builder, &entry.value);
      }
    } else {
      chunks_from_value(builder, &first.key);

      builder.add_soft_space();
      chunks_from_value(builder, &first.value);

      for entry in it {
        builder.break_hard(anchor);
        chunks_from_value(builder, &entry.key);

        builder.add_soft_space();
        chunks_from_value(builder, &entry.value);
      }
    }
  }

  builder.add_text(TextBuilder::new().add("}", S::CollectionDelimiter));
}
