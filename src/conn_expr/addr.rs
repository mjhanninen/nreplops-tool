// conn_expr/addr.rs
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

use std::{fmt, net, str};

use super::parser::{HostExprLanguage, Pair, Parser, Rule};

#[derive(Clone, Debug, PartialEq)]
pub enum Addr {
    Domain(String),
    IP(net::IpAddr),
}

impl fmt::Display for Addr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Addr::Domain(domain) => domain.fmt(fmt),
            Addr::IP(ip) => ip.fmt(fmt),
        }
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for Addr {
    type Error = ConversionError;

    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        if matches!(pair.as_rule(), Rule::addr) {
            let addr = pair
                .into_inner()
                .next()
                .expect("grammar guarantees specific inner address");
            Ok(match addr.as_rule() {
                Rule::ipv4_addr => Addr::IP(
                    addr.as_str()
                        .parse::<net::Ipv4Addr>()
                        .expect("grammar guarantees legal IPv4 address")
                        .into(),
                ),
                Rule::ipv6_addr => Addr::IP(
                    addr.as_str()
                        .parse::<net::Ipv6Addr>()
                        .expect("grammar guarantees legal IPv6 address")
                        .into(),
                ),
                Rule::domain_addr => {
                    let domain = addr.as_str().to_owned();
                    if addr.as_str().len() > 253
                        || addr.into_inner().any(|l| l.as_str().len() > 63)
                    {
                        return Err(ConversionError);
                    }
                    Addr::Domain(domain)
                }
                _ => unreachable!(
                    "grammar guarantees IPv6, IPv4, or domain address"
                ),
            })
        } else {
            Err(ConversionError)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
#[error("failed to convert into address")]
pub struct ConversionError;

impl str::FromStr for Addr {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        HostExprLanguage::parse(Rule::addr_expr, s)
            .map_err(|_| ParseError)?
            .next()
            .expect("grammar guaranteed addr_expr")
            .into_inner()
            .next()
            .expect("grammar guarantees addr")
            .try_into()
            .map_err(|_| ParseError)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
#[error("failed to parse address")]
pub struct ParseError;

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn parse_domain_name() {
        let d = |s: &str| Ok(Addr::Domain(s.to_string()));
        assert_eq!("localhost".parse(), d("localhost"));
        assert_eq!("localhost.".parse(), d("localhost."));
        assert_eq!("a.b.c.d".parse(), d("a.b.c.d"));
        assert_eq!("a1.b-2.c--3.d---4".parse(), d("a1.b-2.c--3.d---4"));
    }

    #[test]
    fn parse_ipv4_address() {
        let ip4 =
            |a, b, c, d| Ok(Addr::IP(net::Ipv4Addr::new(a, b, c, d).into()));
        assert_eq!("0.0.0.0".parse(), ip4(0, 0, 0, 0));
        assert_eq!("1.2.3.4".parse(), ip4(1, 2, 3, 4));
        assert_eq!("255.255.255.255".parse(), ip4(255, 255, 255, 255));
    }

    #[test]
    fn parse_ipv6_address() {
        let ip6 = |a, b, c, d, e, f, g, h| {
            Ok(Addr::IP(net::Ipv6Addr::new(a, b, c, d, e, f, g, h).into()))
        };
        assert_eq!("[::]".parse(), ip6(0, 0, 0, 0, 0, 0, 0, 0));
        assert_eq!("[::1]".parse(), ip6(0, 0, 0, 0, 0, 0, 0, 1));
        assert_eq!("[1::]".parse(), ip6(1, 0, 0, 0, 0, 0, 0, 0));
        assert_eq!("[::0.0.0.0]".parse(), ip6(0, 0, 0, 0, 0, 0, 0, 0));
        assert_eq!(
            "[DEAD::BEEF]".parse(),
            ip6(0xDEAD, 0, 0, 0, 0, 0, 0, 0xBEEF)
        );
        assert_eq!(
            "[dead::beef]".parse(),
            ip6(0xDEAD, 0, 0, 0, 0, 0, 0, 0xBEEF)
        );
        assert_eq!(
            "[1:23:456:789a::127.0.0.1]".parse(),
            ip6(0x0001, 0x0023, 0x0456, 0x789A, 0, 0, 0x7F00, 1),
        );
    }

    #[test]
    fn parse_bad_address() {
        let err = Err(ParseError);
        assert_eq!(" 1.2.3.4".parse::<Addr>(), err);
        assert_eq!("".parse::<Addr>(), err);
        assert_eq!(".".parse::<Addr>(), err);
        assert_eq!("01.2.3.4".parse::<Addr>(), err);
        assert_eq!("1. 2.3.4".parse::<Addr>(), err);
        assert_eq!("1.2.3.04".parse::<Addr>(), err);
        assert_eq!("1.2.3.256".parse::<Addr>(), err);
        assert_eq!("1.2.3.4 ".parse::<Addr>(), err);
        assert_eq!("1.2.3.4.com".parse::<Addr>(), err);
        assert_eq!("1.com".parse::<Addr>(), err);
        assert_eq!("[ ::]".parse::<Addr>(), err);
        assert_eq!("[:: ]".parse::<Addr>(), err);
        assert_eq!("[]".parse::<Addr>(), err);
        assert_eq!("dash-.com".parse::<Addr>(), err);
        assert_eq!("tyre.8bar.com".parse::<Addr>(), err);
        // XXX(soija) These are actually legal IP addresses but ¯\_(ツ)_/¯
        assert_eq!("1".parse(), err);
        assert_eq!("1.2".parse(), err);
        assert_eq!("1.2.3".parse(), err);
    }
}
