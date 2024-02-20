// clojure/lex_test/mod.rs
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

mod discard_and_meta;
mod keyword;

pub(self) use super::lex::*;

#[macro_export]
macro_rules! fields {
  (@field $value:expr, $field:ident == $expected:expr) =>  {
    if $field != $expected {
      panic!(
        "{}: expected {:?}, got {:?}",
        stringify!($field),
        $expected,
        $field,
      );
    }
  };

  (@field $value:expr, $field:ident => $func:expr) => {
    if !($func)(&$field) {
      panic!(
        "field {}.{}: {:?} fails {}",
        stringify!($value),
        stringify!($field),
        $field,
        stringify!($func)
      );
    }
  };

  ($value:expr, $obj:path { $( $field:ident $op:tt $expected:expr, )+ }) => {
    fields!($value, $obj { $( $field $op $expected ),+ })
  };

  ($value:expr, $obj:path { $( $field:ident $op:tt $expected:expr ),* }) => {{
    #[allow(irrefutable_let_patterns)]
    let $obj { $($field,)* .. } = $value else {
      panic!("expected {}, got {:?}", stringify!($obj), $value);
    };
    $(
      fields!(@field $value, $field $op $expected);
    )*
    true
  }};
}

pub(self) use fields;

#[test]
fn test_check() {
  #[derive(Debug)]
  struct S {
    foo: i32,
    bar: Option<&'static str>,
    baz: bool,
  }

  let s = S {
    foo: 42,
    bar: Some("hello"),
    baz: false,
  };

  fields!(s, S { foo == 42 });
  fields!(s, S { bar => |x: &Option<&str>| x.is_some() });
  fields!(s, S { baz == false });
  fields!(
    s,
    S {
      foo == 42,
      bar => |x: &Option<&str>| *x == Some("hello"),
      baz == false,
    }
  );

  #[derive(Debug)]
  enum E {
    Foo,
    Bar { value: i32 },
  }

  let e_foo = E::Foo;

  fields!(e_foo, E::Foo {});

  let e_bar = E::Bar { value: 42 };

  fields!(e_bar, E::Bar {});
  fields!(e_bar, E::Bar { value == 42 });
  fields!(e_bar, E::Bar { value => |x: &i32| *x == 42 });
}

pub(self) fn test_str<T>(expected: &'static str) -> impl Fn(&T) -> bool
where
  T: AsRef<str>,
{
  move |value: &T| value.as_ref() == expected
}

pub(self) fn test_some_str<T>(
  expected: &'static str,
) -> impl Fn(&Option<T>) -> bool
where
  T: AsRef<str>,
{
  move |value: &Option<T>| {
    value
      .as_ref()
      .map(|s| s.as_ref() == expected)
      .unwrap_or(false)
  }
}

pub(self) fn test_none<T>(value: &Option<T>) -> bool {
  value.is_none()
}

pub(self) fn assert_lexemes(input: &str, tests: &[fn(Lexeme)]) {
  let Ok(lexemes)= lex(input) else {
    panic!("failed to parse input: \"{input}\"");
  };
  let mut it = lexemes.into_iter();
  for func in tests.iter() {
    let Some(lexeme) = it.next() else {
      panic!("unexpectedly ran out of lexemes");
    };
    func(lexeme);
  }
  if let Some(lexeme) = it.next() {
    panic!("unexpted residual lexemes remain: {lexeme:?}");
  }
}

pub(self) fn assert_tokens_and_sources(
  input: &str,
  tests: &[fn(Token, Source)],
) {
  let Ok(lexemes)= lex(input) else {
    panic!("failed to parse input: \"{input}\"");
  };
  let mut it = lexemes.into_iter();
  for func in tests.iter() {
    let Some(lexeme) = it.next() else {
      panic!("unexpectedly ran out of lexemes");
    };
    let Some(source) = lexeme.source else {
      panic!("lexeme has no source: {lexeme:?}");
    };
    func(lexeme.token, source);
  }
  if let Some(lexeme) = it.next() {
    panic!("unexpted residual lexemes remain: {lexeme:?}");
  }
}
