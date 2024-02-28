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
  pub fragments: Box<[Fragment]>,
  pub source: Source,
}

/// A fragment of a (top-level) form.
#[derive(Debug)]
pub enum Fragment {
  Lexemes(Box<[Lexeme]>),
  Directive(DirectiveFragment),
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
    .filter(|l| !matches!(l.token, T::Whitespace | T::Comment))
    .peekable();
  while let Some(form) = try_parse_form(source, &mut lexemes)? {
    forms.push(form);
  }
  Ok(())
}

fn try_parse_form<I>(
  source: &Source,
  lexemes: &mut Peekable<I>,
) -> Result<Option<Form>, Error>
where
  I: Iterator<Item = Lexeme>,
{
  let mut collector = FragmentCollector::new();
  while lexemes.peek().is_some() {
    collect_form(lexemes, &mut collector)?;
    if !collector.is_empty() {
      return Ok(Some(Form {
        fragments: collector.build(),
        source: source.clone(),
      }));
    }
  }
  Ok(None)
}

#[derive(Default)]
struct FragmentCollector {
  unfinished: Vec<Lexeme>,
  fragments: Vec<Fragment>,
}

impl FragmentCollector {
  fn new() -> Self {
    Self::default()
  }

  fn collect_lexeme(&mut self, lexeme: Lexeme) {
    self.unfinished.push(lexeme);
  }

  fn is_empty(&self) -> bool {
    self.fragments.is_empty() && self.unfinished.is_empty()
  }

  fn build(mut self) -> Box<[Fragment]> {
    if !self.unfinished.is_empty() {
      self
        .fragments
        .push(Fragment::Lexemes(self.unfinished.into_boxed_slice()));
    }
    self.fragments.into()
  }
}

fn collect_form<I>(
  lexemes: &mut Peekable<I>,
  collector: &mut FragmentCollector,
) -> Result<(), Error>
where
  I: Iterator<Item = Lexeme>,
{
  if let Some(parent_ix) = lexemes.peek().map(|l| l.parent_ix) {
    collect_form_recursively(parent_ix, lexemes, collector)
  } else {
    Ok(())
  }
}

fn collect_form_recursively<I>(
  parent_ix: Ix,
  lexemes: &mut Peekable<I>,
  collector: &mut FragmentCollector,
) -> Result<(), Error>
where
  I: Iterator<Item = Lexeme>,
{
  use Action as A;

  #[derive(Debug)]
  enum Action {
    Discard,
    CollectJustThis,
    CollectCountedChildren(usize),
    CollectDelimitedChildren,
  }

  let action = {
    let lexeme = lexemes.peek().unwrap();

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

      T::Meta { .. } => A::CollectCountedChildren(2),

      T::String { .. } | T::Symbol { .. } => A::CollectJustThis,

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
      collector.collect_lexeme(lexemes.next().unwrap());
      for _ in 0..n {
        collect_form_recursively(form_ix, lexemes, collector)?;
      }
    }
    A::CollectDelimitedChildren => {
      let start_lexeme = lexemes.next().unwrap();
      let form_ix = start_lexeme.form_ix;
      collector.collect_lexeme(lexemes.next().unwrap());
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

fn discard_recursively<I>(lexemes: &mut Peekable<I>)
where
  I: Iterator<Item = Lexeme>,
{
  let lexeme = lexemes.next().unwrap();

  #[allow(clippy::enum_variant_names)]
  enum Discard {
    JustThis,
    CountedChildren(usize),
    DelimitedChildren,
  }

  let action = match lexeme.token {
    (T::Nil
    | T::Boolean { .. }
    | T::Numeric { .. }
    | T::Char { .. }
    | T::String { .. }
    | T::Regex { .. }
    | T::SymbolicValue { .. }
    | T::Symbol { .. }
    | T::Keyword { .. }
    | T::Tag { .. }) => Discard::JustThis,

    (T::Discard
    | T::Quote
    | T::VarQuote
    | T::Synquote
    | T::Unquote
    | T::SplicingUnquote) => Discard::CountedChildren(1),

    T::Meta { .. } => Discard::CountedChildren(2),

    (T::StartList
    | T::StartVector
    | T::StartSet
    | T::StartMap
    | T::StartAnonymousFn
    | T::StartReaderConditional) => Discard::DelimitedChildren,

    _ => panic!("unexpected lexeme while discarding: lexeme = {lexeme:?}",),
  };

  match action {
    Discard::JustThis => {}

    Discard::CountedChildren(n) => {
      for _ in 0..n {
        let child = lexemes.peek().unwrap();
        if child.parent_ix == lexeme.form_ix {
          discard_recursively(lexemes);
        } else {
          panic!(
            "unexpected lexeme while discarding child: parent = {lexeme:?}, child {child:?}"
          );
        }
      }
    }

    Discard::DelimitedChildren => loop {
      let child_or_end = lexemes.peek().unwrap();
      match child_or_end.token {
          (T::EndList
          | T::EndVector
          | T::EndSet
          | T::EndMap
          | T::EndAnonymousFn
          | T::EndReaderConditional)
            if child_or_end.form_ix == lexeme.form_ix => {
            lexemes.next().unwrap();
            break;
          },
          _ if child_or_end.parent_ix == lexeme.form_ix => discard_recursively( lexemes),
          _ => panic!("unexpected lexeme while discarding delimited children: parent = {lexeme:?}, child = {child_or_end:?}"),
        }
    },
  }
}
