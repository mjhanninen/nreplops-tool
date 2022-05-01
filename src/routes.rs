use std::net;

use crate::{
    conn_expr::{Addr, ConnectionExpr, Port, PortSet},
    error::Error,
};

pub fn resolve_routes(conn_expr: &ConnectionExpr) -> Result<Routes, Error> {
    Ok(Routes {
        inner: RouteSet::try_from_conn_expr(conn_expr)?,
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
