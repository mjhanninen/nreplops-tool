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

use std::str;

use super::{
  addr::Addr,
  parser::{ConnectionExprLanguage, Pairs, Parser, Rule},
  port_set::PortSet,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionExpr {
  RouteExpr(RouteExpr),
  HostKey(String),
}

impl ConnectionExpr {
  pub fn try_as_route_expr(&self) -> Option<&RouteExpr> {
    if let ConnectionExpr::RouteExpr(ref e) = *self {
      Some(e)
    } else {
      None
    }
  }
}

impl From<RouteExpr> for ConnectionExpr {
  fn from(route_expr: RouteExpr) -> Self {
    ConnectionExpr::RouteExpr(route_expr)
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RouteExpr {
  pub ports: PortSet,
  pub addr: Option<Addr>,
  pub tunnel: Option<TunnelExpr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TunnelExpr {
  pub user: Option<String>,
  pub addr: Addr,
  pub ports: Option<PortSet>,
}

impl str::FromStr for ConnectionExpr {
  type Err = ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let top_pair = ConnectionExprLanguage::parse(Rule::connection_expr, s)
      .map_err(|_| ParseError)?
      .next()
      .expect("grammar guarantees host expression")
      .into_inner()
      .next()
      .expect("grammar guarantees inner specific host expression");
    match top_pair.as_rule() {
      Rule::local_connection_expr => {
        connection_expr_from_local_connection_expr_pair(top_pair.into_inner())
          .map(|e| e.into())
      }
      Rule::remote_connection_expr => {
        connection_expr_from_remote_connection_expr_pair(top_pair.into_inner())
          .map(|e| e.into())
      }
      Rule::tunneled_connection_expr => {
        connection_expr_from_tunneled_connection_expr_pair(
          top_pair.into_inner(),
        )
        .map(|e| e.into())
      }
      Rule::host_key_expr => {
        Ok(ConnectionExpr::HostKey(top_pair.as_str().to_string()))
      }
      _ => unreachable!(
        r#"grammar guarantees a local, remote, or tunneled route
                   expression to a remote host, or a host key reference"#
      ),
    }
  }
}

fn connection_expr_from_local_connection_expr_pair(
  mut pairs: Pairs<Rule>,
) -> Result<RouteExpr, ParseError> {
  Ok(RouteExpr {
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
) -> Result<RouteExpr, ParseError> {
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
) -> Result<RouteExpr, ParseError> {
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
        addr: next.try_into().expect("grammar guarantess addr is legal"),
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
      Ok(ConnectionExpr::RouteExpr(RouteExpr {
        ports: ps(ports),
        addr: None,
        tunnel: None,
      }))
    };
    assert_eq!("1".parse(), mk(&[1]));
    assert_eq!("1,2".parse(), mk(&[1, 2]));
    assert_eq!("1-3".parse(), mk(&[1, 2, 3]));
    assert_eq!("1,3,1-3".parse(), mk(&[1, 3, 2]));
  }

  #[test]
  fn remote_connection_expr_parsing() {
    let mk = |addr, ports| {
      Ok(ConnectionExpr::RouteExpr(RouteExpr {
        ports: ps(ports),
        addr: Some(addr),
        tunnel: None,
      }))
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
      Ok(ConnectionExpr::RouteExpr(RouteExpr {
        ports: ps(server_ports),
        addr: Some(server_addr),
        tunnel: Some(TunnelExpr {
          user: tunnel_user.map(|s| s.to_owned()),
          addr: tunnel_addr,
          ports: maybe_ps(tunnel_ports),
        }),
      }))
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
  fn host_key_expr_parsing() {
    let mk = |key: &str| Ok(ConnectionExpr::HostKey(key.to_owned()));
    assert_eq!("x".parse(), mk("x"));
    assert_eq!("my_prod_host_1".parse(), mk("my_prod_host_1"));
  }
}
