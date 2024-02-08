// sources.rs
// Copyright 2022 Matti Hänninen
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

use std::{
  fs,
  io::{self, Read},
  iter::Peekable,
  path::Path,
};

use crate::{
  cli,
  clojure::lex::{self, Ix, Lexeme, Token},
  error::Error,
};

use Token as T;

/// A single top-level form.
#[derive(Debug)]
pub struct Form {
  fragments: Box<[Fragment]>,
  source: Source,
}

/// A fragment of a (top-level) form.
#[derive(Debug)]
pub enum Fragment {
  Lexemes(LexemeFragment),
  Directive(DirectiveFragment),
}

/// A fragment of lexemes to used as-is.
#[derive(Debug)]
pub struct LexemeFragment {
  lexemes: Box<[Lexeme]>,
  pos: SourcePos,
}

/// A template directive.
#[derive(Debug)]
pub struct DirectiveFragment {
  /// The command-line argument providing the value.
  arg: Option<ArgId>,
  /// The environment variable providing the value.
  env: Option<Box<str>>,
  /// The prompt to display when asking for the value from the user
  /// interactively.
  prompt: Option<Box<str>>,
  /// The placeholder for the value in the command-line help display.
  placeholder: Option<Box<str>>,
  /// The description in the command-line help display.
  description: Option<Box<str>>,
  /// The default value to be used in case no value is provided.
  default: Option<Box<[Lexeme]>>,
  /// Whether to inject the value as a string literal, value, or spliced value.
  inject_as: InjectAs,
  /// The starting position of the corresponding tagged literal in the original
  /// source.  Used in error reporting (e.g. missing  argument value).
  start: SourcePos,
}

/// A command-line argument identifier
#[derive(Debug)]
pub enum ArgId {
  /// Positional argument
  Positional(usize),
  /// A named argument of the form `--arg NAME=VALUE`.
  Named(Box<str>),
}

/// Governs how the input value should be interpreted and processed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InjectAs {
  /// The value should be escaped and wrapped inside a string literal. The
  /// resulting lexeme should be injected as-is.
  String,
  /// The value should be lexed and the lexemes should be injected as-is.
  Value,
  /// The value should be lexed and the lexemes should be "spliced" into the
  /// containing form in case they represent a container of "congruent" type.
  /// Otherwise an error should be produced.
  SplicedValue,
}

/// The source of the code.
#[derive(Clone, Debug)]
pub enum Source {
  CommandLine { ix: usize },
  StdIn,
  File { path: Box<Path> },
}

/// A location within a single source.
#[derive(Clone, Debug)]
pub struct SourcePos {
  line: u32,
  column: u32,
}

impl From<&lex::Source> for SourcePos {
  fn from(value: &lex::Source) -> Self {
    Self {
      line: value.line,
      column: value.column,
    }
  }
}

pub fn load_sources(
  source_args: &[cli::SourceArg],
) -> Result<Vec<Form>, Error> {
  let mut result = Vec::new();
  for source_arg in source_args.iter() {
    let (source, raw_content) = load_content(source_arg)?;
    dbg!(&source);
    dbg!(&raw_content);
    let _ = source;
    let _ = raw_content;
    parse_forms(&source, &raw_content, &mut result)?;
  }
  Ok(result)
}

fn load_content(
  source_arg: &cli::SourceArg,
) -> Result<(Source, Box<str>), Error> {
  use cli::SourceArg as A;
  match source_arg {
    A::Pipe => {
      let stdin = io::stdin();
      let mut handle = stdin.lock();
      let mut buffer = String::new();
      handle
        .read_to_string(&mut buffer)
        .map_err(|_| Error::CannotReadStdIn)?;
      Ok((Source::StdIn, buffer.into()))
    }
    A::Expr { ix, expr } => Ok((Source::CommandLine { ix: *ix }, expr.clone())),
    A::File { path } => {
      let Ok(mut file) = fs::File::open(path) else {
        return Err(Error::CannotReadFile(path.to_string_lossy().to_string()));
      };
      let mut buffer = String::new();
      if file.read_to_string(&mut buffer).is_err() {
        return Err(Error::CannotReadFile(path.to_string_lossy().to_string()));
      }
      Ok((Source::File { path: path.clone() }, buffer.into()))
    }
  }
}

fn parse_forms(
  source: &Source,
  input: &str,
  forms: &mut Vec<Form>,
) -> Result<(), Error> {
  let mut lexemes = lex::lex(input)
    .map_err(|e| Error::FailedToParseInput(e.into()))?
    .into_iter()
    .peekable();
  while let Some(form) = parse_form(source, &mut lexemes)? {
    forms.push(form);
  }
  Ok(())
}

fn parse_form<I>(
  source: &Source,
  lexemes: &mut Peekable<I>,
) -> Result<Option<Form>, Error>
where
  I: Iterator<Item = Lexeme>,
{
  let mut fragments = Vec::new();
  parse_form_inner(lexemes, &mut fragments)?;
  if fragments.is_empty() {
    Ok(None)
  } else {
    Ok(Some(Form {
      fragments: fragments.into_boxed_slice(),
      source: source.clone(),
    }))
  }
}

struct FragmentCollector {
  unfinished: Vec<Lexeme>,
  fragments: Vec<Fragment>,
}

impl FragmentCollector {
  fn collect_lexeme(&mut self, lexeme: Lexeme) {
    self.unfinished.push(lexeme);
  }

  fn build(mut self) -> Box<[Fragment]> {
    if !self.unfinished.is_empty() {
      let pos = self
        .unfinished
        .first()
        .unwrap()
        .source
        .as_ref()
        .unwrap()
        .into();
      let lexemes = self.unfinished.into_boxed_slice();
      self
        .fragments
        .push(Fragment::Lexemes(LexemeFragment { lexemes, pos }));
    }

    self.fragments.into()
  }
}

fn parse_form_inner<I>(
  lexemes: &mut Peekable<I>,
  fragments: &mut [Fragment],
) -> Result<(), Error>
where
  I: Iterator<Item = Lexeme>,
{
  use Action as A;
  use Token as T;

  enum Action {
    DiscardForm(Ix),
    CollectChildren(Ix),
  }

  loop {
    let action = {
      let l = lexemes.peek().expect("unexpected end of lexemes");

      match l.token {
        T::Discard => A::DiscardForm(l.form_ix),
        T::StartList
        | T::StartVector
        | T::StartSet
        | T::StartMap
        | T::StartAnonymousFn
        | T::StartReaderConditional => A::CollectChildren(l.form_ix),

        _ => panic!("unexpected lexeme: {l:?}"),
      }
    };

    match action {
      A::DiscardForm(form_ix) => {
        lexemes.next().unwrap();
        discard_child_of(form_ix, lexemes);
      }
      A::CollectChildren(form_ix) => {}
    }
  }

  Ok(())
}

fn discard_child_of<I>(parent: Ix, lexemes: &mut Peekable<I>)
where
  I: Iterator<Item = Lexeme>,
{
  use Action as A;

  #[allow(clippy::enum_variant_names)]
  enum Action {
    DiscardAndStop,
    DiscardAndContinue,
    DiscardChildren(Ix, Option<usize>),
  }

  loop {
    skip_whitespace(lexemes);

    let action = {
      let l = lexemes.peek().unwrap();
      match l.token {
        (T::Nil
        | T::Boolean { .. }
        | T::Numeric { .. }
        | T::Char { .. }
        | T::String { .. }
        | T::Regex { .. }
        | T::SymbolicValue { .. }
        | T::Symbol { .. }
        | T::Keyword { .. }
        | T::Tag { .. })
          if l.parent_ix == parent =>
        {
          A::DiscardAndStop
        }

        (T::SymbolicValuePrefix
        | T::MapQualifier { .. }
        | T::ReaderConditionalPrefix { .. })
          if l.parent_ix == parent =>
        {
          A::DiscardAndContinue
        }

        (T::Discard
        | T::Quote
        | T::VarQuote
        | T::Synquote
        | T::Unquote
        | T::SplicingUnquote)
          if l.parent_ix == parent =>
        {
          A::DiscardChildren(l.form_ix, Some(1))
        }

        (T::StartList
        | T::StartVector
        | T::StartSet
        | T::StartMap
        | T::StartAnonymousFn
        | T::StartReaderConditional)
          if l.parent_ix == parent =>
        {
          A::DiscardChildren(l.form_ix, None)
        }

        _ => panic!(
          "unexpected lexeme while discarding:\nlexeme = {l:?}\nparent = {parent}",
        ),
      }
    };

    match action {
      A::DiscardAndStop => {
        lexemes.next().unwrap();
        return;
      }
      A::DiscardAndContinue => {
        lexemes.next().unwrap();
      }
      A::DiscardChildren(form_ix, how_many) => {
        lexemes.next().unwrap();
        if let Some(n) = how_many {
          for _ in 0..n {
            discard_child_of(form_ix, lexemes);
          }
        } else {
          discard_children_of(form_ix, lexemes);
        }
        return;
      }
    }
  }
}

// Discards children until the lexeme that closes the parent form is encountered
fn discard_children_of<I>(parent: u32, lexemes: &mut Peekable<I>)
where
  I: Iterator<Item = Lexeme>,
{
  loop {
    skip_whitespace(lexemes);

    let l = lexemes.peek().unwrap();
    match l.token {
      (T::EndList
      | T::EndVector
      | T::EndSet
      | T::EndMap
      | T::EndAnonymousFn
      | T::EndReaderConditional)
        if l.form_ix == parent =>
      {
        lexemes.next().unwrap();
        break;
      }

      _ => discard_child_of(parent, lexemes),
    }
  }
}

fn skip_whitespace<I>(lexemes: &mut Peekable<I>)
where
  I: Iterator<Item = Lexeme>,
{
  while lexemes
    .next_if(|l| matches!(l.token, T::Whitespace | T::Comment { .. }))
    .is_some()
  {}
}
