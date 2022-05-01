// conn_expr/conn_expr.rs
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

use std::{net, str};

use crate::error::Error;

use super::{
    addr::Addr,
    parser::{HostExprLanguage, Pairs, Parser, Rule},
    port_set::{Port, PortSet},
};

#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionExpr {
    ports: PortSet,
    addr: Option<Addr>,
    tunnel: Option<TunnelExpr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TunnelExpr {
    user: Option<String>,
    addr: Addr,
    ports: Option<PortSet>,
}

impl ConnectionExpr {
    pub fn resolve_routes(&self) -> Result<Routes, Error> {
        Ok(Routes {
            inner: RouteSet::try_from_conn_expr(self)?,
            pos: 0,
        })
    }
}

#[derive(Clone, Debug)]
pub enum Route {
    Direct(net::SocketAddr),
    // Note that we let the ssh client to resolve the ssh server's address and,
    // likewise, the ssh server to resolve to final host's address.  This way
    // the name resolution behaves the same as it would when you debug it by
    // hand with the actual ssh client.
    Tunneled(TunnelOptions),
}

#[derive(Clone, Debug)]
pub struct TunnelOptions {
    pub ssh_user: Option<String>,
    pub ssh_addr: Addr,
    pub ssh_port: Option<Port>,
    pub host_addr: Addr,
    pub host_port: Port,
}

#[derive(Clone, Debug)]
pub struct Routes {
    inner: RouteSet,
    pos: usize,
}

impl Iterator for Routes {
    type Item = Route;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.inner.len() {
            let item = self.inner.produce(self.pos);
            self.pos += 1;
            Some(item)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
enum RouteSet {
    Direct {
        ips: Vec<net::IpAddr>,
        ports: PortSet,
    },
    Tunneled {
        ssh_user: Option<String>,
        ssh_addr: Addr,
        ssh_ports: Option<PortSet>,
        host_addr: Addr,
        host_ports: PortSet,
    },
}

impl RouteSet {
    fn try_from_conn_expr(conn_expr: &ConnectionExpr) -> Result<Self, Error> {
        if let Some(ref tunnel) = conn_expr.tunnel {
            let host_addr = conn_expr
                .addr
                .as_ref()
                .expect("tunneling should guarantee final host address")
                .clone();
            Ok(RouteSet::Tunneled {
                ssh_user: tunnel.user.clone(),
                ssh_addr: tunnel.addr.clone(),
                ssh_ports: tunnel.ports.clone(),
                host_addr,
                host_ports: conn_expr.ports.clone(),
            })
        } else {
            let mut ips = match conn_expr.addr {
                None => dns_lookup::lookup_host("localhost").map_err(|_| {
                    Error::DomainNotFound("localhost".to_owned())
                })?,
                Some(Addr::Domain(ref domain)) => {
                    dns_lookup::lookup_host(domain)
                        .map_err(|_| Error::DomainNotFound(domain.clone()))?
                }
                Some(Addr::IP(ip)) => vec![ip],
            };
            ips.sort();
            Ok(RouteSet::Direct {
                ips,
                ports: conn_expr.ports.clone(),
            })
        }
    }

    fn len(&self) -> usize {
        match self {
            RouteSet::Direct { ips: addrs, ports } => {
                addrs.len() * ports.as_slice().len()
            }
            RouteSet::Tunneled {
                ssh_ports: None,
                host_ports,
                ..
            } => host_ports.as_slice().len(),
            RouteSet::Tunneled {
                ssh_ports: Some(ssh_ports),
                host_ports,
                ..
            } => ssh_ports.as_slice().len() * host_ports.as_slice().len(),
        }
    }

    fn produce(&self, ix: usize) -> Route {
        assert!(ix < self.len());
        match self {
            RouteSet::Direct { ips: addrs, ports } => {
                // Iterate resolved addresses first and given ports second
                let ix_addr = ix % addrs.len();
                let ix_port = ix / addrs.len();
                Route::Direct(net::SocketAddr::new(
                    addrs[ix_addr],
                    ports.as_slice()[ix_port],
                ))
            }
            RouteSet::Tunneled {
                ssh_user,
                ssh_addr,
                ssh_ports: None,
                host_addr,
                host_ports,
            } => Route::Tunneled(TunnelOptions {
                ssh_user: ssh_user.clone(),
                ssh_addr: ssh_addr.clone(),
                ssh_port: None,
                host_addr: host_addr.clone(),
                host_port: host_ports.as_slice()[ix],
            }),
            RouteSet::Tunneled {
                ssh_user,
                ssh_addr,

                ssh_ports: Some(ssh_ports),
                host_addr,
                host_ports,
            } => {
                // Iterate ssh host's ports first and final host's ports second
                let ssh_ports = ssh_ports.as_slice();
                let ix_ssh_port = ix % ssh_ports.len();
                let ix_host_port = ix / ssh_ports.len();
                Route::Tunneled(TunnelOptions {
                    ssh_user: ssh_user.clone(),
                    ssh_addr: ssh_addr.clone(),
                    ssh_port: Some(ssh_ports[ix_ssh_port]),
                    host_addr: host_addr.clone(),
                    host_port: host_ports.as_slice()[ix_host_port],
                })
            }
        }
    }
}

impl str::FromStr for ConnectionExpr {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let top_pair = HostExprLanguage::parse(Rule::connection_expr, s)
            .map_err(|_| ParseError)?
            .next()
            .expect("grammar guarantees host expression")
            .into_inner()
            .next()
            .expect("grammar guarantees inner specific host expression");
        match top_pair.as_rule() {
            Rule::local_connection_expr => {
                connection_expr_from_local_connection_expr_pair(
                    top_pair.into_inner(),
                )
            }
            Rule::remote_connection_expr => {
                connection_expr_from_remote_connection_expr_pair(
                    top_pair.into_inner(),
                )
            }
            Rule::tunneled_connection_expr => {
                connection_expr_from_tunneled_connection_expr_pair(
                    top_pair.into_inner(),
                )
            }
            Rule::host_key_expr => {
                todo!("host key expression parsing");
                /*
                connection_expr_from_host_key_expr_pair(
                    top_pair.into_inner(),
                )
                */
            }
            _ => unreachable!(
                r#"grammar guarantees local, remote, or tunneled remote host \
                   expression"#
            ),
        }
    }
}

fn connection_expr_from_local_connection_expr_pair(
    mut pairs: Pairs<Rule>,
) -> Result<ConnectionExpr, ParseError> {
    Ok(ConnectionExpr {
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

fn connection_expr_from_remote_connection_expr_pair(
    mut pairs: Pairs<Rule>,
) -> Result<ConnectionExpr, ParseError> {
    let addr = pairs
        .next()
        .expect("grammar guarantees an address")
        .try_into()
        .expect("grammar guarantees the address is legal");
    let mut connection_expr = connection_expr_from_local_connection_expr_pair(
        pairs
            .next()
            .expect("grammar guarantees a local port expression")
            .into_inner(),
    )?;
    connection_expr.addr = Some(addr);
    Ok(connection_expr)
}

fn connection_expr_from_tunneled_connection_expr_pair(
    pairs: Pairs<Rule>,
) -> Result<ConnectionExpr, ParseError> {
    let (tunnel, mut pairs) = tunnel_from_pairs(pairs)?;
    let mut connection_expr = connection_expr_from_remote_connection_expr_pair(
        pairs
            .next()
            .expect("grammar guarantees a remote port expression")
            .into_inner(),
    )?;
    connection_expr.tunnel = Some(tunnel);
    Ok(connection_expr)
}

fn tunnel_from_pairs(
    mut pairs: Pairs<Rule>,
) -> Result<(TunnelExpr, Pairs<Rule>), ParseError> {
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
            TunnelExpr {
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
                TunnelExpr {
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

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("parse error")]
pub struct ParseError;

#[cfg(test)]
mod test {

    use super::*;
    use std::net;

    fn ps(ports: &[u16]) -> PortSet {
        maybe_ps(ports).unwrap()
    }

    fn maybe_ps(ports: &[u16]) -> Option<PortSet> {
        PortSet::try_from_iter(ports.iter().cloned())
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
    fn local_connection_expr_parsing() {
        let mk = |ports| {
            Ok(ConnectionExpr {
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
    fn remote_connection_expr_parsing() {
        let mk = |addr, ports| {
            Ok(ConnectionExpr {
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
    fn tunneled_connection_expr_parsing() {
        let mk = |tunnel_user: Option<&str>,
                  tunnel_addr,
                  tunnel_ports: &[u16],
                  server_addr,
                  server_ports| {
            Ok(ConnectionExpr {
                ports: ps(server_ports),
                addr: Some(server_addr),
                tunnel: Some(TunnelExpr {
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
}
