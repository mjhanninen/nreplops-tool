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

pub use self::addr::{
    Addr, ConversionError as AddrConversionError, ParseError as AddrParseError,
};
use parser::{HostExprLanguage, Pairs, Parser, Rule};
pub use port_set::{
    CannotConvertToPortSetError, Port, PortSet, PortSetParseError,
};

#[derive(Clone, Debug, PartialEq)]
pub struct HostExpr {
    ports: PortSet,
    addr: Option<Addr>,
    tunnel: Option<Tunnel>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tunnel {
    user: Option<String>,
    addr: Addr,
    ports: Option<PortSet>,
}

impl str::FromStr for HostExpr {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let top_pair = HostExprLanguage::parse(Rule::host_expr, s)
            .map_err(|_| ParseError)?
            .next()
            .expect("grammar guarantees host expression")
            .into_inner()
            .next()
            .expect("grammar guarantees inner specific host expression");
        match top_pair.as_rule() {
            Rule::local_port => host_expr_from_local_port_pair(top_pair.into_inner()),
            Rule::remote_port => host_expr_from_remote_port_pair(top_pair.into_inner()),
            Rule::tunneled_port => host_expr_from_tunneled_port_pair(top_pair.into_inner()),
            _ => unreachable!("grammar guarantees local, remote, or tunneled remote host expression"),
        }
    }
}

fn host_expr_from_local_port_pair(
    mut pairs: Pairs<Rule>,
) -> Result<HostExpr, ParseError> {
    Ok(HostExpr {
        ports: pairs
            .next()
            .expect("grammar guarantees a port set")
            .try_into()
            // grammar does not limit the port to u16
            .map_err(|_| ParseError)?,
        addr: None,
        tunnel: None,
    })
}

fn host_expr_from_remote_port_pair(
    mut pairs: Pairs<Rule>,
) -> Result<HostExpr, ParseError> {
    let addr = pairs
        .next()
        .expect("grammar guarantees an address")
        .try_into()
        .expect("grammar guarantees the address is legal");
    let mut host_expr = host_expr_from_local_port_pair(
        pairs
            .next()
            .expect("grammar guarantees a local port expression")
            .into_inner(),
    )?;
    host_expr.addr = Some(addr);
    Ok(host_expr)
}

fn host_expr_from_tunneled_port_pair(
    pairs: Pairs<Rule>,
) -> Result<HostExpr, ParseError> {
    let (tunnel, mut pairs) = tunnel_from_pairs(pairs)?;
    let mut host_expr = host_expr_from_remote_port_pair(
        pairs
            .next()
            .expect("grammar guarantees a remote port expression")
            .into_inner(),
    )?;
    host_expr.tunnel = Some(tunnel);
    Ok(host_expr)
}

fn tunnel_from_pairs(
    mut pairs: Pairs<Rule>,
) -> Result<(Tunnel, Pairs<Rule>), ParseError> {
    let mut next = pairs.next().expect("grammar guarantees a user or address");
    let user = if matches!(next.as_rule(), Rule::user) {
        let s = next.as_str().to_owned();
        next = pairs.next().expect("grammar guarantees an address");
        Some(s)
    } else {
        None
    };
    match next.as_rule() {
        Rule::addr => Ok((
            Tunnel {
                user,
                addr: next
                    .try_into()
                    .expect("grammar guarantess addr is legal"),
                ports: None,
            },
            pairs,
        )),
        Rule::addr_and_port => {
            let mut inner = next.into_inner();
            let addr = inner
                .next()
                .expect("addr by grammar")
                .try_into()
                .expect("correct by gramma");
            let ports = inner
                .next()
                .expect("port_set by grammar")
                .try_into()
                .map_err(|_| ParseError)?;
            Ok((
                Tunnel {
                    user,
                    addr,
                    ports: Some(ports),
                },
                pairs,
            ))
        }
        _ => unreachable!("grammar guarantees addr or addr_and_port"),
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum Host_DEPRECATED {
    Local(u16),
    RemoteDomain(String, u16),
    RemoteIP(net::SocketAddr),
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

    fn ps(ports: &[u16]) -> PortSet {
        PortSet(Vec::from_iter(ports.iter().cloned()))
    }

    fn maybe_ps(ports: &[u16]) -> Option<PortSet> {
        if ports.is_empty() {
            None
        } else {
            Some(ps(ports))
        }
    }

    fn ip4(a: u8, b: u8, c: u8, d: u8) -> Addr {
        Addr::IP(net::Ipv4Addr::new(a, b, c, d).into())
    }

    fn ip6(
        a: u16,
        b: u16,
        c: u16,
        d: u16,
        e: u16,
        f: u16,
        g: u16,
        h: u16,
    ) -> Addr {
        Addr::IP(net::Ipv6Addr::new(a, b, c, d, e, f, g, h).into())
    }

    fn dom(domain: &str) -> Addr {
        Addr::Domain(domain.to_owned())
    }

    #[test]
    fn local_port_expression_parsing() {
        let mk = |ports| {
            Ok(HostExpr {
                ports: ps(ports),
                addr: None,
                tunnel: None,
            })
        };
        assert_eq!("1".parse(), mk(&[1]));
        assert_eq!("1,2".parse(), mk(&[1, 2]));
        assert_eq!("1-3".parse(), mk(&[1, 2, 3]));
        assert_eq!("1,3,1-3".parse(), mk(&[1, 3, 2]));
    }

    #[test]
    fn remote_port_expression_parsing() {
        let mk = |addr, ports| {
            Ok(HostExpr {
                ports: ps(ports),
                addr: Some(addr),
                tunnel: None,
            })
        };
        assert_eq!("1.2.3.4:1,2-3".parse(), mk(ip4(1, 2, 3, 4), &[1, 2, 3]));
        assert_eq!(
            "[0:dead::beef:0]:1,2-3".parse(),
            mk(ip6(0, 0xDEAD, 0, 0, 0, 0, 0xBEEF, 0), &[1, 2, 3])
        );
        assert_eq!("localhost:1,2-3".parse(), mk(dom("localhost"), &[1, 2, 3]));
    }

    #[test]
    fn tunneled_port_expression_parsing() {
        let mk = |tunnel_user: Option<&str>,
                  tunnel_addr,
                  tunnel_ports: &[u16],
                  server_addr,
                  server_ports| {
            Ok(HostExpr {
                ports: ps(server_ports),
                addr: Some(server_addr),
                tunnel: Some(Tunnel {
                    user: tunnel_user.map(|s| s.to_owned()),
                    addr: tunnel_addr,
                    ports: maybe_ps(tunnel_ports),
                }),
            })
        };
        assert_eq!(
            "1.2.3.4:5.6.7.8:9".parse(),
            mk(None, ip4(1, 2, 3, 4), &[], ip4(5, 6, 7, 8), &[9])
        );
        assert_eq!(
            "1.2.3.4:5:6.7.8.9:10".parse(),
            mk(None, ip4(1, 2, 3, 4), &[5], ip4(6, 7, 8, 9), &[10])
        );
        assert_eq!(
            "1.2.3.4:5,6-7:8.9.10.11:12-14,15-16".parse(),
            mk(
                None,
                ip4(1, 2, 3, 4),
                &[5, 6, 7],
                ip4(8, 9, 10, 11),
                &[12, 13, 14, 15, 16]
            )
        );
        assert_eq!(
            "a@1.2.3.4:5.6.7.8:9".parse(),
            mk(Some("a"), ip4(1, 2, 3, 4), &[], ip4(5, 6, 7, 8), &[9])
        );
        assert_eq!(
            "a@1.2.3.4:5:6.7.8.9:10".parse(),
            mk(Some("a"), ip4(1, 2, 3, 4), &[5], ip4(6, 7, 8, 9), &[10])
        );
        assert_eq!(
            "[::]:[::]:1".parse(),
            mk(
                None,
                ip6(0, 0, 0, 0, 0, 0, 0, 0),
                &[],
                ip6(0, 0, 0, 0, 0, 0, 0, 0),
                &[1]
            )
        );
        assert_eq!(
            "[::]:1:[::]:1".parse(),
            mk(
                None,
                ip6(0, 0, 0, 0, 0, 0, 0, 0),
                &[1],
                ip6(0, 0, 0, 0, 0, 0, 0, 0),
                &[1]
            )
        );
        assert_eq!(
            "a@[1:2:3:4:5:6:7:8]:9:[10:11:12:13:14:15:16:17]:18".parse(),
            mk(
                Some("a"),
                ip6(0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08),
                &[9],
                ip6(0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17),
                &[18]
            )
        );
        assert_eq!(
            "a@b.c.d:e.f.g:1".parse(),
            mk(Some("a"), dom("b.c.d"), &[], dom("e.f.g"), &[1])
        );
        assert_eq!(
            "a@b.c.d.:1:e.f.g.:2".parse(),
            mk(Some("a"), dom("b.c.d."), &[1], dom("e.f.g."), &[2])
        );
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
