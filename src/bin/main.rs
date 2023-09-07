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

use std::{
    io::{self, Write},
    process,
};

use nreplops_tool::{self, error::Error, outputs::OutputTarget, *};

fn main() {
    if let Err(e) = main1() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn main1() -> Result<(), anyhow::Error> {
    let args = cli::Args::from_command_line()?;

    let conn_expr = args.conn_expr_src.resolve_expr()?;

    let host_opts_table = hosts_files::load_default_hosts_files()?;

    let sources =
        sources::load_sources(&args.source_args[..], &args.template_args[..])?;

    // Short-circuit if there is nothing to evaluate; effectively this happens
    // when the program is used only as a latch for the port file.
    if sources.is_empty() {
        return Ok(());
    }

    let conn_routes = routes::resolve_routes(&conn_expr, &host_opts_table)?;

    let outputs = outputs::Outputs::try_from_args(&args)?;

    let socket = socket::connect(conn_routes)?;

    let con = nrepl::Connection::new(socket);

    let mut session = con.session()?;

    fn xform_err(output: &outputs::Output, _: io::Error) -> Error {
        match output.target() {
            OutputTarget::StdOut => panic!("cannot write to stdout"),
            OutputTarget::StdErr => panic!("cannot write to stderr"),
            OutputTarget::File(path) => {
                Error::CannotWriteFile(path.to_string_lossy().to_string())
            }
        }
    }

    for input in sources.into_iter() {
        session.eval(&input.content, None, Some(1), Some(1), |response| {
            if let Some(ref s) = response.value {
                if let Some(ref output) = outputs.nrepl_results {
                    writeln!(output.writer(), "{}", s)
                        .map_err(|e| xform_err(output, e))?;
                }
            }
            if let Some(ref s) = response.out {
                if let Some(ref output) = outputs.nrepl_stdout {
                    write!(output.writer(), "{}", s)
                        .map_err(|e| xform_err(output, e))?;
                }
            }
            if let Some(ref s) = response.err {
                if let Some(ref output) = outputs.nrepl_stderr {
                    write!(output.writer(), "{}", s)
                        .map_err(|e| xform_err(output, e))?;
                }
            }
            Ok(())
        })?;
    }

    let _con = session.close()?;

    Ok(())
}
