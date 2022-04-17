// host_expression/port_set.rs
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

use super::parser::{self, Parser};

pub type Port = u16;

#[derive(Clone, Debug, PartialEq)]
pub struct PortSet(pub Vec<u16>);

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
#[error("cannot convert to port set")]
pub struct CannotConvertToPortSetError;

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
#[error("cannot parse port set expression")]
pub struct PortSetParseError;

#[cfg(test)]
mod test {
    use super::*;

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
}
