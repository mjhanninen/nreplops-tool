// clojure/lex_test/keyword.rs
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

use super::*;

use std::rc::Rc;

#[test]
fn plain_keyword() {
  assert_lexemes! {
    ":foo",
    [
      Lexeme::Keyword {
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        name => |n: &str| { n == "foo"},
        source => |s: &str| { s == ":foo" }
      }
    ]
  }
}

#[test]
fn keyword_with_dot_in_name() {
  assert_lexemes! {
    ":foo.bar",
    [
      Lexeme::Keyword {
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        name => |n: &str| { n ==  "foo.bar" },
        source => |s: &str| { s == ":foo.bar" }
      }
    ]
  }
}

#[test]
fn keyword_with_colon_in_name() {
  assert_lexemes! {
    ":foo:bar",
    [
      Lexeme::Keyword {
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        name => |n: &str| { n == "foo:bar" },
        source => |s: &str| { s == ":foo:bar" }
      }
    ]
  }
}

#[test]
fn simple_keyword_with_namespace() {
  assert_lexemes! {
    ":foo/bar",
    [
      Lexeme::Keyword {
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.as_deref() == Some("foo") },
        name => |n: &str| { n == "bar" },
        source => |s: &str| { s == ":foo/bar" }
      }
    ]
  }
}

#[test]
fn keyword_with_dotted_namespace_and_name() {
  assert_lexemes! {
    ":foo.bar/zip.zap",
    [
      Lexeme::Keyword {
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.as_deref() == Some("foo.bar") },
        name => |n: &str| { n == "zip.zap" },
        source => |s: &str| { s == ":foo.bar/zip.zap" }
      }
    ]
  }
}

#[test]
fn namespaced_keyword_with_slash_in_name() {
  assert_lexemes! {
    ":foo.bar/zip/zap",
    [
      Lexeme::Keyword {
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.as_deref() == Some("foo.bar") },
        name => |n: &str| { n == "zip/zap" },
        source => |s: &str| { s == ":foo.bar/zip/zap" }
      }
    ]
  }
}

#[test]
fn namespaced_keyword_with_complex_name_1() {
  assert_lexemes! {
    ":foo.bar/zip/:zap",
    [
      Lexeme::Keyword {
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.as_deref() == Some("foo.bar") },
        name => |n: &str| { n == "zip/:zap" },
        source => |s: &str| { s == ":foo.bar/zip/:zap" }
      }
    ]
  }
}

#[test]
fn namespaced_keyword_with_complex_name_2() {
  assert_lexemes! {
    ":foo.bar/zip//:zap",
    [
      Lexeme::Keyword {
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.as_deref() == Some("foo.bar") },
        name => |n: &str| { n == "zip//:zap" },
        source => |s: &str| { s == ":foo.bar/zip//:zap" }
      }
    ]
  }
}

#[test]
fn just_slash_keyword() {
  assert_lexemes! {
    ":/",
    [
      Lexeme::Keyword {
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        name => |n: &str| { n == "/" },
        source => |s: &str| { s == ":/" }
      }
    ]
  }
}

#[test]
fn just_slash_keyword_with_namespace() {
  assert_lexemes! {
    ":foo//",
    [
      Lexeme::Keyword {
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.as_deref() == Some("foo") },
        name => |n: &str| { n == "/" },
        source => |s: &str| { s == ":foo//" }
      }
    ]
  }
}

#[test]
fn unqualified_begins_with_number() {
  assert_lexemes! {
    ":42",
    [
      Lexeme::Keyword {
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        name => |n: &str| { n == "42" },
        source => |s: &str| { s == ":42" }
      }
    ]
  }
}

#[test]
fn unqualified_begins_with_quote() {
  assert_lexemes! {
    ":'foo",
    [
      Lexeme::Keyword {
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        name => |n: &str| { n == "'foo" },
        source => |s: &str| { s == ":'foo" }
      }
    ]
  }
}

#[test]
fn unqualified_begins_with_hash() {
  assert_lexemes! {
    ":#foo",
    [
      Lexeme::Keyword {
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        name => |n: &str| { n == "#foo" },
        source => |s: &str| { s == ":#foo" }
      }
    ]
  }
}

#[test]
fn unqualified_begins_with_hash_quote() {
  assert_lexemes! {
    ":#'foo",
    [
      Lexeme::Keyword {
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        name => |n: &str| { n == "#'foo" },
        source => |s: &str| { s == ":#'foo" }
      }
    ]
  }
}

#[test]
fn plain_alias() {
  assert_lexemes! {
    "::foo",
    [
      Lexeme::Keyword {
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        name => |n: &str| { n == "foo" },
        alias == true,
        source => |s: &str| { s == "::foo" }
      }
    ]
  }
}

#[test]
fn alias_with_dot_in_name() {
  assert_lexemes! {
    "::foo.bar",
    [
      Lexeme::Keyword {
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        name => |n: &str| { n == "foo.bar" },
        alias == true,
        source => |s: &str| { s == "::foo.bar" }
      }
    ]
  }
}

#[test]
fn alias_with_colon_in_name() {
  assert_lexemes! {
    "::foo:bar",
    [
      Lexeme::Keyword {
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        name => |n: &str| { n == "foo:bar" },
        alias == true,
        source => |s: &str| { s == "::foo:bar" }
      }
    ]
  }
}

#[test]
fn simple_namespaced_alias() {
  assert_lexemes! {
    "::foo/bar",
    [
      Lexeme::Keyword {
        namespace => |ns: &Option<Rc<str>>| { ns.as_deref() == Some("foo") },
        name => |n: &str| { n == "bar" },
        alias == true,
        source => |s: &str| { s == "::foo/bar" }
      }
    ]
  }
}

#[test]
fn alias_with_dotted_namespace_and_name() {
  assert_lexemes! {
    "::foo.bar/zip.zap",
    [
      Lexeme::Keyword {
        namespace => |ns: &Option<Rc<str>>| { ns.as_deref() == Some("foo.bar") },
        name => |n: &str| { n == "zip.zap" },
        alias == true,
        source => |s: &str| { s == "::foo.bar/zip.zap" }
      }
    ]
  }
}

#[test]
fn alias_with_namespace_and_complex_name_1() {
  assert_lexemes! {
    "::foo.bar/zip/:zap",
    [
      Lexeme::Keyword {
        namespace => |ns: &Option<Rc<str>>| { ns.as_deref() == Some("foo.bar") },
        name => |n: &str| { n == "zip/:zap" },
        alias == true,
        source => |s: &str| { s == "::foo.bar/zip/:zap" }
      }
    ]
  }
}

#[test]
fn alias_with_namespace_and_complex_name_2() {
  assert_lexemes! {
    "::foo.bar/zip//:zap",
    [
      Lexeme::Keyword {
        namespace => |ns: &Option<Rc<str>>| { ns.as_deref() == Some("foo.bar") },
        name => |n: &str| { n == "zip//:zap" },
        alias == true,
        source => |s: &str| { s == "::foo.bar/zip//:zap" }
      }
    ]
  }
}

// NB: Clojure parser is not able to parser `::/`.
#[test]
fn just_slash_alias() {
  assert_lexemes! {
    "::/",
    [
      Lexeme::Keyword {
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        name => |n: &str| { n == "/" },
        alias == true,
        source => |s: &str| { s == "::/" }
      }
    ]
  }
}

#[test]
fn just_slash_alias_with_namespace() {
  assert_lexemes! {
    "::foo//",
    [
      Lexeme::Keyword {
        namespace => |ns: &Option<Rc<str>>| { ns.as_deref() == Some("foo") },
        name => |n: &str| { n == "/" },
        alias == true,
        source => |s: &str| { s == "::foo//" }
      }
    ]
  }
}
