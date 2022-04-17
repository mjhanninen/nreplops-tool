// host_expression/mod.rs
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

mod addr;
pub mod parser;
mod port_set;

use std::{net, str};

pub use port_set::{
    CannotConvertToPortSetError, Port, PortSet, PortSetParseError,
};

#[derive(Clone, Debug, PartialEq)]
pub struct HostExpr {
    host: Host,
    tunnel: Option<Tunnel>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Host {
    Local { ports: PortSet },
    RemoteDomain { domain: String, ports: PortSet },
    RemoteIP { addr: net::IpAddr, ports: PortSet },
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum Host_DEPRECATED {
    Local(u16),
    RemoteDomain(String, u16),
    RemoteIP(net::SocketAddr),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tunnel {
    user: Option<String>,
    host: String,
    port: Option<PortSet>,
}

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("parse error")]
pub struct ParseError;

impl str::FromStr for Host_DEPRECATED {
    type Err = HostOptionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(port) = s.parse::<u16>() {
            return Ok(Host_DEPRECATED::Local(port));
        }
        if let Ok(socket_addr) = s.parse::<net::SocketAddr>() {
            return Ok(Host_DEPRECATED::RemoteIP(socket_addr));
        }
        if let Some((host_part, port_part)) = s.rsplit_once(':') {
            if let Ok(port) = port_part.parse::<u16>() {
                if let Ok(domain) = ::addr::parse_domain_name(host_part) {
                    return Ok(Host_DEPRECATED::RemoteDomain(
                        domain.to_string(),
                        port,
                    ));
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
mod test {

    use super::*;
    use std::net;

    #[test]
    fn host_option_parsing() {
        assert_eq!(
            "5678".parse::<Host_DEPRECATED>(),
            Ok(Host_DEPRECATED::Local(5678)),
        );
        assert_eq!(
            "1.2.3.4:5678".parse::<Host_DEPRECATED>(),
            Ok(Host_DEPRECATED::RemoteIP(net::SocketAddr::from((
                [1, 2, 3, 4],
                5678
            )))),
        );
        assert_eq!(
            "localhost:5678".parse::<Host_DEPRECATED>(),
            Ok(Host_DEPRECATED::RemoteDomain("localhost".to_string(), 5678)),
        );
        assert_eq!(
            "example.com:5678".parse::<Host_DEPRECATED>(),
            Ok(Host_DEPRECATED::RemoteDomain(
                "example.com".to_string(),
                5678
            )),
        );
    }
}
