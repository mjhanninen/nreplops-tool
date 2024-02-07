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

use std::{cmp::Ordering, io::Write, process};

use nreplops_tool::{
  self, cli, error::Error, hosts_files, nrepl, outputs, routes, socket,
  sources, version,
};

fn main() {
  let args = cli::Args::from_command_line().unwrap_or_else(die);

  // Check version requirement
  if let Some(ref r) = args.version_requirement {
    let v = version::crate_version();
    match v.range_cmp(r) {
      Ordering::Less => die(Error::TooOldVersion(v, r.clone())),
      Ordering::Greater => die(Error::TooNewVersion(v, r.clone())),
      Ordering::Equal => (),
    }
  }

  // Obtain connection expression; possibly waits for the port file to appear
  let conn_expr = args.conn_expr_src.resolve_expr().unwrap_or_else(die);

  // Short-circuit if there is nothing to evaluate; effectively this happens
  // only when the program is used as a latch for the port file
  if args.source_args.is_empty() {
    return;
  }

  // Load known hosts
  let host_opts_table =
    hosts_files::load_default_hosts_files().unwrap_or_else(die);

  // Load sources (also inject template arguments)
  let sources =
    sources::load_sources(&args.source_args[..], &args.template_args[..])
      .unwrap_or_else(die);

  // Resolve what outputs to collect and where to send them
  let outputs = outputs::Outputs::try_from_args(&args).unwrap_or_else(die);

  // Resolve the actual connection routes to try
  let conn_routes =
    routes::resolve_routes(&conn_expr, &host_opts_table).unwrap_or_else(die);

  // XXX(soija) In case we wanted switch to using async this would be the point
  //            where to transition from sync to async.

  // Connect and form nREPL session
  let socket = socket::connect(conn_routes).unwrap_or_else(die);
  let con = nrepl::Connection::new(socket);
  let mut session = con.session().unwrap_or_else(die);

  // Evaluate the sources within the session
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
