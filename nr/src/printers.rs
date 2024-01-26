// printer.rs
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

use crate::clojure::{
  lex::Lexeme,
  result_ir::{self, KeywordNamespace, MapEntry, Value},
};

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

mod printer {
  use std::io::{self, Write};

  use super::fragments::Fragment;
  use crate::style::Style;

  pub enum Command<'a> {
    NewLine,
    Space(u16),
    SetStyle(Style),
    ResetStyle(Style),
    Text(&'a str),
  }

  impl<'a> Command<'a> {
    fn is_set_style(&self) -> bool {
      matches!(self, Command::SetStyle(_))
    }
  }

  const SPACES_LEN: usize = 64;
  static SPACES: [u8; SPACES_LEN] = [b' '; SPACES_LEN];

  pub fn print<'a, W, I>(
    writer: &mut W,
    input: I,
    use_color: bool,
  ) -> io::Result<()>
  where
    W: Write,
    I: Iterator<Item = &'a Command<'a>>,
  {
    use Command as C;

    let mut current_color = None;
    let mut it = input.peekable();

    while let Some(command) = it.next() {
      match command {
        C::NewLine => writeln!(writer)?,
        C::Space(amount) => {
          let mut remaining = *amount as usize;
          while remaining > SPACES_LEN {
            writer.write_all(&SPACES)?;
            remaining -= SPACES_LEN;
          }
          if remaining > 0 {
            writer.write_all(&SPACES[0..remaining])?;
          }
        }
        C::SetStyle(style) => {
          if use_color {
            let new_color = style.to_ansi_color();
            match current_color {
              Some(old_color) if old_color == new_color => (),
              _ => {
                write!(writer, "{}", new_color.render_fg())?;
                current_color = Some(new_color);
              }
            }
          }
        }
        C::ResetStyle(style) => {
          if use_color
            && it.peek().map(|next| !next.is_set_style()).unwrap_or(true)
          {
            write!(writer, "{}", anstyle::Reset.render())?;
            current_color = None;
          }
        }
        C::Text(text) => writer.write_all(text.as_bytes())?,
      }
    }
    Ok(())
  }

  pub trait BuildInput<'a> {
    fn add_new_line(&mut self);
    fn add_spaces(&mut self, amount: u16);
    fn add_fragment(&mut self, fragment: &'a Fragment<'a>);
  }

  impl<'a> BuildInput<'a> for Vec<Command<'a>> {
    fn add_new_line(&mut self) {
      self.push(Command::NewLine);
    }
    fn add_spaces(&mut self, amount: u16) {
      self.push(Command::Space(amount))
    }
    fn add_fragment(&mut self, fragment: &'a Fragment<'a>) {
      self.push(Command::SetStyle(fragment.style));
      self.push(Command::Text(fragment.text.as_str()));
      self.push(Command::ResetStyle(fragment.style));
    }
  }
}

use layout_solver::Chunk;

fn clojure_chunks<'a>(value: &Value<'a>) -> Box<[Chunk<'a>]> {
  let mut chunks = Vec::new();
  chunks_from_value(&mut chunks, value);
  chunks.shrink_to_fit();
  chunks.into_boxed_slice()
}

fn chunks_from_value<'a>(chunks: &mut Vec<Chunk<'a>>, value: &Value<'a>) {
  use crate::style::Style as S;
  use fragments::*;
  use layout_solver::*;
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
  use crate::style::Style as S;
  use fragments::*;
  use layout_solver::*;

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
  use crate::style::Style as S;
  use fragments::*;
  use layout_solver::*;

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

mod layout_solver {

  use super::fragments::{Fragment, FragmentText};
  use super::printer::Command;
  use crate::style::Style;

  #[derive(Clone, Debug)]
  pub enum Chunk<'a> {
    /// Marks horizontal
    PushAnchor,
    //
    // XXX(soija) FIXME: Instead using a stacked anchors use indexed ones and
    //            remove the pop.
    //
    /// Removes the latest anchor
    PopAnchor,
    /// Inserts space, except at the start of the line
    SoftSpace,
    /// Unconditional line break
    HardBreak,
    /// Unbreakable strip of fragments
    Text(Box<[Fragment<'a>]>),
  }

  #[derive(Default)]
  pub struct TextBuilder<'a> {
    fragments: Vec<Fragment<'a>>,
  }

  impl<'a> TextBuilder<'a> {
    pub fn new() -> Self {
      Self::default()
    }

    pub fn add<T: Into<FragmentText<'a>>>(
      mut self,
      text: T,
      style: Style,
    ) -> Self {
      self.fragments.push(Fragment::new(style, text));
      self
    }

    pub fn apply<F>(self, mut func: F) -> Self
    where
      F: FnOnce(Self) -> Self,
    {
      func(self)
    }

    pub fn build(mut self) -> Chunk<'a> {
      self.fragments.shrink_to_fit();
      Chunk::Text(self.fragments.into_boxed_slice())
    }
  }

  pub fn solve<'a>(
    chunks: &'a [Chunk<'a>],
    printer_input: &mut Vec<Command<'a>>,
  ) {
    use super::printer::BuildInput;
    use Chunk as C;

    let mut col = 0_u16;
    let mut anchors = vec![0_u16];

    for c in chunks {
      match c {
        C::Text(fragments) => {
          for f in fragments.iter() {
            printer_input.add_fragment(f);
            col += f.width() as u16;
          }
        }
        C::SoftSpace => {
          printer_input.add_spaces(1);
          col += 1;
        }
        C::HardBreak => {
          printer_input.add_new_line();
          col = *anchors.last().unwrap();
          printer_input.add_spaces(col);
        }
        C::PushAnchor => {
          anchors.push(col);
        }
        C::PopAnchor => {
          anchors.pop();
        }
      }
    }
  }
}

mod fragments {

  use std::borrow::Borrow;

  use crate::style::Style;

  #[derive(Clone, Debug)]
  pub struct Fragment<'a> {
    pub style: Style,
    pub text: FragmentText<'a>,
  }

  impl<'a> Fragment<'a> {
    pub fn new<T>(style: Style, text: T) -> Self
    where
      T: Into<FragmentText<'a>>,
    {
      Self {
        style,
        text: text.into(),
      }
    }

    pub fn width(&self) -> usize {
      self.text.as_str().chars().count()
    }
  }

  #[derive(Clone, Debug)]
  pub enum FragmentText<'a> {
    Borrowed(&'a str),
    Owned(Box<str>),
  }

  impl<'a> FragmentText<'a> {
    pub fn as_str(&'a self) -> &'a str {
      self.borrow()
    }
  }

  impl<'a> From<&'a str> for FragmentText<'a> {
    fn from(s: &'a str) -> Self {
      FragmentText::Borrowed(s)
    }
  }

  impl<'a> From<String> for FragmentText<'a> {
    fn from(s: String) -> Self {
      FragmentText::Owned(s.into_boxed_str())
    }
  }

  impl<'a> Borrow<str> for FragmentText<'a> {
    fn borrow(&self) -> &str {
      match self {
        FragmentText::Borrowed(s) => s,
        FragmentText::Owned(s) => s.borrow(),
      }
    }
  }
}
