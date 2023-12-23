// error.rs
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

use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("No input")]
  NoInput,
  #[error("file {0} not found")]
  NotFound(String),
  #[error("cannot read stdin")]
  CannotReadStdIn,
  #[error("cannot read file {0}")]
  CannotReadFile(String),
  #[error("cannot write file {0}")]
  CannotWriteFile(String),
  #[error("bad port file {0}")]
  CannotParsePortFile(String),
  #[error("cannot resolve the IP address for the domain {0}")]
  DomainNotFound(String),
  #[error(
    "the nREPL server address not specified; try using the --port or \
    --port-file option or ensure that there is a port file .nrepl-port \
    in the current working directory or its ancestors"
  )]
  NotSpecified,
  #[error("unknown error")]
  Unknown,
  #[error("stdin conflict")]
  StdInConflict,
  #[error("bad stdin")]
  BadStdIn,
  #[error("bad source file")]
  BadSourceFile,
  #[error("template arguments must be utf-8")]
  NonUtf8TemplateArgument,
  #[error("non-positional template argument must be named")]
  UnnamedNonPositionalTemplateArgument,
  #[error("timeout while waiting for port file")]
  PortFileTimeout,
  #[error("cannot find host definition for key \"{0}\"")]
  HostKeyNotFound(String),
  #[error(
    "host key \"{0}\" refers recursively to another host key but this is \
    not supported yet"
  )]
  RecursiveHostKeysNotSupported(String),
  #[error(
    "unexpected error while loading for default host configuration: {0}"
  )]
  FailedToLoadDefaultHostConfig(io::Error),

  // Related to nREPL connection
  #[error("failed to connect to host: {0}")]
  FailedToConnectToHost(io::Error),
  #[error("unexpected error while receiving from host: {0}")]
  CannotReceiveFromHost(io::Error),
  #[error("unexpected error while sending to host: {0}")]
  CannotSendToHost(io::Error),
  #[error("host sent corrupted response")]
  CorruptedResponse,
  #[error("host disconnected unexpectedly")]
  HostDisconnected,
  #[error("host sent unexptected response")]
  UnexptectedResponse,
}
