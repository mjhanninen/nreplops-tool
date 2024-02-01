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
  env, ffi,
  io::{self, IsTerminal},
  path,
  rc::Rc,
  time,
};

use clap::Parser;

use crate::{
  conn_expr::{ConnectionExpr, ConnectionExprSource},
  error::Error,
  version::{Version, VersionRange},
};

pub use self::tristate::Tristate;

#[derive(Debug)]
pub struct Args {
  pub version_range: Option<VersionRange>,
  pub conn_expr_src: ConnectionExprSource,
  pub stdin_from: Option<IoArg>,
  pub stdout_to: Option<IoArg>,
  pub stderr_to: Option<IoArg>,
  pub results_to: Option<IoArg>,
  pub source_args: Vec<SourceArg>,
  pub template_args: Vec<TemplateArg>,
  pub pretty: Tristate,
  pub color: Tristate,
}

impl Args {
  pub fn from_command_line() -> Result<Self, Error> {
    let mut args_os = env::args_os().collect::<Vec<ffi::OsString>>();

    // Inject an "any" version range after the shebang mode flag (`-!`) in
    // certain cases.  This helps Clap to not confuse positional arguments with
    // the optional version assertion.
    if let Some(pos) =
      args_os.iter().position(|arg| *arg == ffi::OsStr::new("-!"))
    {
      if args_os
        .get(pos + 1)
        .and_then(|value| value.to_str())
        .map(|value| parse_version_range(value).is_err())
        .unwrap_or(true)
      {
        args_os.insert(pos + 1, "..".into());
      }
    }

    Self::try_from(Cli::parse_from(args_os))
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
    let (shebang_mode, assert_version) = match cli.shebang_guard {
      Some(version) => (true, version),
      None => (false, None),
    };

    let mut pos_arg_it = cli.pos_args.iter();

    // XXX(soija) I don't like the how expressions and files are mutually
    //            exclusive. Figure out a way to lift this restriction SO that
    //            the relative order of expressions and files is honored.  That
    //            is, `-e a -f b -e c` should be sourced in order `a`, `b`,
    //            and `c`.

    let source_args = if shebang_mode {
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
    //
    // XXX(soija) Is this correct? `--stdin` determines what gets sent to the
    //            server as stdin and `stdin_reserved` means that the source is
    //            read from the local stdin.  So, I think, it should be okay to
    //
    //                cat program.clj | nr --stdin input.txt
    //
    //            which should be more or less equivalent to
    //
    //                nr --file program.clj --stdin input.txt
    //
    //            Right?
    //
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

    fn tristate(on: bool, off: bool) -> Tristate {
      use Tristate::*;
      match (on, off) {
        (true, false) => On,
        (false, true) => Off,
        _ => Auto,
      }
    }

    Ok(Self {
      version_range: assert_version,
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
      pretty: tristate(cli.pretty, cli.no_pretty),
      color: tristate(cli.color, cli.no_color),
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

mod tristate {

  use std::{fmt, str};

  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum Tristate {
    On,
    Off,
    Auto,
  }

  impl Tristate {
    pub fn to_bool(&self, default: bool) -> bool {
      use Tristate::*;
      match self {
        On => true,
        Off => false,
        Auto => default,
      }
    }
  }

  impl fmt::Display for Tristate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      use Tristate::*;
      write!(
        f,
        "{}",
        match self {
          On => "on",
          Off => "off",
          Auto => "auto",
        }
      )
    }
  }

  impl str::FromStr for Tristate {
    type Err = ParseTristateSwitchError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
      use Tristate::*;
      match s {
        "on" => Ok(On),
        "off" => Ok(Off),
        "auto" => Ok(Auto),
        _ => Err(ParseTristateSwitchError),
      }
    }
  }

  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub struct ParseTristateSwitchError;

  #[cfg(test)]
  mod test {
    use super::*;
    use Tristate::*;

    #[test]
    fn parse_tristate_switch() {
      assert_eq!("on".parse::<Tristate>(), Ok(On));
      assert_eq!("off".parse::<Tristate>(), Ok(Off));
      assert_eq!("auto".parse::<Tristate>(), Ok(Auto));

      assert_eq!("".parse::<Tristate>(), Err(ParseTristateSwitchError));
      assert_eq!(" on".parse::<Tristate>(), Err(ParseTristateSwitchError));
      assert_eq!(
        "whatever".parse::<Tristate>(),
        Err(ParseTristateSwitchError)
      );
    }
  }
}

//
// Clap cli
//

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
    value_name = "VERSION_EXPR",
    value_parser = parse_version_range,
    conflicts_with_all = &["exprs", "files"],
  )]
  shebang_guard: Option<Option<VersionRange>>,

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

  /// Enforce result value pretty-printing
  #[arg(long, conflicts_with = "no_pretty")]
  pretty: bool,

  /// Enforce unformatted result values
  #[arg(long, conflicts_with = "pretty")]
  no_pretty: bool,

  /// Enforce colored output
  #[arg(long, conflicts_with = "no_color")]
  color: bool,

  /// Enforce plain output
  #[arg(long, conflicts_with = "color")]
  no_color: bool,
}

fn parse_version_range(s: &str) -> Result<VersionRange, &'static str> {
  if let Ok(start) = s.parse::<Version>() {
    let end = start.next_breaking();
    Ok(VersionRange {
      start: Some(start),
      end: Some(end),
      inclusive: false,
    })
  } else if let Ok(range) = s.parse::<VersionRange>() {
    Ok(range)
  } else {
    Err("bad version or version range")
  }
}

fn not_implemented<T>(_: &str) -> Result<T, &'static str> {
  Err("this feature has not been implemented yet, sorry")
}
