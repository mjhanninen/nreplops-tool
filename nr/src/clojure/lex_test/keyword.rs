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
  assert_lexemes! {
    ":foo",
    Lexeme::Keyword {
      alias: false,
      namespace: None,
      name: "foo",
      source: ":foo",
      ..
    }
  }
}

#[test]
fn keyword_with_dot_in_name() {
  assert_lexemes! {
    ":foo.bar",
    Lexeme::Keyword {
      alias: false,
      namespace: None,
      name: "foo.bar",
      source: ":foo.bar",
      ..
    }
  }
}

#[test]
fn keyword_with_colon_in_name() {
  assert_lexemes! {
    ":foo:bar",
    Lexeme::Keyword {
      alias: false,
      namespace: None,
      name: "foo:bar",
      source: ":foo:bar",
      ..
    }
  }
}

#[test]
fn simple_keyword_with_namespace() {
  assert_lexemes! {
    ":foo/bar",
    Lexeme::Keyword {
      alias: false,
      namespace: Some("foo"),
      name: "bar",
      source: ":foo/bar",
      ..
    }
  }
}

#[test]
fn keyword_with_dotted_namespace_and_name() {
  assert_lexemes! {
    ":foo.bar/zip.zap",
    Lexeme::Keyword {
      alias: false,
      namespace: Some("foo.bar"),
      name: "zip.zap",
      source: ":foo.bar/zip.zap",
      ..
    }
  }
}

#[test]
fn namespaced_keyword_with_slash_in_name() {
  assert_lexemes! {
    ":foo.bar/zip/zap",
    Lexeme::Keyword {
      alias: false,
      namespace: Some("foo.bar"),
      name: "zip/zap",
      source: ":foo.bar/zip/zap",
      ..
    }
  }
}

#[test]
fn namespaced_keyword_with_complex_name_1() {
  assert_lexemes! {
    ":foo.bar/zip/:zap",
    Lexeme::Keyword {
      alias: false,
      namespace: Some("foo.bar"),
      name: "zip/:zap",
      source: ":foo.bar/zip/:zap",
      ..
    }
  }
}

#[test]
fn namespaced_keyword_with_complex_name_2() {
  assert_lexemes! {
    ":foo.bar/zip//:zap",
    Lexeme::Keyword {
      alias: false,
      namespace: Some("foo.bar"),
      name: "zip//:zap",
      source: ":foo.bar/zip//:zap",
      ..
    }
  }
}

#[test]
fn just_slash_keyword() {
  assert_lexemes! {
    ":/",
    Lexeme::Keyword {
      alias: false,
      namespace: None,
      name: "/",
      source: ":/",
      ..
    }
  }
}

#[test]
fn just_slash_keyword_with_namespace() {
  assert_lexemes! {
    ":foo//",
    Lexeme::Keyword {
      alias: false,
      namespace: Some("foo"),
      name: "/",
      source: ":foo//",
      ..
    }
  }
}

#[test]
fn unqualified_begins_with_number() {
  assert_lexemes! {
    ":42",
    Lexeme::Keyword {
      alias: false,
      namespace: None,
      name: "42",
      source: ":42",
      ..
    }
  }
}

#[test]
fn unqualified_begins_with_quote() {
  assert_lexemes! {
    ":'foo",
    Lexeme::Keyword {
      alias: false,
      namespace: None,
      name: "'foo",
      source: ":'foo",
      ..
    }
  }
}

#[test]
fn unqualified_begins_with_hash() {
  assert_lexemes! {
    ":#foo",
    Lexeme::Keyword {
      alias: false,
      namespace: None,
      name: "#foo",
      source: ":#foo",
      ..
    }
  }
}

#[test]
fn unqualified_begins_with_hash_quote() {
  assert_lexemes! {
    ":#'foo",
    Lexeme::Keyword {
      alias: false,
      namespace: None,
      name: "#'foo",
      source: ":#'foo",
      ..
    }
  }
}

#[test]
fn plain_alias() {
  assert_lexemes! {
    "::foo",
    Lexeme::Keyword {
      namespace: None,
      name: "foo",
      alias: true,
      source: "::foo",
      ..
    }
  }
}

#[test]
fn alias_with_dot_in_name() {
  assert_lexemes! {
    "::foo.bar",
    Lexeme::Keyword {
      namespace: None,
      name: "foo.bar",
      alias: true,
      source: "::foo.bar",
      ..
    }
  }
}

#[test]
fn alias_with_colon_in_name() {
  assert_lexemes! {
    "::foo:bar",
    Lexeme::Keyword {
      namespace: None,
      name: "foo:bar",
      alias: true,
      source: "::foo:bar",
      ..
    }
  }
}

#[test]
fn simple_namespaced_alias() {
  assert_lexemes! {
    "::foo/bar",
    Lexeme::Keyword {
      namespace: Some("foo"),
      name: "bar",
      alias: true,
      source: "::foo/bar",
      ..
    }
  }
}

#[test]
fn alias_with_dotted_namespace_and_name() {
  assert_lexemes! {
    "::foo.bar/zip.zap",
    Lexeme::Keyword {
      namespace: Some("foo.bar"),
      name: "zip.zap",
      alias: true,
      source: "::foo.bar/zip.zap",
      ..
    }
  }
}

#[test]
fn alias_with_namespace_and_complex_name_1() {
  assert_lexemes! {
    "::foo.bar/zip/:zap",
    Lexeme::Keyword {
      namespace: Some("foo.bar"),
      name: "zip/:zap",
      alias: true,
      source: "::foo.bar/zip/:zap",
      ..
    }
  }
}

#[test]
fn alias_with_namespace_and_complex_name_2() {
  assert_lexemes! {
    "::foo.bar/zip//:zap",
    Lexeme::Keyword {
      namespace: Some("foo.bar"),
      name: "zip//:zap",
      alias: true,
      source: "::foo.bar/zip//:zap",
      ..
    }
  }
}

// NB: Clojure parser is not able to parser `::/`.
#[test]
fn just_slash_alias() {
  assert_lexemes! {
    "::/",
    Lexeme::Keyword {
      namespace: None,
      name: "/",
      alias: true,
      source: "::/",
      ..
    }
  }
}

#[test]
fn just_slash_alias_with_namespace() {
  assert_lexemes! {
    "::foo//",
    Lexeme::Keyword {
      namespace: Some("foo"),
      name: "/",
      alias: true,
      source: "::foo//",
      ..
    }
  }
}
