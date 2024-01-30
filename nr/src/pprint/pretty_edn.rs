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

pub fn convert_to_layout_program<'a>(value: &Value<'a>) -> Program<'a> {
  let mut builder = ProgramBuilder::new();
  chunks_from_value(&mut builder, value);
  builder.build()
}

fn chunks_from_value<'a>(builder: &mut ProgramBuilder<'a>, value: &Value<'a>) {
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
      builder.add_text(TB::new().add(*literal, S::NumberValue));
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
      builder.add_text(TB::new().add(*literal, S::SymbolicValue));
    }
    V::Symbol { namespace, name } => {
      builder.add_text(
        TB::new()
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
      TB::new()
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
        TB::new()
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
  builder: &mut ProgramBuilder<'a>,
  values: &[Value<'a>],
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

  builder.add_text(TB::new().add(closing_delim, S::CollectionDelimiter));
}

fn chunks_from_map<'a>(
  builder: &mut ProgramBuilder<'a>,
  entries: &[MapEntry<'a>],
) {
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

  builder.add_text(TB::new().add("}", S::CollectionDelimiter));
}

/// The width of a rigid value, i.e. a value that in no circumstance can be
/// split or combined to occupy a narrower or wider horizontal span.
//
// XXX(soija) In fact, we could call this a natural width and some values
//            (lists, maps, and the like) just don't have it.
fn rigid_width(value: &Value) -> Option<usize> {
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
