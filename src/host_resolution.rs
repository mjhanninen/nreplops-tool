// host_resolution.rs
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

use std::{fs, io, net, path};

use crate::{cli, error::Error};

pub fn resolve_host_from_args(
    host_arg: &cli::HostArg,
) -> Result<net::SocketAddr, Error> {
    use cli::HostArg::*;
    match host_arg {
        HostExpr(e) => resolve_from_host_expr(e),
        PortFile(Some(ref f)) => resolve_from_port_file(f),
        PortFile(None) => {
            if let Some(ref f) = find_port_file().map_err(|_| Error::Unknown)? {
                resolve_from_port_file(f)
            } else {
                Err(Error::NotSpecified)
            }
        }
    }
}

fn find_port_file() -> io::Result<Option<path::PathBuf>> {
    let current_dir = path::PathBuf::from(".").canonicalize()?;
    for dir in current_dir.ancestors() {
        let mut path = path::PathBuf::from(dir);
        path.push(".nrepl-port");
        if path.is_file() {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

fn resolve_from_host_expr(
    host_expr: &cli::HostExpr,
) -> Result<net::SocketAddr, Error> {
    use cli::HostExpr::*;
    match host_expr {
        Port(port) => resolve_from_domaint_and_port("localhost", *port),
        SocketAddr(addr) => Ok(*addr),
        DomainPort(domain, port) => {
            resolve_from_domaint_and_port(domain, *port)
        }
    }
}

fn resolve_from_domaint_and_port(
    domain: &str,
    port: u16,
) -> Result<net::SocketAddr, Error> {
    if let Ok(mut ips) = dns_lookup::lookup_host(domain) {
        // Prefer IPv4 addresses and 127.0.0.1, in particular.
        let ipv4_localhost = "127.0.0.1".parse::<net::IpAddr>().unwrap();
        ips.sort();
        let ip = ips
            .iter()
            .find(|ip| **ip == ipv4_localhost)
            .or_else(|| ips.first())
            .expect(
                r#"assumed that dns_loop::lookup_host would always \
                   result in non-empty IP list upon success"#,
            );
        Ok(net::SocketAddr::from((*ip, port)))
    } else {
        Err(Error::DomainNotFound(domain.to_owned()))
    }
}

fn resolve_from_port_file(
    path: impl AsRef<path::Path>,
) -> Result<net::SocketAddr, Error> {
    let path = path.as_ref();
    let host_expr = fs::read_to_string(path)
        .map_err(|_| Error::CannotReadFile(path.to_string_lossy().into()))?
        .trim()
        .parse::<cli::HostExpr>()
        .map_err(|_| {
            Error::CannotParsePortFile(path.to_string_lossy().into())
        })?;
    resolve_from_host_expr(&host_expr)
}
