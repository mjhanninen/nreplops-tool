mod discard_and_meta;
mod keyword;

pub(self) use super::lex::*;

#[macro_export]
macro_rules! assert_lexemes {
  ( $input:expr, $( $expect:pat ),+ ) => {
    {
      let input = $input;
      let Ok(lexemes) = lex(&input) else {
        panic!("failed to parse: \"{}\"", input);
      };
      let mut it = lexemes.into_iter();
      $(
        {
          let actual = it.next();
          assert!(matches!(
              actual,
              Some($expect),
            ),
            "expected: {}, got: {:?}",
            stringify!($expect),
            actual
          );
        }
      )+
      assert!(it.next().is_none(), "unexpected residual lexemes remain");
    }
  }
}

pub(self) use assert_lexemes;
