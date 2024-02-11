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
macro_rules! assert_lexemes {

  (@attr $enum:path, $field:ident == $expect:expr) => {
    if $field != $expect {
      panic!("field {}.{}: {:?} != {:?}", stringify!($enum), stringify!($field), $field, $expect);

    }
  };

  (@attr $enum:path, $field:ident => $func:expr) => {
    if !($func)(&$field) {
      panic!("field {}.{}: {:?} fails {}", stringify!($enum), stringify!($field), $field, stringify!($func));
    }
  };

  ($input:expr, [ $( $enum:path { $( $field:ident $op:tt $expect_attr:expr ),* } ),+ ]) => {
      let input = $input;
      let Ok(lexemes) = lex(&input) else {
        panic!("failed to parse: \"{}\"", input);
      };
      let mut it = lexemes.into_iter();
      $(
        {
          let Some(actual) = it.next() else {
            panic!("expected: {}, got no lexeme", stringify!($enum));
          };
          let $enum { $( $field, )* .. } = actual else {
            panic!("expected: {}, got: {:?}", stringify!($enum), actual);
          };
          $(
            assert_lexemes!(@attr $enum, $field $op $expect_attr);
          )*
        }
      )+
      assert!(it.next().is_none(), "unexpected residual lexemes remain");

  }
}

pub(self) use assert_lexemes;
