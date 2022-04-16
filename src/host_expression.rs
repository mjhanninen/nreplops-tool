// host_expression.rs
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

use std::{
    iter::{Enumerate, Peekable},
    net, str,
};

#[derive(Clone, Debug, PartialEq)]
pub struct PortSet(Vec<u16>);

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("parse error")]
pub struct ParseError;

#[derive(Debug)]
enum PortExprToken<'a> {
    Number(&'a str),
    Separator,
    Range,
}

struct PortExprTokens<'a> {
    s: &'a str,
    i: Peekable<Enumerate<str::Chars<'a>>>,
}

impl<'a> PortExprTokens<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            s,
            i: s.chars().enumerate().peekable(),
        }
    }
}

impl<'a> Iterator for PortExprTokens<'a> {
    type Item = Result<PortExprToken<'a>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let head = self.i.peek()?;
        match *head {
            (start, '0'..='9') => {
                let s = loop {
                    self.i.next().expect("peek guarantees existence");
                    match self.i.peek() {
                        Some((_, '0'..='9')) => (),
                        Some((stop, _)) => break &self.s[start..*stop],
                        None => break &self.s[start..],
                    }
                };
                Some(Ok(PortExprToken::Number(s)))
            }
            (_, ',') => {
                self.i.next().expect("peek guarantees existence");
                Some(Ok(PortExprToken::Separator))
            }
            (_, '-') => {
                self.i.next().expect("peek guarantees existence");
                Some(Ok(PortExprToken::Range))
            }
            (_, _) => Some(Err(ParseError)),
        }
    }
}

fn parse_range_or_port(
    tokens: &mut Peekable<PortExprTokens>,
    ports: &mut Vec<u16>,
) -> Result<(), ParseError> {
    use PortExprToken::*;
    if let Some(Ok(Number(s))) = tokens.next() {
        let start = s.parse().map_err(|_| ParseError)?;
        let is_range = match tokens.peek() {
            Some(Ok(Range)) => true,
            _ => false,
        };
        if is_range {
            tokens
                .next()
                .expect("peek guarantees Some")
                .expect("peek guarantees Ok");
            if let Some(Ok(Number(s))) = tokens.next() {
                let stop = s.parse().map_err(|_| ParseError)?;
                if start <= stop {
                    for port in start..=stop {
                        if !ports.contains(&port) {
                            ports.push(port)
                        }
                    }
                } else {
                    for port in (stop..=start).rev() {
                        if !ports.contains(&port) {
                            ports.push(port)
                        }
                    }
                }
            } else {
                return Err(ParseError);
            }
        } else {
            if !ports.contains(&start) {
                ports.push(start)
            }
        }
    } else {
        return Err(ParseError);
    }
    Ok(())
}

impl str::FromStr for PortSet {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ports = vec![];
        let mut tokens = PortExprTokens::new(s).peekable();
        parse_range_or_port(&mut tokens, &mut ports)?;
        while let Some(token) = tokens.next() {
            if let Ok(PortExprToken::Separator) = token {
                parse_range_or_port(&mut tokens, &mut ports)?;
            } else {
                return Err(ParseError);
            }
        }
        Ok(Self(ports))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum HostExpr {
    Local(u16),
    RemoteDomain(String, u16),
    RemoteIP(net::SocketAddr),
}

impl str::FromStr for HostExpr {
    type Err = HostOptionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(port) = s.parse::<u16>() {
            return Ok(HostExpr::Local(port));
        }
        if let Ok(socket_addr) = s.parse::<net::SocketAddr>() {
            return Ok(HostExpr::RemoteIP(socket_addr));
        }
        if let Some((host_part, port_part)) = s.rsplit_once(':') {
            if let Ok(port) = port_part.parse::<u16>() {
                if let Ok(domain) = addr::parse_domain_name(host_part) {
                    return Ok(HostExpr::RemoteDomain(
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

use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "host_expression.pest"]
struct HostExprParser;

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
        assert_eq!("".parse::<PortSet>(), Err(ParseError));
        assert_eq!(" 1".parse::<PortSet>(), Err(ParseError));
        assert_eq!("1 ".parse::<PortSet>(), Err(ParseError));
        assert_eq!(",".parse::<PortSet>(), Err(ParseError));
        assert_eq!(",1".parse::<PortSet>(), Err(ParseError));
        assert_eq!("1,".parse::<PortSet>(), Err(ParseError));
        assert_eq!("-1".parse::<PortSet>(), Err(ParseError));
        assert_eq!("1-".parse::<PortSet>(), Err(ParseError));
        assert_eq!("1,,2".parse::<PortSet>(), Err(ParseError));
        assert_eq!("1--2".parse::<PortSet>(), Err(ParseError));
        assert_eq!("1-2-3".parse::<PortSet>(), Err(ParseError));
        assert_eq!("65536".parse::<PortSet>(), Err(ParseError));
    }

    #[test]
    fn host_option_parsing() {
        assert_eq!("5678".parse::<HostExpr>(), Ok(HostExpr::Local(5678)),);
        assert_eq!(
            "1.2.3.4:5678".parse::<HostExpr>(),
            Ok(HostExpr::RemoteIP(net::SocketAddr::from((
                [1, 2, 3, 4],
                5678
            )))),
        );
        assert_eq!(
            "localhost:5678".parse::<HostExpr>(),
            Ok(HostExpr::RemoteDomain("localhost".to_string(), 5678)),
        );
        assert_eq!(
            "example.com:5678".parse::<HostExpr>(),
            Ok(HostExpr::RemoteDomain("example.com".to_string(), 5678)),
        );
    }
}
