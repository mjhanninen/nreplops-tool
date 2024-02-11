// pprint/pretty_edn.rs
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

use crate::clojure::result_ir::{MapEntry, Value};

use super::{
  layout_solver::{Program, ProgramBuilder, TextBuilder},
  style::Style,
};

use Style as S;
use TextBuilder as TB;
use Value as V;

/// Convert the given result value into a program to be passed on to the layout
/// solver.
pub fn convert_to_layout_program(value: &Value) -> Program {
  let mut builder = ProgramBuilder::new();
  program_value(&mut builder, value);
  builder.build()
}

fn program_value(builder: &mut ProgramBuilder, value: &Value) {
  match value {
    V::Nil => {
      builder.add_text(TB::new().add("nil", S::NilValue));
    }
    V::Boolean { value } => {
      builder.add_text(
        TB::new().add(if *value { "true" } else { "false" }, S::BooleanValue),
      );
    }
    V::Number { literal } => {
      builder.add_text(TB::new().add(literal.clone(), S::NumberValue));
    }
    V::String { literal } => {
      builder.add_text(TB::new().add("\"", S::StringDecoration));
      builder.add_text(
        TB::new()
          // XXX(soija) This subrange is a hack. FIXME: Make the string value
          //            (and the corresponding string lexeme) to expose the
          //            string *content* directly.
          .add(&literal[1..literal.len() - 1], S::StringValue),
      );
      builder.add_text(TB::new().add("\"", S::StringDecoration));
    }
    V::SymbolicValue { literal } => {
      builder.add_text(TB::new().add("##", S::SymbolicValueDecoration));
      builder.add_text(TB::new().add(literal.clone(), S::SymbolicValue));
    }
    V::Symbol { namespace, name } => {
      builder.add_text(
        TB::new()
          .apply(|b| {
            if let Some(ns) = namespace {
              b.add(ns.clone(), S::SymbolNamespace)
                .add("/", S::SymbolDecoration)
            } else {
              b
            }
          })
          .add(name.clone(), S::SymbolName),
      );
    }
    V::Keyword {
      namespace,
      name,
      alias,
    } => builder.add_text(
      TB::new()
        .add(if *alias { "::" } else { ":" }, S::KeywordDecoration)
        .apply(|b| {
          if let Some(ns) = namespace {
            b.add(ns.clone(), S::KeywordNamespace)
              .add("/", S::KeywordDecoration)
          } else {
            b
          }
        })
        .add(name.clone(), S::KeywordName),
    ),
    V::VarQuoted { namespace, name } => builder.add_text(
      TB::new()
        .add("#'", S::VarQuoteDecoration)
        .apply(|b| {
          if let Some(ns) = namespace {
            b.add(ns.clone(), S::VarQuoteNamespace)
              .add("/", S::VarQuoteDecoration)
          } else {
            b
          }
        })
        .add(name.clone(), S::VarQuoteName),
    ),
    V::TaggedLiteral {
      namespace,
      name,
      value,
    } => {
      builder.add_text(
        TB::new()
          .add("#", S::TaggedLiteralDecoration)
          .apply(|b| {
            if let Some(ns) = namespace {
              b.add(ns.clone(), S::TaggedLiteralNamespace)
                .add("/", S::TaggedLiteralDecoration)
            } else {
              b
            }
          })
          .add(name.clone(), S::TaggedLiteralName),
      );
      builder.add_soft_space();
      program_value(builder, value);
    }
    V::List { values } => program_seq(builder, values, true, "(", ")"),
    V::Vector { values } => program_seq(builder, values, false, "[", "]"),
    V::Set { values } => program_seq(builder, values, false, "#{", "}"),
    V::Map { entries } => program_map(builder, entries),
  }
}

/// Generate a program for a delimited sequence of values.
fn program_seq(
  builder: &mut ProgramBuilder,
  values: &[Value],
  anchor_after_first: bool,
  opening_delim: &'static str,
  closing_delim: &'static str,
) {
  let only_simple_values = values.iter().all(|v| {
    !matches!(
      v,
      Value::List { .. }
        | Value::Vector { .. }
        | Value::Set { .. }
        | Value::Map { .. }
    )
  });

  builder.add_text(TB::new().add(opening_delim, S::CollectionDelimiter));

  let mut it = values.iter();

  if anchor_after_first {
    if let Some(first) = it.next() {
      program_value(builder, first);
      builder.add_soft_space();
    }
  }

  if let Some(first) = it.next() {
    let anchor = builder.set_anchor();
    program_value(builder, first);

    for value in it {
      if only_simple_values {
        builder.add_soft_space();
      } else {
        builder.break_hard(anchor);
      }
      program_value(builder, value);
    }
  }

  builder.add_text(TB::new().add(closing_delim, S::CollectionDelimiter));
}

/// Generate program for a map structure.
fn program_map(builder: &mut ProgramBuilder, entries: &[MapEntry]) {
  builder.add_text(TB::new().add("{", S::CollectionDelimiter));

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

      program_value(builder, &first.key);

      builder.jump_to(value_anchor);
      program_value(builder, &first.value);

      for entry in it {
        builder.break_hard(anchor);
        program_value(builder, &entry.key);

        builder.jump_to(value_anchor);
        program_value(builder, &entry.value);
      }
    } else {
      program_value(builder, &first.key);

      builder.add_soft_space();
      program_value(builder, &first.value);

      for entry in it {
        builder.break_hard(anchor);
        program_value(builder, &entry.key);

        builder.add_soft_space();
        program_value(builder, &entry.value);
      }
    }
  }

  builder.add_text(TB::new().add("}", S::CollectionDelimiter));
}

/// The width of a rigid value, i.e. a value that in no circumstance can be
/// split or combined to occupy a narrower or wider horizontal span.
//
// XXX(soija) In fact, we could call this a natural width and some values
//            (lists, maps, and the like) just don't have it.
fn rigid_width(value: &Value) -> Option<usize> {
  match value {
    V::Nil => Some(ascii_len("nil")),
    V::Number { literal } => Some(literal.len()),
    V::String { literal } => Some(literal.len()),
    V::Boolean { value } => Some(if *value {
      ascii_len("true")
    } else {
      ascii_len("false")
    }),
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
        + namespace.as_ref().map(|s| s.len() + 1).unwrap_or(0)
        + name.len(),
    ),
    V::Symbol { namespace, name } => Some(if let Some(ns) = namespace {
      ns.len() + 1 + name.len()
    } else {
      name.len()
    }),
    _ => None,
  }
}

/// Returns the length of the given ASCII string.  Panics, if `s` contains
/// non-ASCII characters.
const fn ascii_len(s: &str) -> usize {
  let mut i = 0;
  let b = s.as_bytes();
  let n = s.len();
  while i < n {
    if b[i] > 0x7F {
      panic!("non-ascii str");
    }
    i += 1;
  }
  s.len()
}
