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

use std::{ffi, io, net, path};

use clap::Parser;
use is_terminal::IsTerminal;

use crate::error::Error;

#[derive(Debug)]
pub struct Args {
    pub host: HostArg,
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
    pub name: Option<String>,
    pub value: String,
}

impl TryFrom<Cli> for Args {
    type Error = Error;

    fn try_from(cli: Cli) -> Result<Self, Self::Error> {
        let mut pos_arg_it = cli.pos_args.iter();

        let source_args = if !cli.exprs.is_empty() {
            cli.exprs
                .iter()
                .map(|e| SourceArg::Expr(e.clone()))
                .collect()
        } else if !cli.files.is_empty() {
            cli.files
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

        let host = if let Some(ref h) = cli.port {
            HostArg::HostExpr(h.clone())
        } else {
            HostArg::PortFile(cli.port_file.clone())
        };

        Ok(Self {
            host,
            stdin_from,
            stdout_to: if cli.no_stdout {
                None
            } else {
                Some(
                    cli.stdout
                        .as_ref()
                        .map(IoArg::from_path)
                        .unwrap_or(IoArg::Pipe),
                )
            },
            stderr_to: if cli.no_stderr {
                None
            } else {
                Some(
                    cli.stderr
                        .as_ref()
                        .map(IoArg::from_path)
                        .unwrap_or(IoArg::Pipe),
                )
            },
            results_to: if cli.no_results {
                None
            } else {
                Some(
                    cli.results
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
                    name: Some(name.to_owned()),
                    value: value.to_owned(),
                })
            } else if pos.is_some() {
                Ok(Self {
                    pos,
                    name: None,
                    value: s.to_owned(),
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
#[clap(about = "An nREPL client for the command line", max_term_width = 80)]
struct Cli {
    /// Connect to server on [HOST:]PORT
    #[clap(long, short, visible_alias = "host", value_name = "[HOST:]PORT")]
    port: Option<HostExpr>,

    /// Read server port from FILE
    #[clap(long, value_name = "FILE")]
    port_file: Option<path::PathBuf>,

    /// Evaluate within NAMESPACE
    #[clap(long, visible_alias = "namespace", value_name = "NAMESPACE")]
    ns: Option<String>,

    /// Evaluate EXPRESSION
    #[clap(
        long = "expr",
        short,
        value_name = "EXPRESSION",
        conflicts_with = "files"
    )]
    exprs: Vec<String>,

    /// Evaluate FILE
    #[clap(
        long = "file",
        short = 'f',
        value_name = "FILE",
        conflicts_with = "exprs"
    )]
    files: Vec<ffi::OsString>,

    /// Send FILE to server's stdin
    #[clap(long, visible_aliases = &["in", "input"], value_name = "FILE")]
    stdin: Option<ffi::OsString>,

    /// Write server's stdout to FILE
    #[clap(long, visible_aliases = &["out", "output"], value_name = "FILE")]
    stdout: Option<path::PathBuf>,

    /// Discard server's stdout
    #[clap(
        long,
        visible_aliases = &["no-out", "no-output"],
        conflicts_with = "stdout",
    )]
    no_stdout: bool,

    /// Write server's stderr to FILE
    #[clap(long, visible_alias = "err", value_name = "FILE")]
    stderr: Option<path::PathBuf>,

    /// Discard server's stderr
    #[clap(
        long,
        visible_aliases = &["no-err", "no-error"],
        conflicts_with = "stderr",
    )]
    no_stderr: bool,

    /// Write evaluation results to FILE
    #[clap(long, visible_aliases = &["res", "values"], value_name = "FILE")]
    results: Option<path::PathBuf>,

    /// Discard evaluation results
    #[clap(
        long,
        visible_aliases = &["no-res", "no-values"],
        conflicts_with = "results",
    )]
    no_results: bool,

    /// Set template argument NAME to VALUE
    #[clap(long = "arg", short = 'a', value_name = "NAME=VALUE")]
    args: Vec<String>,

    #[clap(value_name = "ARG")]
    pos_args: Vec<ffi::OsString>,

    /// Shebang-safe mode
    #[clap(
        long = "shebang-guard",
        short = '!',
        conflicts_with_all = &["exprs", "files"],
    )]
    _shebang_guard: bool,
}

#[derive(Debug)]
pub enum HostArg {
    HostExpr(HostExpr),
    PortFile(Option<path::PathBuf>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum HostExpr {
    Port(u16),
    DomainPort(String, u16),
    SocketAddr(net::SocketAddr),
}

impl std::str::FromStr for HostExpr {
    type Err = HostOptionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(port) = s.parse::<u16>() {
            return Ok(HostExpr::Port(port));
        }
        if let Ok(socket_addr) = s.parse::<net::SocketAddr>() {
            return Ok(HostExpr::SocketAddr(socket_addr));
        }
        if let Some((host_part, port_part)) = s.rsplit_once(':') {
            if let Ok(port) = port_part.parse::<u16>() {
                if let Ok(domain) = addr::parse_domain_name(host_part) {
                    return Ok(HostExpr::DomainPort(domain.to_string(), port));
                }
            }
        }
        Err(HostOptionParseError)
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("bad host and port expression")]
pub struct HostOptionParseError;

#[cfg(test)]
mod test_options {

    use super::*;
    use std::net;

    #[test]
    fn host_option_parsing() {
        assert_eq!("5678".parse::<HostExpr>(), Ok(HostExpr::Port(5678)),);
        assert_eq!(
            "1.2.3.4:5678".parse::<HostExpr>(),
            Ok(HostExpr::SocketAddr(net::SocketAddr::from((
                [1, 2, 3, 4],
                5678
            )))),
        );
        assert_eq!(
            "localhost:5678".parse::<HostExpr>(),
            Ok(HostExpr::DomainPort("localhost".to_string(), 5678)),
        );
        assert_eq!(
            "example.com:5678".parse::<HostExpr>(),
            Ok(HostExpr::DomainPort("example.com".to_string(), 5678)),
        );
    }
}
