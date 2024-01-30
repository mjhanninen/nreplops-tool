// main.rs
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

#![deny(
  future_incompatible,
  missing_debug_implementations,
  nonstandard_style,
  rust_2021_compatibility,
  unused
)]

use std::{cmp, io::Write, process};

use nreplops_tool::{self, error::Error, version, *};

fn main() {
  let args = cli::Args::from_command_line().unwrap_or_else(die);

  if let Some(ref required) = args.version_range {
    let current = version::crate_version();
    use cmp::Ordering::*;
    match current.cmp_to_range(required) {
      Less => die(Error::TooOldVersion(current, required.clone())),
      Greater => die(Error::TooNewVersion(current, required.clone())),
      Equal => (),
    }
  }

  let conn_expr = args.conn_expr_src.resolve_expr().unwrap_or_else(die);

  let host_opts_table =
    hosts_files::load_default_hosts_files().unwrap_or_else(die);
  let sources =
    sources::load_sources(&args.source_args[..], &args.template_args[..])
      .unwrap_or_else(die);

  // Short-circuit if there is nothing to evaluate; effectively this happens
  // when the program is used only as a latch for the port file.
  if sources.is_empty() {
    return;
  }

  let conn_routes =
    routes::resolve_routes(&conn_expr, &host_opts_table).unwrap_or_else(die);
  let outputs = outputs::Outputs::try_from_args(&args).unwrap_or_else(die);
  let socket = socket::connect(conn_routes).unwrap_or_else(die);
  let con = nrepl::Connection::new(socket);
  let mut session = con.session().unwrap_or_else(die);

  match eval_sources(&mut session, &sources, &outputs) {
    Ok(_) => {
      session.close().unwrap_or_else(die);
    }
    Err(Error::HostDisconnected) => die(Error::HostDisconnected),
    Err(err) => {
      eprintln!("Error: {:?}", err);
      session.close().unwrap_or_else(die);
      process::exit(1);
    }
  };
}

fn die<T>(e: Error) -> T {
  eprintln!("Error: {}", e);
  process::exit(1);
}

fn eval_sources(
  session: &mut nrepl::Session,
  sources: &[sources::Source],
  outputs: &outputs::Outputs,
) -> Result<(), Error> {
  for input in sources.iter() {
    session.eval(&input.content, None, Some(1), Some(1), |response| {
      if let Some(value) = response.value {
        if let Some(ref sink) = outputs.nrepl_results {
          sink.output(value)?;
        }
      }
      if let Some(ref s) = response.out {
        if let Some(ref output) = outputs.nrepl_stdout {
          write!(output.writer(), "{}", s)
            .map_err(|e| output.generate_error(e))?;
        }
      }
      if let Some(ref s) = response.err {
        if let Some(ref output) = outputs.nrepl_stderr {
          write!(output.writer(), "{}", s)
            .map_err(|e| output.generate_error(e))?;
        }
      }
      Ok(())
    })?;
  }

  Ok(())
}
