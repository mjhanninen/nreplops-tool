// cli.rs
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

use std::{
  ffi,
  io::{self, IsTerminal},
  path,
  rc::Rc,
  time,
};

use clap::Parser;

use crate::{
  conn_expr::{ConnectionExpr, ConnectionExprSource},
  error::Error,
};

#[derive(Debug)]
pub struct Args {
  pub conn_expr_src: ConnectionExprSource,
  pub stdin_from: Option<IoArg>,
  pub stdout_to: Option<IoArg>,
  pub stderr_to: Option<IoArg>,
  pub results_to: Option<IoArg>,
  pub source_args: Vec<SourceArg>,
  pub template_args: Vec<TemplateArg>,
}

impl Args {
  pub fn from_command_line() -> Result<Self, Error> {
    Self::try_from(Cli::parse())
  }
}

#[derive(Debug, PartialEq)]
pub enum IoArg {
  Pipe,
  File(path::PathBuf),
}

#[derive(Debug, PartialEq)]
pub enum SourceArg {
  Pipe,
  Expr(String),
  File(path::PathBuf),
}

impl SourceArg {
  fn is_pipe(&self) -> bool {
    matches!(*self, SourceArg::Pipe)
  }
}

impl From<IoArg> for SourceArg {
  fn from(io: IoArg) -> Self {
    match io {
      IoArg::Pipe => SourceArg::Pipe,
      IoArg::File(p) => SourceArg::File(p),
    }
  }
}

#[derive(Debug)]
pub struct TemplateArg {
  pub pos: Option<usize>,
  pub name: Option<Rc<str>>,
  pub value: Rc<str>,
}

impl TryFrom<Cli> for Args {
  type Error = Error;

  fn try_from(cli: Cli) -> Result<Self, Self::Error> {
    let mut pos_arg_it = cli.pos_args.iter();

    let source_args = if cli.shebang_guard.is_some() {
      //
      // When the shebang guard is given the first positional argument is always
      // interpreted as the (sole) source.
      //
      let pos_arg = pos_arg_it.next().ok_or(Error::NoInput)?;
      let src_arg = IoArg::parse_from_path(pos_arg)
        .map(SourceArg::from)
        .map_err(|_| Error::BadSourceFile)?;
      vec![src_arg]
    } else if !cli.exprs.is_empty() {
      cli
        .exprs
        .iter()
        .map(|e| SourceArg::Expr(e.clone()))
        .collect()
    } else if !cli.files.is_empty() {
      cli
        .files
        .iter()
        .map(|f| {
          IoArg::parse_from_path_or_pipe(f)
            .map(SourceArg::from)
            .map_err(|_| Error::BadSourceFile)
        })
        .collect::<Result<_, _>>()?
    } else if !io::stdin().is_terminal() {
      vec![SourceArg::Pipe]
    } else if let Some(f) = pos_arg_it.next() {
      vec![IoArg::parse_from_path_or_pipe(f)
        .map(SourceArg::from)
        .map_err(|_| Error::BadSourceFile)?]
    } else if cli.wait_port_file.is_some() {
      vec![]
    } else {
      return Err(Error::NoInput);
    };

    let stdin_reserved =
      match source_args.iter().filter(|s| s.is_pipe()).count() {
        0 => false,
        1 => true,
        _ => return Err(Error::StdInConflict),
      };

    let stdin_from = cli
      .stdin
      .as_ref()
      .map(IoArg::try_from)
      .transpose()
      .map_err(|_| Error::BadStdIn)?;
    if stdin_from.is_some() && stdin_reserved {
      return Err(Error::StdInConflict);
    }

    let args = cli
      .args
      .iter()
      .map(|arg| TemplateArg::parse(None, arg))
      .chain(
        pos_arg_it
          .enumerate()
          .map(|(i, arg)| TemplateArg::parse(Some(i), arg)),
      )
      .collect::<Result<_, _>>()?;

    let conn_expr_src = if let Some(ref h) = cli.port {
      h.into()
    } else {
      ConnectionExprSource::PortFile {
        path: cli.port_file.clone(),
        wait_for: cli.wait_port_file.map(time::Duration::from_secs),
      }
    };

    Ok(Self {
      conn_expr_src,
      stdin_from,
      stdout_to: if cli.no_stdout {
        None
      } else {
        Some(
          cli
            .stdout
            .as_ref()
            .map(IoArg::from_path)
            .unwrap_or(IoArg::Pipe),
        )
      },
      stderr_to: if cli.no_stderr {
        None
      } else {
        Some(
          cli
            .stderr
            .as_ref()
            .map(IoArg::from_path)
            .unwrap_or(IoArg::Pipe),
        )
      },
      results_to: if cli.no_results {
        None
      } else {
        Some(
          cli
            .results
            .as_ref()
            .map(IoArg::from_path)
            .unwrap_or(IoArg::Pipe),
        )
      },
      source_args,
      template_args: args,
    })
  }
}

impl TemplateArg {
  fn parse(
    pos: Option<usize>,
    s: impl AsRef<ffi::OsStr>,
  ) -> Result<Self, Error> {
    if let Some(s) = s.as_ref().to_str() {
      if let Some((name, value)) = s.split_once('=') {
        Ok(Self {
          pos,
          name: Some(name.to_string().into()),
          value: value.to_string().into(),
        })
      } else if pos.is_some() {
        Ok(Self {
          pos,
          name: None,
          value: s.to_string().into(),
        })
      } else {
        // non-positional arg must have a name
        Err(Error::UnnamedNonPositionalTemplateArgument)
      }
    } else {
      // arg must be UTF-8 string
      Err(Error::NonUtf8TemplateArgument)
    }
  }
}

impl IoArg {
  fn from_path(path: impl AsRef<path::Path>) -> Self {
    IoArg::File(path.as_ref().to_owned())
  }

  fn parse_from_path(s: impl AsRef<ffi::OsStr>) -> Result<Self, IoParseError> {
    let s = s.as_ref();
    if let Ok(p) = path::PathBuf::from(s).canonicalize() {
      Ok(IoArg::File(p))
    } else {
      Err(IoParseError(s.to_string_lossy().into()))
    }
  }

  fn parse_from_path_or_pipe(
    s: impl AsRef<ffi::OsStr>,
  ) -> Result<Self, IoParseError> {
    let s = s.as_ref();
    if s == "-" {
      Ok(IoArg::Pipe)
    } else if let Ok(p) = path::PathBuf::from(s).canonicalize() {
      Ok(IoArg::File(p))
    } else {
      Err(IoParseError(s.to_string_lossy().into()))
    }
  }
}

impl TryFrom<&ffi::OsString> for IoArg {
  type Error = IoParseError;

  fn try_from(s: &ffi::OsString) -> Result<Self, Self::Error> {
    if s == "-" {
      Ok(IoArg::Pipe)
    } else if let Ok(p) = path::PathBuf::from(s).canonicalize() {
      Ok(IoArg::File(p))
    } else {
      Err(IoParseError(s.to_string_lossy().into()))
    }
  }
}

#[derive(Debug, thiserror::Error)]
#[error("bad file agument: {0}")]
pub struct IoParseError(String);

#[derive(Debug, clap::Parser)]
#[command(
  about = "Non-interactive nREPL client for scripts and command-line",
  version,
  max_term_width = 80
)]
struct Cli {
  /// Connect to server on [HOST:]PORT
  #[arg(
    long,
    short,
    visible_alias = "host",
    value_name = "[[[USER@]TUNNEL[:PORT]:]HOST:]PORT"
  )]
  port: Option<ConnectionExpr>,

  /// Read server port from FILE
  #[arg(long, value_name = "FILE")]
  port_file: Option<path::PathBuf>,

  /// Evaluate within NAMESPACE
  #[arg(long, visible_alias = "namespace", value_name = "NAMESPACE")]
  ns: Option<String>,

  /// Evaluate EXPRESSION
  #[arg(
    long = "expr",
    short,
    value_name = "EXPRESSION",
    conflicts_with = "files"
  )]
  exprs: Vec<String>,

  /// Evaluate FILE
  #[arg(
    long = "file",
    short = 'f',
    value_name = "FILE",
    conflicts_with = "exprs"
  )]
  files: Vec<ffi::OsString>,

  /// Send FILE to server's stdin
  #[arg(long, visible_aliases = &["in", "input"], value_name = "FILE")]
  stdin: Option<ffi::OsString>,

  /// Write server's stdout to FILE
  #[arg(long, visible_aliases = &["out", "output"], value_name = "FILE")]
  stdout: Option<path::PathBuf>,

  /// Discard server's stdout
  #[arg(
        long,
        visible_aliases = &["no-out", "no-output"],
        conflicts_with = "stdout",
    )]
  no_stdout: bool,

  /// Write server's stderr to FILE
  #[arg(long, visible_alias = "err", value_name = "FILE")]
  stderr: Option<path::PathBuf>,

  /// Discard server's stderr
  #[arg(
        long,
        visible_aliases = &["no-err"],
        conflicts_with = "stderr",
    )]
  no_stderr: bool,

  /// Write evaluation results to FILE
  #[arg(long, visible_aliases = &["res", "values"], value_name = "FILE")]
  results: Option<path::PathBuf>,

  /// Discard evaluation results
  #[arg(
        long,
        visible_aliases = &["no-res", "no-values"],
        conflicts_with = "results",
    )]
  no_results: bool,

  /// Set template argument NAME to VALUE
  #[arg(long = "arg", short = 'a', value_name = "NAME=VALUE")]
  args: Vec<String>,

  /// Positional arguments
  #[arg(value_name = "ARG")]
  pos_args: Vec<ffi::OsString>,

  /// Run in shebang (#!) mode
  #[arg(
        short = '!',
        value_name = "MIN_VERSION",
        conflicts_with_all = &["exprs", "files"],
    )]
  shebang_guard: Option<Option<String>>,

  /// Wait .nrepl-port file to appear for SECONDS
  #[arg(long = "wait-port-file", value_name = "SECONDS")]
  wait_port_file: Option<u64>,

  /// Set timeout for program execution
  #[arg(
        long = "timeout",
        value_name = "SECONDS",
        value_parser = not_implemented::<u32>,
    )]
  _timeout: Option<u32>,
}

fn not_implemented<T>(_: &str) -> Result<T, &'static str> {
  Err("this feature has not been implemented yet, sorry")
}
