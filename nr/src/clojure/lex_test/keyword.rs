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

#[test]
fn plain_keyword() {
  assert_tokens_and_sources(
    ":foo",
    &[|t: Token, s: Source| {
      fields!(t,
        Token::Keyword {
          alias == false,
          namespace => test_none,
          name => test_str("foo"),
        }
      );
      fields!(s,
        Source {
          str => test_str(":foo"),
        }
      );
    }],
  )
}

#[test]
fn keyword_with_dot_in_name() {
  assert_tokens_and_sources(
    ":foo.bar",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          alias == false,
          namespace => test_none ,
          name => test_str("foo.bar"),
        }
      );
      fields!(
        s,
        Source {
          str => test_str(":foo.bar"),
        }
      );
    }],
  )
}

#[test]
fn keyword_with_colon_in_name() {
  assert_tokens_and_sources(
    ":foo:bar",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          alias == false,
          namespace => test_none,
          name => test_str("foo:bar"),
        }
      );
      fields!(
        s,
        Source {
          str => test_str(":foo:bar"),
        }
      );
    }],
  )
}

#[test]
fn simple_keyword_with_namespace() {
  assert_tokens_and_sources(
    ":foo/bar",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          alias == false,
          namespace => test_some_str("foo"),
          name => test_str("bar"),
        }
      );
      fields!(
        s,
        Source {
          str => test_str(":foo/bar"),
        }
      );
    }],
  )
}

#[test]
fn keyword_with_dotted_namespace_and_name() {
  assert_tokens_and_sources(
    ":foo.bar/zip.zap",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          alias == false,
          namespace => test_some_str("foo.bar"),
          name => test_str("zip.zap"),
        }
      );
      fields!(
        s,
        Source {
          str => test_str(":foo.bar/zip.zap"),
        }
      );
    }],
  )
}

#[test]
fn namespaced_keyword_with_slash_in_name() {
  assert_tokens_and_sources(
    ":foo.bar/zip/zap",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          alias == false,
          namespace => test_some_str("foo.bar"),
          name => test_str("zip/zap"),
        }
      );
      fields!(
        s,
        Source {
          str => test_str(":foo.bar/zip/zap"),
        }
      );
    }],
  );
}

#[test]
fn namespaced_keyword_with_complex_name_1() {
  assert_tokens_and_sources(
    ":foo.bar/zip/:zap",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          alias == false,
          namespace => test_some_str("foo.bar"),
          name => test_str("zip/:zap"),
        }
      );
      fields!(
        s,
        Source {
          str => test_str(":foo.bar/zip/:zap"),
        }
      );
    }],
  )
}

#[test]
fn namespaced_keyword_with_complex_name_2() {
  assert_tokens_and_sources(
    ":foo.bar/zip//:zap",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          alias == false,
          namespace => test_some_str("foo.bar"),
          name => test_str("zip//:zap"),
        }
      );
      fields!(
        s,
        Source {
          str => test_str(":foo.bar/zip//:zap"),
        }
      );
    }],
  )
}

#[test]
fn just_slash_keyword() {
  assert_tokens_and_sources(
    ":/",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          alias == false,
          namespace => test_none,
          name => test_str("/"),
        }
      );
      fields!(
        s,
        Source {
          str => test_str(":/"),
        }
      );
    }],
  )
}

#[test]
fn just_slash_keyword_with_namespace() {
  assert_tokens_and_sources(
    ":foo//",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          alias == false,
          namespace => test_some_str("foo"),
          name => test_str("/"),
        }
      );
      fields!(
        s,
        Source {
          str => test_str(":foo//"),
        }
      );
    }],
  )
}

#[test]
fn unqualified_begins_with_number() {
  assert_tokens_and_sources(
    ":42",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          alias == false,
          namespace => test_none,
          name => test_str("42"),
        }
      );
      fields!(
        s,
        Source {
          str => test_str(":42"),
        }
      );
    }],
  )
}

#[test]
fn unqualified_begins_with_quote() {
  assert_tokens_and_sources(
    ":'foo",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          alias == false,
          namespace => test_none,
          name => test_str("'foo"),
        }
      );
      fields!(
        s,
        Source {
          str => test_str(":'foo"),
        }
      );
    }],
  )
}

#[test]
fn unqualified_begins_with_hash() {
  assert_tokens_and_sources(
    ":#foo",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          alias == false,
          namespace => test_none,
          name => test_str("#foo"),
        }
      );
      fields!(
        s,
        Source {
          str => test_str(":#foo"),
        }
      );
    }],
  )
}

#[test]
fn unqualified_begins_with_hash_quote() {
  assert_tokens_and_sources(
    ":#'foo",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          alias == false,
          namespace => test_none,
          name => test_str("#'foo"),
        }
      );
      fields!(
        s,
        Source {
          str => test_str(":#'foo"),
        }
      );
    }],
  )
}

#[test]
fn plain_alias() {
  assert_tokens_and_sources(
    "::foo",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          namespace => test_none,
          name => test_str("foo"),
          alias == true,
        }
      );
      fields!(
        s,
        Source {
          str => test_str("::foo"),
        }
      );
    }],
  )
}

#[test]
fn alias_with_dot_in_name() {
  assert_tokens_and_sources(
    "::foo.bar",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          namespace => test_none,
          name => test_str("foo.bar"),
          alias == true,
        }
      );
      fields!(
        s,
        Source {
          str => test_str("::foo.bar"),
        }
      );
    }],
  )
}

#[test]
fn alias_with_colon_in_name() {
  assert_tokens_and_sources(
    "::foo:bar",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          namespace => test_none,
          name => test_str("foo:bar"),
          alias == true,
        }
      );
      fields!(
        s,
        Source {
          str => test_str("::foo:bar"),
        }
      );
    }],
  )
}

#[test]
fn simple_namespaced_alias() {
  assert_tokens_and_sources(
    "::foo/bar",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          namespace => test_some_str("foo"),
          name => test_str("bar"),
          alias == true,
        }
      );
      fields!(
        s,
        Source {
          str => test_str("::foo/bar"),
        }
      );
    }],
  )
}

#[test]
fn alias_with_dotted_namespace_and_name() {
  assert_tokens_and_sources(
    "::foo.bar/zip.zap",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          namespace => test_some_str("foo.bar"),
          name => test_str("zip.zap"),
          alias == true,
        }
      );
      fields!(
        s,
        Source {
          str => test_str("::foo.bar/zip.zap"),
        }
      );
    }],
  )
}

#[test]
fn alias_with_namespace_and_complex_name_1() {
  assert_tokens_and_sources(
    "::foo.bar/zip/:zap",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          namespace => test_some_str("foo.bar"),
          name => test_str("zip/:zap"),
          alias == true,
        }
      );
      fields!(
        s,
        Source {
          str => test_str("::foo.bar/zip/:zap"),
        }
      );
    }],
  )
}

#[test]
fn alias_with_namespace_and_complex_name_2() {
  assert_tokens_and_sources(
    "::foo.bar/zip//:zap",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          namespace => test_some_str("foo.bar"),
          name => test_str("zip//:zap"),
          alias == true,
        }
      );
      fields!(
        s,
        Source {
          str => test_str("::foo.bar/zip//:zap"),
        }
      );
    }],
  )
}

// NB: Clojure parser is not able to parser `::/`.
#[test]
fn just_slash_alias() {
  assert_tokens_and_sources(
    "::/",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          namespace => test_none,
          name => test_str("/"),
          alias == true,
        }
      );
      fields!(
        s,
        Source {
          str => test_str("::/"),
        }
      );
    }],
  )
}

#[test]
fn just_slash_alias_with_namespace() {
  assert_tokens_and_sources(
    "::foo//",
    &[|t: Token, s: Source| {
      fields!(
        t,
        Token::Keyword {
          namespace => test_some_str("foo"),
          name => test_str("/"),
          alias == true,
        }
      );
      fields!(
        s,
        Source {
          str => test_str("::foo//"),
        }
      );
    }],
  )
}
