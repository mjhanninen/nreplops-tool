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

use std::{cmp::Ordering, env, fmt, str};

use VersionBlock as B;
use VersionExpr as E;

/// A trait for version ranges, possibly unbounded on either side.
pub trait VersionRange {
  /// The smallest version of the range or `None`, if the range is unbounded
  /// from below.
  fn start(&self) -> Option<Version>;

  /// The smallest version greater than the range or `None`, if the range is
  /// unbounded from above.
  fn stop(&self) -> Option<Version>;

  /// Checks whether the range is sensible, i.e. it doesn't stop before it
  /// starts.  However, ranges of zero size are permitted.
  fn sound(&self) -> bool {
    if let (Some(start), Some(stop)) = (self.start(), self.stop()) {
      start <= stop
    } else {
      true
    }
  }
}

pub fn crate_version() -> Version {
  let x = env!("CARGO_PKG_VERSION_MAJOR")
    .parse::<u16>()
    .expect("bad CARGO_PKG_VERSION_MAJOR");
  let y = env!("CARGO_PKG_VERSION_MINOR")
    .parse::<u16>()
    .expect("bad CARGO_PKG_VERSION_MINOR");
  let z = env!("CARGO_PKG_VERSION_PATCH")
    .parse::<u16>()
    .expect("bad CARGO_PKG_VERSION_PATCH");
  Version(x, y, z)
}

/// Represents a point version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u16, u16, u16);

impl Version {
  /// Returns the next version block that can contain breaking changes with
  /// respect to this point version.
  fn next_break(&self) -> VersionBlock {
    if self.0 == 0 {
      B::Minor(self.0, self.1 + 1)
    } else {
      B::Major(self.0 + 1)
    }
  }

  /// Compares the version against the given version range.
  pub fn range_cmp<R>(&self, range: &R) -> Ordering
  where
    R: VersionRange,
  {
    if range.start().map(|x| *self < x).unwrap_or(false) {
      Ordering::Less
    } else if range.stop().map(|x| x <= *self).unwrap_or(false) {
      Ordering::Greater
    } else {
      Ordering::Equal
    }
  }
}

impl VersionRange for Version {
  fn start(&self) -> Option<Version> {
    Some(*self)
  }

  fn stop(&self) -> Option<Version> {
    Some(Version(
      self.0,
      self.1,
      self.2.checked_add(1).expect("too big patch version"),
    ))
  }
}

impl fmt::Display for Version {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}.{}.{}", self.0, self.1, self.2)
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseVersionError;

impl str::FromStr for Version {
  type Err = ParseVersionError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut it = s.split('.');
    let mut pop = || match it.next().map(str::parse) {
      Some(Ok(v)) => Ok(v),
      _ => Err(ParseVersionError),
    };

    let x = pop()?;
    let y = pop()?;
    let z = pop()?;

    if it.next().is_none() {
      Ok(Version(x, y, z))
    } else {
      Err(ParseVersionError)
    }
  }
}

/// Represents prefixed version blocks.
#[derive(Clone, Copy, Debug)]
pub enum VersionBlock {
  /// `x`
  Major(u16),
  /// `x.y`
  Minor(u16, u16),
  /// `x.y.z`
  Patch(u16, u16, u16),
}

impl VersionBlock {
  /// The next version block of the same "size".
  fn successor(&self) -> Self {
    match self {
      B::Major(x) => B::Major(x.checked_add(1).expect("too big major version")),
      B::Minor(x, y) => {
        B::Minor(*x, y.checked_add(1).expect("too big minor version"))
      }
      B::Patch(x, y, z) => {
        B::Patch(*x, *y, z.checked_add(1).expect("too big patch version"))
      }
    }
  }
}

impl VersionRange for VersionBlock {
  fn start(&self) -> Option<Version> {
    match &self {
      B::Major(x) => Some(Version(*x, 0, 0)),
      B::Minor(x, y) => Some(Version(*x, *y, 0)),
      B::Patch(x, y, z) => Some(Version(*x, *y, *z)),
    }
  }

  fn stop(&self) -> Option<Version> {
    self.successor().start()
  }
}

impl fmt::Display for VersionBlock {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      B::Major(x) => write!(f, "{}", x),
      B::Minor(x, y) => write!(f, "{}.{}", x, y),
      B::Patch(x, y, z) => write!(f, "{}.{}.{}", x, y, z),
    }
  }
}

impl str::FromStr for VersionBlock {
  type Err = ParseVersionBlockError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut it = s.split('.');
    let Some(major_str) = it.next() else {
      return Err(ParseVersionBlockError)
    };
    let major = major_str
      .parse::<u16>()
      .map_err(|_| ParseVersionBlockError)?;
    let Some(minor_str) = it.next() else {
      return Ok(B::Major(major));
    };
    let minor = minor_str
      .parse::<u16>()
      .map_err(|_| ParseVersionBlockError)?;
    let Some(patch_str) = it.next() else {
      return Ok(B::Minor(major, minor))
    };
    let patch = patch_str
      .parse::<u16>()
      .map_err(|_| ParseVersionBlockError)?;
    if it.next().is_none() {
      Ok(B::Patch(major, minor, patch))
    } else {
      Err(ParseVersionBlockError)
    }
  }
}

/// Represents contiguous version ranges that may span over multiple version
/// blocks.
#[derive(Clone, Debug)]
pub enum VersionExpr {
  /// The range from the given minimum version up to next breaking version
  /// according to Semantic Versioning.
  MinSemVer(VersionBlock),
  /// `from..to`
  RangeExcl(VersionBlock, VersionBlock),
  /// `from..=to`
  RangeIncl(VersionBlock, VersionBlock),
  /// `from..`
  From(VersionBlock),
  /// `..to`
  UpToExcl(VersionBlock),
  /// `..=to`
  UpToIncl(VersionBlock),
}

impl VersionRange for VersionExpr {
  fn start(&self) -> Option<Version> {
    match self {
      E::MinSemVer(b) => b.start(),
      E::RangeExcl(b, _) => b.start(),
      E::RangeIncl(b, _) => b.start(),
      E::From(b) => b.start(),
      E::UpToExcl(_) => None,
      E::UpToIncl(_) => None,
    }
  }

  fn stop(&self) -> Option<Version> {
    match self {
      // `VersionBlock` is guaranteed to have a `start()`.
      E::MinSemVer(b) => b.start().unwrap().next_break().start(),
      E::RangeExcl(_, b) => b.start(),
      E::RangeIncl(_, b) => b.stop(),
      E::From(_) => None,
      E::UpToExcl(b) => b.start(),
      E::UpToIncl(b) => b.stop(),
    }
  }
}

impl fmt::Display for VersionExpr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      E::MinSemVer(b) => {
        write!(f, "{}..{}", b, b.start().unwrap().next_break())
      }
      E::RangeExcl(b, e) => write!(f, "{}..{}", b, e),
      E::RangeIncl(b, e) => write!(f, "{}..={}", b, e),
      E::From(b) => write!(f, "{}..", b),
      E::UpToExcl(e) => write!(f, "..{}", e),
      E::UpToIncl(e) => write!(f, "..={}", e),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseVersionBlockError;

impl str::FromStr for VersionExpr {
  type Err = ParseVersionExprError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    fn parse_block(s: &str) -> Result<VersionBlock, ParseVersionExprError> {
      s.parse().map_err(|_| ParseVersionExprError)
    }

    fn parse_opt_block(
      s: &str,
    ) -> Result<Option<VersionBlock>, ParseVersionExprError> {
      if s.is_empty() {
        Ok(None)
      } else {
        parse_block(s).map(Some)
      }
    }

    if let Some((start_str, end_str)) = s.split_once("..=") {
      let end = parse_block(end_str)?;
      if let Some(start) = parse_opt_block(start_str)? {
        Ok(E::RangeIncl(start, end))
      } else {
        Ok(E::UpToIncl(end))
      }
    } else if let Some((start_str, end_str)) = s.split_once("..") {
      match (parse_opt_block(start_str)?, parse_opt_block(end_str)?) {
        (Some(start), None) => Ok(E::From(start)),
        (None, Some(end)) => Ok(E::UpToExcl(end)),
        (Some(start), Some(end)) => Ok(E::RangeExcl(start, end)),
        (None, None) => Err(ParseVersionExprError),
      }
    } else {
      Ok(E::MinSemVer(parse_block(s)?))
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParseVersionExprError;

#[cfg(test)]
mod test {

  use super::*;

  fn v(s: &str) -> Version {
    match s.parse() {
      Ok(v) => v,
      Err(e) => panic!(
        "failed to parse version: input = \"{}\", error = {:?}",
        s, e
      ),
    }
  }

  fn b(s: &str) -> VersionBlock {
    match s.parse() {
      Ok(v) => v,
      Err(e) => panic!(
        "failed to parse version block: input = \"{}\", error = {:?}",
        s, e
      ),
    }
  }

  fn e(s: &str) -> VersionExpr {
    match s.parse() {
      Ok(v) => v,
      Err(e) => panic!(
        "failed to parse version expression: input = \"{}\", error = {:?}",
        s, e
      ),
    }
  }

  fn assert_version_parsing_fails(s: &str) {
    if s.parse::<Version>().is_ok() {
      panic!("version parsing should have failed: input = \"{}\"", s);
    }
  }

  fn assert_block_parsing_fails(s: &str) {
    if s.parse::<Version>().is_ok() {
      panic!(
        "version block parsing should have failed: input = \"{}\"",
        s
      );
    }
  }

  fn assert_expression_parsing_fails(s: &str) {
    if s.parse::<Version>().is_ok() {
      panic!(
        "version expression parsing should have failed: input = \"{}\"",
        s
      );
    }
  }

  #[test]
  fn parsing_version() {
    assert!(matches!(v("1.2.3"), Version(1, 2, 3)));

    assert_version_parsing_fails("");
    assert_version_parsing_fails(" ");
    assert_version_parsing_fails("1.2.3 ");
    assert_version_parsing_fails(" 1.2.3");
    assert_version_parsing_fails("1");
    assert_version_parsing_fails("1.");
    assert_version_parsing_fails("1.2");
    assert_version_parsing_fails("1.2.");
    assert_version_parsing_fails("1.2.3.");
    assert_version_parsing_fails("1.2.3.4");
    assert_version_parsing_fails("whatever");
  }

  #[test]
  fn displaying_version() {
    assert_eq!(Version(123, 456, 789).to_string(), "123.456.789");
  }

  #[test]
  fn parsing_version_block() {
    assert!(matches!(b("1"), B::Major(1)));
    assert!(matches!(b("1.2"), B::Minor(1, 2)));
    assert!(matches!(b("1.2.3"), B::Patch(1, 2, 3)));

    assert_block_parsing_fails("");
    assert_block_parsing_fails(" ");
    assert_block_parsing_fails("1 ");
    assert_block_parsing_fails(" 1");
    assert_block_parsing_fails("1.2 ");
    assert_block_parsing_fails(" 1.2");
    assert_block_parsing_fails("1.2.3 ");
    assert_block_parsing_fails(" 1.2.3");
    assert_block_parsing_fails("1.");
    assert_block_parsing_fails("1.2.");
    assert_block_parsing_fails("1.2.3.");
    assert_block_parsing_fails("1.2.3.4");
    assert_block_parsing_fails("whatever");
  }

  #[test]
  fn displaying_version_block() {
    assert_eq!(format!("{}", B::Major(123,)), "123");
    assert_eq!(format!("{}", B::Minor(123, 456)), "123.456");
    assert_eq!(format!("{}", B::Patch(123, 456, 789)), "123.456.789");
  }

  #[test]
  fn parsing_version_expression() {
    assert!(matches!(e("1"), E::MinSemVer(B::Major(1))));
    assert!(matches!(e("1.2"), E::MinSemVer(B::Minor(1, 2))));
    assert!(matches!(e("1.2.3"), E::MinSemVer(B::Patch(1, 2, 3))));

    assert!(matches!(e("1..4"), E::RangeExcl(B::Major(1), B::Major(4))));
    assert!(matches!(
      e("1.2..4.5"),
      E::RangeExcl(B::Minor(1, 2), B::Minor(4, 5))
    ));
    assert!(matches!(
      e("1.2.3..4.5.6"),
      E::RangeExcl(B::Patch(1, 2, 3), B::Patch(4, 5, 6))
    ));

    assert!(matches!(e("1..=4"), E::RangeIncl(B::Major(1), B::Major(4))));
    assert!(matches!(
      e("1.2..=4.5"),
      E::RangeIncl(B::Minor(1, 2), B::Minor(4, 5))
    ));
    assert!(matches!(
      e("1.2.3..=4.5.6"),
      E::RangeIncl(B::Patch(1, 2, 3), B::Patch(4, 5, 6))
    ));

    assert!(matches!(e("1.."), E::From(B::Major(1))));
    assert!(matches!(e("1.2.."), E::From(B::Minor(1, 2))));
    assert!(matches!(e("1.2.3.."), E::From(B::Patch(1, 2, 3))));

    assert!(matches!(e("..4"), E::UpToExcl(B::Major(4))));
    assert!(matches!(e("..4.5"), E::UpToExcl(B::Minor(4, 5))));
    assert!(matches!(e("..4.5.6"), E::UpToExcl(B::Patch(4, 5, 6))));

    assert!(matches!(e("..=4"), E::UpToIncl(B::Major(4))));
    assert!(matches!(e("..=4.5"), E::UpToIncl(B::Minor(4, 5))));
    assert!(matches!(e("..=4.5.6"), E::UpToIncl(B::Patch(4, 5, 6))));

    assert_expression_parsing_fails("");
    assert_expression_parsing_fails(" ");

    assert_expression_parsing_fails("1.2.3 ");
    assert_expression_parsing_fails(" 1");
    assert_expression_parsing_fails("1.2 ");
    assert_expression_parsing_fails(" 1.2");
    assert_expression_parsing_fails("1.2.3 ");
    assert_expression_parsing_fails(" 1.2.3");

    assert_expression_parsing_fails("1.");
    assert_expression_parsing_fails("1.2.");
    assert_expression_parsing_fails("1.2.3.");
    assert_expression_parsing_fails("1.2.3.4");

    assert_expression_parsing_fails("1...4");
    assert_expression_parsing_fails("1.2.3...4");
    assert_expression_parsing_fails("1...4.5.6");
    assert_expression_parsing_fails("1.2.3...4.5.6");

    assert_expression_parsing_fails("1.=4");
    assert_expression_parsing_fails("1.2.3.=4");
    assert_expression_parsing_fails("1.=4.5.6");
    assert_expression_parsing_fails("1.2.3.=4.5.6");

    assert_expression_parsing_fails("1...=4");
    assert_expression_parsing_fails("1.2.3...=4");
    assert_expression_parsing_fails("1...=4.5.6");
    assert_expression_parsing_fails("1.2.3...=4.5.6");

    assert_expression_parsing_fails("1...");
    assert_expression_parsing_fails("1.2.3...");

    assert_expression_parsing_fails(".=4");
    assert_expression_parsing_fails(".=4.5.6");
    assert_expression_parsing_fails("...=4");
    assert_expression_parsing_fails("...=4.5.6");

    assert_expression_parsing_fails(".4");
    assert_expression_parsing_fails(".4.5.6");
    assert_expression_parsing_fails("...4");
    assert_expression_parsing_fails("...4.5.6");

    assert_expression_parsing_fails("whatever");
  }

  #[test]
  fn displaying_version_expression() {
    // below 1.0
    assert_eq!(E::MinSemVer(B::Major(0,)).to_string(), "0..0.1");
    assert_eq!(E::MinSemVer(B::Minor(0, 1)).to_string(), "0.1..0.2");
    assert_eq!(E::MinSemVer(B::Patch(0, 1, 2)).to_string(), "0.1.2..0.2");

    // from 1.0 onwards
    assert_eq!(E::MinSemVer(B::Major(1,)).to_string(), "1..2");
    assert_eq!(E::MinSemVer(B::Minor(1, 2)).to_string(), "1.2..2");
    assert_eq!(E::MinSemVer(B::Patch(1, 2, 3)).to_string(), "1.2.3..2");

    assert_eq!(E::RangeExcl(B::Major(1,), B::Major(4,)).to_string(), "1..4");
    assert_eq!(
      E::RangeExcl(B::Minor(1, 2), B::Minor(4, 5)).to_string(),
      "1.2..4.5"
    );
    assert_eq!(
      E::RangeExcl(B::Patch(1, 2, 3), B::Patch(4, 5, 6)).to_string(),
      "1.2.3..4.5.6"
    );

    assert_eq!(
      E::RangeIncl(B::Major(1,), B::Major(4,)).to_string(),
      "1..=4"
    );
    assert_eq!(
      E::RangeIncl(B::Minor(1, 2), B::Minor(4, 5)).to_string(),
      "1.2..=4.5"
    );
    assert_eq!(
      E::RangeIncl(B::Patch(1, 2, 3), B::Patch(4, 5, 6)).to_string(),
      "1.2.3..=4.5.6"
    );

    assert_eq!(E::UpToExcl(B::Major(4,)).to_string(), "..4");
    assert_eq!(E::UpToExcl(B::Minor(4, 5)).to_string(), "..4.5");
    assert_eq!(E::UpToExcl(B::Patch(4, 5, 6)).to_string(), "..4.5.6");

    assert_eq!(E::UpToIncl(B::Major(4,)).to_string(), "..=4");
    assert_eq!(E::UpToIncl(B::Minor(4, 5)).to_string(), "..=4.5");
    assert_eq!(E::UpToIncl(B::Patch(4, 5, 6)).to_string(), "..=4.5.6");

    assert_eq!(E::From(B::Major(1,)).to_string(), "1..");
    assert_eq!(E::From(B::Minor(1, 2)).to_string(), "1.2..");
    assert_eq!(E::From(B::Patch(1, 2, 3)).to_string(), "1.2.3..");
  }

  #[test]
  fn next_breaking_version() {
    // below 1.0
    assert!(matches!(Version(0, 0, 0).next_break(), B::Minor(0, 1)));
    assert!(matches!(Version(0, 0, 1).next_break(), B::Minor(0, 1)));
    assert!(matches!(Version(0, 1, 0).next_break(), B::Minor(0, 2)));
    assert!(matches!(Version(0, 1, 1).next_break(), B::Minor(0, 2)));

    // from 1.0 onwards
    assert!(matches!(Version(1, 0, 0).next_break(), B::Major(2)));
    assert!(matches!(Version(1, 0, 1).next_break(), B::Major(2)));
    assert!(matches!(Version(1, 1, 0).next_break(), B::Major(2)));
    assert!(matches!(Version(1, 1, 1).next_break(), B::Major(2)));
  }

  #[test]
  fn version_block_starts() {
    // below 1.0
    assert_eq!(B::Major(0).start(), Some(Version(0, 0, 0)));
    assert_eq!(B::Minor(0, 0).start(), Some(Version(0, 0, 0)));
    assert_eq!(B::Patch(0, 0, 0).start(), Some(Version(0, 0, 0)));

    // from 1.0 onwards
    assert_eq!(B::Major(1).start(), Some(Version(1, 0, 0)));
    assert_eq!(B::Minor(1, 2).start(), Some(Version(1, 2, 0)));
    assert_eq!(B::Patch(1, 2, 3).start(), Some(Version(1, 2, 3)));
  }

  #[test]
  fn version_block_stops() {
    // below 1.0
    assert_eq!(B::Major(0).stop(), Some(Version(1, 0, 0)));
    assert_eq!(B::Minor(0, 0).stop(), Some(Version(0, 1, 0)));
    assert_eq!(B::Patch(0, 0, 0).stop(), Some(Version(0, 0, 1)));

    // from 1.0 onwards
    assert_eq!(B::Major(1).stop(), Some(Version(2, 0, 0)));
    assert_eq!(B::Minor(1, 2).stop(), Some(Version(1, 3, 0)));
    assert_eq!(B::Patch(1, 2, 3).stop(), Some(Version(1, 2, 4)));
  }

  #[test]
  fn start_of_expression() {
    // below 1.0
    assert_eq!(E::MinSemVer(B::Major(0)).start(), Some(Version(0, 0, 0)));
    assert_eq!(E::MinSemVer(B::Minor(0, 0)).start(), Some(Version(0, 0, 0)));
    assert_eq!(
      E::MinSemVer(B::Patch(0, 0, 0)).start(),
      Some(Version(0, 0, 0))
    );

    // from 1.0 onwards
    assert_eq!(E::MinSemVer(B::Major(1)).start(), Some(Version(1, 0, 0)));
    assert_eq!(E::MinSemVer(B::Minor(1, 2)).start(), Some(Version(1, 2, 0)));
    assert_eq!(
      E::MinSemVer(B::Patch(1, 2, 3)).start(),
      Some(Version(1, 2, 3))
    );

    assert_eq!(
      E::RangeExcl(B::Major(1), B::Major(4)).start(),
      Some(Version(1, 0, 0))
    );
    assert_eq!(
      E::RangeExcl(B::Minor(1, 2), B::Minor(4, 5)).start(),
      Some(Version(1, 2, 0))
    );
    assert_eq!(
      E::RangeExcl(B::Patch(1, 2, 3), B::Patch(4, 5, 6)).start(),
      Some(Version(1, 2, 3))
    );

    assert_eq!(
      E::RangeIncl(B::Major(1), B::Major(4)).start(),
      Some(Version(1, 0, 0))
    );
    assert_eq!(
      E::RangeIncl(B::Minor(1, 2), B::Minor(4, 5)).start(),
      Some(Version(1, 2, 0))
    );
    assert_eq!(
      E::RangeIncl(B::Patch(1, 2, 3), B::Patch(4, 5, 6)).start(),
      Some(Version(1, 2, 3))
    );

    assert_eq!(E::UpToExcl(B::Major(4)).start(), None);
    assert_eq!(E::UpToExcl(B::Minor(4, 5)).start(), None);
    assert_eq!(E::UpToExcl(B::Patch(4, 5, 6)).start(), None);

    assert_eq!(E::UpToIncl(B::Major(4)).start(), None);
    assert_eq!(E::UpToIncl(B::Minor(4, 5)).start(), None);
    assert_eq!(E::UpToIncl(B::Patch(4, 5, 6)).start(), None);

    assert_eq!(E::From(B::Major(1)).start(), Some(Version(1, 0, 0)));
    assert_eq!(E::From(B::Minor(1, 2)).start(), Some(Version(1, 2, 0)));
    assert_eq!(E::From(B::Patch(1, 2, 3)).start(), Some(Version(1, 2, 3)));
  }

  #[test]
  fn stop_of_expression() {
    // below 1.0
    assert_eq!(E::MinSemVer(B::Major(0)).stop(), Some(Version(0, 1, 0)));
    assert_eq!(E::MinSemVer(B::Minor(0, 0)).stop(), Some(Version(0, 1, 0)));
    assert_eq!(
      E::MinSemVer(B::Patch(0, 0, 0)).stop(),
      Some(Version(0, 1, 0))
    );

    // from 1.0 onwards
    assert_eq!(E::MinSemVer(B::Major(1)).stop(), Some(Version(2, 0, 0)));
    assert_eq!(E::MinSemVer(B::Minor(1, 2)).stop(), Some(Version(2, 0, 0)));
    assert_eq!(
      E::MinSemVer(B::Patch(1, 2, 3)).stop(),
      Some(Version(2, 0, 0))
    );

    assert_eq!(
      E::RangeExcl(B::Major(1), B::Major(4)).stop(),
      Some(Version(4, 0, 0))
    );
    assert_eq!(
      E::RangeExcl(B::Minor(1, 2), B::Minor(4, 5)).stop(),
      Some(Version(4, 5, 0))
    );
    assert_eq!(
      E::RangeExcl(B::Patch(1, 2, 3), B::Patch(4, 5, 6)).stop(),
      Some(Version(4, 5, 6))
    );

    assert_eq!(
      E::RangeIncl(B::Major(1), B::Major(4)).stop(),
      Some(Version(5, 0, 0))
    );
    assert_eq!(
      E::RangeIncl(B::Minor(1, 2), B::Minor(4, 5)).stop(),
      Some(Version(4, 6, 0))
    );
    assert_eq!(
      E::RangeIncl(B::Patch(1, 2, 3), B::Patch(4, 5, 6)).stop(),
      Some(Version(4, 5, 7))
    );

    assert_eq!(E::UpToExcl(B::Major(4)).stop(), Some(Version(4, 0, 0)));
    assert_eq!(E::UpToExcl(B::Minor(4, 5)).stop(), Some(Version(4, 5, 0)));
    assert_eq!(
      E::UpToExcl(B::Patch(4, 5, 6)).stop(),
      Some(Version(4, 5, 6))
    );

    assert_eq!(E::UpToIncl(B::Major(4)).stop(), Some(Version(5, 0, 0)));
    assert_eq!(E::UpToIncl(B::Minor(4, 5)).stop(), Some(Version(4, 6, 0)));
    assert_eq!(
      E::UpToIncl(B::Patch(4, 5, 6)).stop(),
      Some(Version(4, 5, 7))
    );

    assert_eq!(E::From(B::Major(1)).stop(), None);
    assert_eq!(E::From(B::Minor(1, 2)).stop(), None);
    assert_eq!(E::From(B::Patch(1, 2, 3)).stop(), None);
  }

  #[test]
  fn comparing_version_against_expression() {
    use Ordering::*;

    const MAX_VERNUM: u16 = 0xFFFF - 1;

    let v = Version(1, 2, 3);

    assert_eq!(v.range_cmp(&E::MinSemVer(B::Major(0))), Greater);
    assert_eq!(v.range_cmp(&E::MinSemVer(B::Major(1))), Equal);
    assert_eq!(v.range_cmp(&E::MinSemVer(B::Major(2))), Less);

    assert_eq!(v.range_cmp(&E::MinSemVer(B::Minor(0, MAX_VERNUM))), Greater);
    assert_eq!(v.range_cmp(&E::MinSemVer(B::Minor(1, 2))), Equal);
    assert_eq!(v.range_cmp(&E::MinSemVer(B::Minor(1, 3))), Less);

    assert_eq!(
      v.range_cmp(&E::MinSemVer(B::Patch(0, MAX_VERNUM, MAX_VERNUM))),
      Greater
    );
    assert_eq!(v.range_cmp(&E::MinSemVer(B::Patch(1, 2, 3))), Equal);
    assert_eq!(v.range_cmp(&E::MinSemVer(B::Patch(1, 2, 4))), Less);

    assert_eq!(
      v.range_cmp(&E::RangeExcl(B::Major(0), B::Major(1))),
      Greater
    );
    assert_eq!(v.range_cmp(&E::RangeExcl(B::Major(1), B::Major(2))), Equal);
    assert_eq!(v.range_cmp(&E::RangeExcl(B::Major(2), B::Major(3))), Less);

    assert_eq!(
      v.range_cmp(&E::RangeExcl(B::Minor(0, 0), B::Minor(1, 2))),
      Greater
    );
    assert_eq!(
      v.range_cmp(&E::RangeExcl(B::Minor(1, 2), B::Minor(1, 3))),
      Equal
    );
    assert_eq!(
      v.range_cmp(&E::RangeExcl(B::Minor(1, 3), B::Minor(3, 0))),
      Less
    );

    assert_eq!(
      v.range_cmp(&E::RangeExcl(B::Patch(0, 0, 0), B::Patch(1, 2, 3))),
      Greater
    );
    assert_eq!(
      v.range_cmp(&E::RangeExcl(B::Patch(1, 2, 3), B::Patch(1, 2, 4))),
      Equal
    );
    assert_eq!(
      v.range_cmp(&E::RangeExcl(B::Patch(1, 2, 4), B::Patch(3, 0, 0))),
      Less
    );

    assert_eq!(
      v.range_cmp(&E::RangeIncl(B::Major(0), B::Major(0))),
      Greater
    );
    assert_eq!(v.range_cmp(&E::RangeIncl(B::Major(1), B::Major(1))), Equal);
    assert_eq!(v.range_cmp(&E::RangeIncl(B::Major(2), B::Major(2))), Less);

    assert_eq!(
      v.range_cmp(&E::RangeIncl(B::Minor(0, 0), B::Minor(1, 1))),
      Greater
    );
    assert_eq!(
      v.range_cmp(&E::RangeIncl(B::Minor(1, 2), B::Minor(1, 2))),
      Equal
    );
    assert_eq!(
      v.range_cmp(&E::RangeIncl(B::Minor(1, 3), B::Minor(1, MAX_VERNUM))),
      Less
    );

    assert_eq!(
      v.range_cmp(&E::RangeIncl(B::Patch(0, 0, 0), B::Patch(1, 2, 2))),
      Greater
    );
    assert_eq!(
      v.range_cmp(&E::RangeIncl(B::Patch(1, 2, 3), B::Patch(1, 2, 3))),
      Equal
    );
    assert_eq!(
      v.range_cmp(&E::RangeIncl(
        B::Patch(1, 2, 4),
        B::Patch(1, MAX_VERNUM, MAX_VERNUM)
      )),
      Less
    );

    assert_eq!(v.range_cmp(&E::UpToExcl(B::Major(1))), Greater);
    assert_eq!(v.range_cmp(&E::UpToExcl(B::Major(2))), Equal);

    assert_eq!(v.range_cmp(&E::UpToExcl(B::Minor(1, 2))), Greater);
    assert_eq!(v.range_cmp(&E::UpToExcl(B::Minor(1, 3))), Equal);

    assert_eq!(v.range_cmp(&E::UpToExcl(B::Patch(1, 2, 3))), Greater);
    assert_eq!(v.range_cmp(&E::UpToExcl(B::Patch(1, 2, 4))), Equal);

    assert_eq!(v.range_cmp(&E::UpToIncl(B::Major(0))), Greater);
    assert_eq!(v.range_cmp(&E::UpToIncl(B::Major(1))), Equal);

    assert_eq!(v.range_cmp(&E::UpToIncl(B::Minor(1, 1))), Greater);
    assert_eq!(v.range_cmp(&E::UpToIncl(B::Minor(1, 2))), Equal);

    assert_eq!(v.range_cmp(&E::UpToIncl(B::Patch(1, 2, 2))), Greater);
    assert_eq!(v.range_cmp(&E::UpToIncl(B::Patch(1, 2, 3))), Equal);

    assert_eq!(v.range_cmp(&E::From(B::Major(1))), Equal);
    assert_eq!(v.range_cmp(&E::From(B::Major(2))), Less);

    assert_eq!(v.range_cmp(&E::From(B::Minor(1, 2))), Equal);
    assert_eq!(v.range_cmp(&E::From(B::Minor(1, 3))), Less);

    assert_eq!(v.range_cmp(&E::From(B::Patch(1, 2, 3))), Equal);
    assert_eq!(v.range_cmp(&E::From(B::Patch(1, 2, 4))), Less);
  }
}
