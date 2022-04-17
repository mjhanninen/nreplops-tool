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

use std::io::Write;
use std::{net, process};

use nreplops_tool::*;

fn main() {
    if let Err(e) = main1() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn main1() -> Result<(), anyhow::Error> {
    let args = cli::Args::from_command_line()?;

    let host = host_resolution::resolve_host_from_args(
        &args.host,
        &args.wait_port_file_for,
    )?;

    let sources =
        sources::load_sources(&args.source_args[..], &args.template_args[..])?;

    // Short-circuit if there is nothing to evaluate; effectively this happens
    // when the program is used only as a latch for the port file.
    if sources.is_empty() {
        return Ok(());
    }

    let outputs = outputs::Outputs::try_from_args(&args)?;

    let stream = net::TcpStream::connect(host)?;
    stream.set_nodelay(true)?;
    let mut con = nrepl::Connection::new(stream);

    con.send(&nrepl::WireRequest {
        op: nrepl::Op::Clone.to_string(),
        id: "1".into(),
        session: None,
        ns: None,
        code: None,
        line: None,
        column: None,
        file: None,
    })?;
    let first_resp = con.try_recv()?;
    let session = first_resp.new_session.unwrap();
    for (ix, input) in sources.into_iter().enumerate() {
        let id = format!("eval-{}", ix + 1);
        con.send(&nrepl::WireRequest {
            op: nrepl::Op::Eval.to_string(),
            id: id.clone(),
            session: Some(session.clone()),
            ns: None,
            code: Some(input.content),
            line: Some(1),
            column: Some(1),
            file: input.file,
        })?;
        loop {
            let resp = con.try_recv()?;
            if resp.id.as_deref().unwrap_or("") != id {
                continue;
            }
            if let Some(ref s) = resp.value {
                if let Some(ref w) = outputs.nrepl_results {
                    writeln!(w.writer(), "{}", s)?;
                }
            }
            if let Some(ref s) = resp.out {
                if let Some(ref w) = outputs.nrepl_stdout {
                    write!(w.writer(), "{}", s)?;
                }
            }
            if let Some(ref s) = resp.err {
                if let Some(ref w) = outputs.nrepl_stderr {
                    write!(w.writer(), "{}", s)?;
                }
            }
            if resp.has_status("done") {
                break;
            }
        }
    }
    Ok(())
}
