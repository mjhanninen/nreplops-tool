// version.rs
// Copyright 2024 Matti Hänninen
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

use std::{cmp, env, fmt, str};

pub fn crate_version() -> Version {
  let major = env!("CARGO_PKG_VERSION_MAJOR").parse::<u16>().unwrap();
  let minor = env!("CARGO_PKG_VERSION_MINOR").parse::<u16>().unwrap();
  let patch = env!("CARGO_PKG_VERSION_PATCH").parse::<u16>().unwrap();
  Version::MajorMinorPatch(major, minor, patch)
}

#[derive(Clone, Debug)]
pub enum Version {
  Major(u16),
  MajorMinor(u16, u16),
  MajorMinorPatch(u16, u16, u16),
}

impl Version {
  fn to_triplet(&self) -> (u16, u16, u16) {
    match self {
      Version::Major(a) => (*a, 0, 0),
      Version::MajorMinor(a, b) => (*a, *b, 0),
      Version::MajorMinorPatch(a, b, c) => (*a, *b, *c),
    }
  }

  pub fn next_breaking(&self) -> Self {
    use Version::*;
    match self {
      Major(0) => MajorMinorPatch(0, 1, 0),
      Major(a) => MajorMinorPatch(a + 1, 0, 0),
      MajorMinor(0, b) => MajorMinorPatch(0, b + 1, 0),
      MajorMinor(a, _) => MajorMinorPatch(a + 1, 0, 0),
      MajorMinorPatch(0, b, _) => MajorMinorPatch(0, b + 1, 0),
      MajorMinorPatch(a, _, _) => MajorMinorPatch(a + 1, 0, 0),
    }
  }

  pub fn cmp_to_range(&self, range: &VersionRange) -> cmp::Ordering {
    use cmp::Ordering::*;
    if let Some(ref start) = range.start {
      if *self < *start {
        return Less;
      }
    }
    if let Some(ref end) = range.end {
      if *end < *self || (!range.inclusive && *end == *self) {
        return Greater;
      }
    }
    Equal
  }
}

impl cmp::PartialEq for Version {
  fn eq(&self, rhs: &Self) -> bool {
    self.to_triplet() == rhs.to_triplet()
  }
}

impl cmp::PartialOrd for Version {
  fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
    self.to_triplet().partial_cmp(&rhs.to_triplet())
  }
}

impl str::FromStr for Version {
  type Err = ParseVersionError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut it = s.split('.');
    let Some(major_str) = it.next() else {
      return Err(ParseVersionError)
    };
    let major = major_str.parse::<u16>().map_err(|_| ParseVersionError)?;
    let Some(minor_str) = it.next() else {
      return Ok(Version::Major(major));
    };
    let minor = minor_str.parse::<u16>().map_err(|_| ParseVersionError)?;
    let Some(patch_str) = it.next() else {
      return Ok(Version::MajorMinor(major, minor))
    };
    let patch = patch_str.parse::<u16>().map_err(|_| ParseVersionError)?;
    if it.next().is_none() {
      Ok(Version::MajorMinorPatch(major, minor, patch))
    } else {
      Err(ParseVersionError)
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParseVersionError;

impl fmt::Display for Version {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use Version::*;
    match self {
      Major(a) => write!(f, "{}.0.0", a),
      MajorMinor(a, b) => write!(f, "{}.{}.0", a, b),
      MajorMinorPatch(a, b, c) => write!(f, "{}.{}.{}", a, b, c),
    }
  }
}

#[derive(Clone, Debug)]
pub struct VersionRange {
  pub start: Option<Version>,
  pub end: Option<Version>,
  pub inclusive: bool,
}

impl VersionRange {
  pub fn non_breaking_from(start: &Version) -> Self {
    Self {
      start: Some(start.clone()),
      end: Some(start.next_breaking()),
      inclusive: false,
    }
  }
}

impl str::FromStr for VersionRange {
  type Err = ParseVersionRangeError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    fn parse_version(
      s: &str,
    ) -> Result<Option<Version>, ParseVersionRangeError> {
      match s.parse() {
        Ok(v) => Ok(Some(v)),
        Err(_) => Err(ParseVersionRangeError),
      }
    }

    fn parse_opt_version(
      s: &str,
    ) -> Result<Option<Version>, ParseVersionRangeError> {
      if s.is_empty() {
        Ok(None)
      } else {
        parse_version(s)
      }
    }

    if let Some((start_str, end_str)) = s.split_once("..=") {
      let start = parse_opt_version(start_str)?;
      let end = parse_version(end_str)?;
      if start.is_some() && end.is_some() && start > end {
        Err(ParseVersionRangeError)
      } else {
        Ok(Self {
          start,
          end,
          inclusive: true,
        })
      }
    } else if let Some((start_str, end_str)) = s.split_once("..") {
      let start = parse_opt_version(start_str)?;
      let end = parse_opt_version(end_str)?;
      if start.is_some() && end.is_some() && start >= end {
        Err(ParseVersionRangeError)
      } else {
        Ok(Self {
          start,
          end,
          inclusive: false,
        })
      }
    } else {
      Err(ParseVersionRangeError)
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParseVersionRangeError;

impl fmt::Display for VersionRange {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if let Some(ref v) = self.start {
      write!(f, "{}", v)?;
    }
    write!(f, "{}", if self.inclusive { "..=" } else { ".." })?;
    if let Some(ref v) = self.end {
      write!(f, "{}", v)?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod test {

  use super::*;

  #[test]
  fn version_equivalence() {
    use Version::*;

    assert_eq!(Major(1), Major(1));
    assert_eq!(Major(1), MajorMinor(1, 0));
    assert_eq!(Major(1), MajorMinorPatch(1, 0, 0));

    assert_eq!(MajorMinor(1, 2), MajorMinor(1, 2));
    assert_eq!(MajorMinor(1, 2), MajorMinorPatch(1, 2, 0));

    assert_eq!(MajorMinorPatch(1, 2, 3), MajorMinorPatch(1, 2, 3));
  }

  #[test]
  fn version_ordering() {
    use Version::*;

    assert!(Major(1) < Major(2));
    assert!(Major(2) > Major(1));

    assert!(MajorMinor(1, 2) < MajorMinor(1, 3));
    assert!(MajorMinor(1, 3) > MajorMinor(1, 2));

    assert!(MajorMinor(1, 2) < MajorMinor(2, 1));
    assert!(MajorMinor(2, 1) > MajorMinor(1, 2));

    assert!(Major(1) < MajorMinor(1, 2));
    assert!(MajorMinor(1, 2) < Major(2));

    assert!(MajorMinor(1, 2) > Major(1));
    assert!(Major(2) > MajorMinor(1, 2));

    assert!(MajorMinorPatch(1, 2, 3) < MajorMinorPatch(1, 2, 4));
    assert!(MajorMinorPatch(1, 2, 4) > MajorMinorPatch(1, 2, 3));

    assert!(MajorMinor(1, 2) < MajorMinorPatch(1, 2, 3));
    assert!(Major(1) < MajorMinorPatch(1, 2, 3));
    assert!(MajorMinorPatch(1, 2, 3) < MajorMinor(1, 3));
    assert!(MajorMinorPatch(1, 2, 3) < Major(2));

    assert!(MajorMinorPatch(1, 2, 3) > MajorMinor(1, 2));
    assert!(MajorMinorPatch(1, 2, 3) > Major(1));
    assert!(MajorMinor(1, 3) > MajorMinorPatch(1, 2, 3));
    assert!(Major(2) > MajorMinorPatch(1, 2, 3));
  }

  #[test]
  fn parse_good_version_strings() {
    use Version::*;

    assert!(match "1".parse::<Version>() {
      Ok(Major(1)) => true,
      _ => false,
    });
    assert!(match "1.2".parse::<Version>() {
      Ok(MajorMinor(1, 2)) => true,
      _ => false,
    });
    assert!(match "1.2.3".parse::<Version>() {
      Ok(MajorMinorPatch(1, 2, 3)) => true,
      _ => false,
    });
    assert!(match "123.456.789".parse::<Version>() {
      Ok(MajorMinorPatch(123, 456, 789)) => true,
      _ => false,
    });
    assert!(match "0.0.0".parse::<Version>() {
      Ok(MajorMinorPatch(0, 0, 0)) => true,
      _ => false,
    });
  }

  #[test]
  fn try_parsing_bad_version_strings() {
    assert_eq!("".parse::<Version>(), Err(ParseVersionError));
    assert_eq!("1 ".parse::<Version>(), Err(ParseVersionError));
    assert_eq!(" 1".parse::<Version>(), Err(ParseVersionError));
    assert_eq!(" ".parse::<Version>(), Err(ParseVersionError));
    assert_eq!("1.".parse::<Version>(), Err(ParseVersionError));
    assert_eq!("1.".parse::<Version>(), Err(ParseVersionError));
    assert_eq!("1.2.".parse::<Version>(), Err(ParseVersionError));
    assert_eq!("1.2.3.".parse::<Version>(), Err(ParseVersionError));
    assert_eq!("1.2.3.4".parse::<Version>(), Err(ParseVersionError));
    assert_eq!("whatever".parse::<Version>(), Err(ParseVersionError));
  }

  #[test]
  fn display_version_string() {
    use Version::*;

    assert_eq!(format!("{}", Major(123)), "123.0.0");
    assert_eq!(format!("{}", MajorMinor(123, 456)), "123.456.0");
    assert_eq!(format!("{}", MajorMinorPatch(123, 456, 789)), "123.456.789");
  }

  #[test]
  fn parse_good_version_ranges() {
    use Version::*;

    assert!(match "1..=1".parse() {
      Ok(VersionRange {
        start: Some(Major(1)),
        end: Some(Major(1)),
        inclusive: true,
      }) => true,
      bad => panic!("bad parse: {:#?}", bad),
    });

    assert!(match "1..=1.0.0".parse() {
      Ok(VersionRange {
        start: Some(Major(1)),
        end: Some(MajorMinorPatch(1, 0, 0)),
        inclusive: true,
      }) => true,
      bad => panic!("bad parse: {:#?}", bad),
    });

    assert!(match "1..1.0.1".parse() {
      Ok(VersionRange {
        start: Some(Major(1)),
        end: Some(MajorMinorPatch(1, 0, 1)),
        inclusive: false,
      }) => true,
      bad => panic!("bad parse: {:#?}", bad),
    });

    assert!(match "1..".parse() {
      Ok(VersionRange {
        start: Some(Major(1)),
        end: None,
        inclusive: false,
      }) => true,
      bad => panic!("bad parse: {:#?}", bad),
    });

    assert!(match "..1".parse() {
      Ok(VersionRange {
        start: None,
        end: Some(Major(1)),
        inclusive: false,
      }) => true,
      bad => panic!("bad parse: {:#?}", bad),
    });

    assert!(match "..=1".parse() {
      Ok(VersionRange {
        start: None,
        end: Some(Major(1)),
        inclusive: true,
      }) => true,
      bad => panic!("bad parse: {:#?}", bad),
    });

    assert!(match "..".parse() {
      Ok(VersionRange {
        start: None,
        end: None,
        inclusive: false,
      }) => true,
      bad => panic!("bad parse: {:#?}", bad),
    });
  }

  #[test]
  fn try_parsing_bad_version_ranges() {
    assert_eq!(
      "1..1".parse::<VersionRange>().unwrap_err(),
      ParseVersionRangeError
    );
    assert_eq!(
      "1.2.3..1.2.2".parse::<VersionRange>().unwrap_err(),
      ParseVersionRangeError
    );
    assert_eq!(
      "1.2.3..=1.2.2".parse::<VersionRange>().unwrap_err(),
      ParseVersionRangeError
    );
    assert_eq!(
      "..=".parse::<VersionRange>().unwrap_err(),
      ParseVersionRangeError
    );
  }

  #[test]
  fn compare_version_against_range() {
    use cmp::Ordering::*;

    let point: VersionRange = "1.2.3..=1.2.3".parse().unwrap();

    assert_eq!(
      "1.2.2".parse::<Version>().unwrap().cmp_to_range(&point),
      Less
    );
    assert_eq!(
      "1.2.3".parse::<Version>().unwrap().cmp_to_range(&point),
      Equal
    );
    assert_eq!(
      "1.2.4".parse::<Version>().unwrap().cmp_to_range(&point),
      Greater
    );

    let open: VersionRange = "1..2".parse().unwrap();

    assert_eq!(
      "0.65535.65535"
        .parse::<Version>()
        .unwrap()
        .cmp_to_range(&open),
      Less
    );
    assert_eq!(
      "1.0.0".parse::<Version>().unwrap().cmp_to_range(&open),
      Equal
    );
    assert_eq!(
      "1.65535.65535"
        .parse::<Version>()
        .unwrap()
        .cmp_to_range(&open),
      Equal
    );
    assert_eq!(
      "2.0.0".parse::<Version>().unwrap().cmp_to_range(&open),
      Greater
    );
    assert_eq!(
      "2.0.1".parse::<Version>().unwrap().cmp_to_range(&open),
      Greater
    );

    let closed: VersionRange = "1..=2".parse().unwrap();

    assert_eq!(
      "0.65535.65535"
        .parse::<Version>()
        .unwrap()
        .cmp_to_range(&closed),
      Less
    );
    assert_eq!(
      "1.0.0".parse::<Version>().unwrap().cmp_to_range(&closed),
      Equal
    );
    assert_eq!(
      "1.65535.65535"
        .parse::<Version>()
        .unwrap()
        .cmp_to_range(&closed),
      Equal
    );
    assert_eq!(
      "2.0.0".parse::<Version>().unwrap().cmp_to_range(&closed),
      Equal
    );
    assert_eq!(
      "2.0.1".parse::<Version>().unwrap().cmp_to_range(&closed),
      Greater
    );
  }
}
