// routes.rs
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

use std::net;

use crate::{
    conn_expr::{Addr, ConnectionExpr, Port, PortSet, RouteExpr},
    error::Error,
    host_options::HostOptionsTable,
};

pub fn resolve_routes(
    conn_expr: &ConnectionExpr,
    host_opts_table: &HostOptionsTable,
) -> Result<Routes, Error> {
    use ConnectionExpr::*;
    let route_expr = match conn_expr {
        RouteExpr(ref e) => e,
        HostKey(ref k) => host_opts_table
            .get(k)
            .ok_or_else(|| Error::HostKeyNotFound(k.to_string()))?
            .conn_expr
            .try_as_route_expr()
            .ok_or_else(|| {
                Error::RecursiveHostKeysNotSupported(k.to_string())
            })?,
    };
    Ok(Routes {
        inner: RoutesInner::try_from_route_expr(route_expr)?,
        pos: 0,
    })
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
    inner: RoutesInner,
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
enum RoutesInner {
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

impl RoutesInner {
    fn try_from_route_expr(route_expr: &RouteExpr) -> Result<Self, Error> {
        if let Some(ref tunnel) = route_expr.tunnel {
            let host_addr = route_expr
                .addr
                .as_ref()
                .expect("tunneling should guarantee final host address")
                .clone();
            Ok(RoutesInner::Tunneled {
                ssh_user: tunnel.user.clone(),
                ssh_addr: tunnel.addr.clone(),
                ssh_ports: tunnel.ports.clone(),
                host_addr,
                host_ports: route_expr.ports.clone(),
            })
        } else {
            let mut ips = match route_expr.addr {
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
            Ok(RoutesInner::Direct {
                ips,
                ports: route_expr.ports.clone(),
            })
        }
    }

    fn len(&self) -> usize {
        match self {
            RoutesInner::Direct { ips: addrs, ports } => {
                addrs.len() * ports.as_slice().len()
            }
            RoutesInner::Tunneled {
                ssh_ports: None,
                host_ports,
                ..
            } => host_ports.as_slice().len(),
            RoutesInner::Tunneled {
                ssh_ports: Some(ssh_ports),
                host_ports,
                ..
            } => ssh_ports.as_slice().len() * host_ports.as_slice().len(),
        }
    }

    fn produce(&self, ix: usize) -> Route {
        assert!(ix < self.len());
        match self {
            RoutesInner::Direct { ips: addrs, ports } => {
                // Iterate resolved addresses first and given ports second
                let ix_addr = ix % addrs.len();
                let ix_port = ix / addrs.len();
                Route::Direct(net::SocketAddr::new(
                    addrs[ix_addr],
                    ports.as_slice()[ix_port],
                ))
            }
            RoutesInner::Tunneled {
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
            RoutesInner::Tunneled {
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

// FIXME: Tests
