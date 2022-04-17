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

pub mod parser;

use std::{net, str};

use parser::Parser;

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

#[derive(Clone, Debug, PartialEq)]
pub struct PortSet(Vec<u16>);

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("cannot convert to port set")]
pub struct CannotConvertToPortSetError;

impl<'a> TryFrom<parser::Pair<'a, parser::Rule>> for PortSet {
    type Error = CannotConvertToPortSetError;

    fn try_from(
        pair: parser::Pair<'a, parser::Rule>,
    ) -> Result<Self, Self::Error> {
        use parser::Rule;
        if matches!(pair.as_rule(), Rule::port_set) {
            let mut ports = vec![];
            for p in pair.into_inner() {
                match p.as_rule() {
                    Rule::port => {
                        let port = p
                            .as_str()
                            .parse()
                            .map_err(|_| CannotConvertToPortSetError)?;
                        if !ports.contains(&port) {
                            ports.push(port)
                        }
                    }
                    Rule::port_range => {
                        let mut limits = p.into_inner();
                        let start = limits
                            .next()
                            .expect("grammar guarantees start port")
                            .as_str()
                            .parse()
                            .map_err(|_| CannotConvertToPortSetError)?;
                        let end = limits
                            .next()
                            .expect("grammar guarantees end port")
                            .as_str()
                            .parse()
                            .map_err(|_| CannotConvertToPortSetError)?;
                        if start <= end {
                            for port in start..=end {
                                if !ports.contains(&port) {
                                    ports.push(port)
                                }
                            }
                        } else {
                            for port in (end..=start).rev() {
                                if !ports.contains(&port) {
                                    ports.push(port)
                                }
                            }
                        }
                    }
                    _ => unreachable!("grammar guarantees port or port_range"),
                }
            }
            Ok(Self(ports))
        } else {
            Err(CannotConvertToPortSetError)
        }
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("cannot parse port set expression")]
pub struct PortSetParseError;

impl str::FromStr for PortSet {
    type Err = PortSetParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::HostExprLanguage::parse(parser::Rule::port_set_expr, s)
            .map_err(|_| PortSetParseError)?
            .next()
            .expect("grammar guaranteed post_set_expr")
            .into_inner()
            .next()
            .expect("grammar guarantees post_set")
            .try_into()
            .map_err(|_| PortSetParseError)
    }
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
                if let Ok(domain) = addr::parse_domain_name(host_part) {
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
    fn port_set_parsing() {
        assert_eq!("1".parse(), Ok(PortSet(vec![1])));
        assert_eq!("65535".parse(), Ok(PortSet(vec![65535])));
        assert_eq!("1,2".parse(), Ok(PortSet(vec![1, 2])));
        assert_eq!("1-3".parse(), Ok(PortSet(vec![1, 2, 3])));
        assert_eq!("3-1".parse(), Ok(PortSet(vec![3, 2, 1])));
        assert_eq!("1,1-2,5,2-4".parse(), Ok(PortSet(vec![1, 2, 5, 3, 4])));
        assert_eq!("".parse::<PortSet>(), Err(PortSetParseError));
        assert_eq!(" 1".parse::<PortSet>(), Err(PortSetParseError));
        assert_eq!("1 ".parse::<PortSet>(), Err(PortSetParseError));
        assert_eq!(",".parse::<PortSet>(), Err(PortSetParseError));
        assert_eq!(",1".parse::<PortSet>(), Err(PortSetParseError));
        assert_eq!("1,".parse::<PortSet>(), Err(PortSetParseError));
        assert_eq!("-1".parse::<PortSet>(), Err(PortSetParseError));
        assert_eq!("1-".parse::<PortSet>(), Err(PortSetParseError));
        assert_eq!("1,,2".parse::<PortSet>(), Err(PortSetParseError));
        assert_eq!("1--2".parse::<PortSet>(), Err(PortSetParseError));
        assert_eq!("1-2-3".parse::<PortSet>(), Err(PortSetParseError));
        assert_eq!("65536".parse::<PortSet>(), Err(PortSetParseError));
    }

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
