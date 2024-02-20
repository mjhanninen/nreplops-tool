use super::*;

#[test]
fn intermingled_discards_and_meta() {
  assert_lexemes(
    "#_ #_{} ^{} {} {}",
    &[
      //
      // We are lexing the form 1 which is the last `{}` in the input.  Hence,
      // the discarded form is a diversion and gets the index 2.
      //
      |l: Lexeme| {
        fields!(
          l,
          Lexeme {
            form_ix == 2,
            parent_ix == 0,
          }
        );
        fields!(l.token, Token::Discard {});
      },
      |l: Lexeme| {
        fields!(
          l,
          Lexeme {
            form_ix == 3,
            parent_ix == 2,
          }
        );
        fields!(l.token, Token::Whitespace {});
      },
      //
      // Currently, we are already lexing the discarded form 2 and, hence, this
      // discard becomes its child.  The actual form that is discarded by this
      // discard is allocated the form index 4 and this discard itself the index
      // 5.
      //
      |l: Lexeme| {
        fields!(
          l,
          Lexeme {
            form_ix == 5,
            parent_ix == 2,
          }
        );
        fields!(l.token, Token::Discard {});
      },
      |l: Lexeme| {
        fields!(
          l,
          Lexeme {
            form_ix == 6,
            parent_ix == 5,
          }
        );
        fields!(l.token, Token::StartMap {});
      },
      |l: Lexeme| {
        fields!(
          l,
          Lexeme {
            form_ix == 6,
            parent_ix == 5,
          }
        );
        fields!(l.token, Token::EndMap {});
      },
      //
      // The following whitespace continues within the first discarded form and,
      // hence, the parent is 2.
      //
      |l: Lexeme| {
        fields!(l.token, Token::Whitespace {});
        fields!(
          l,
          Lexeme {
            form_ix == 7,
            parent_ix == 2,
          }
        );
      },
      //
      // We're lexing the discarded form 2 and, hence, the meta-data becomes
      // its child. The form containing the meta-data itself has already been
      // allocated the index 4 earlier.  (I'm a bit confused how this works;
      // maybe check it?)
      //
      |l: Lexeme| {
        fields!(
          l,
          Lexeme {
            form_ix == 4,
            parent_ix == 2,
          }
        );
        fields!(
          l.token,
          Token::Meta {
            metaform_ix == 8,
            subform_ix == 9,
          }
        );
      },
      |l: Lexeme| {
        fields!(
          l,
          Lexeme {
            form_ix == 8,
            parent_ix == 4,
          }
        );
        fields!(l.token, Token::StartMap {});
      },
      |l: Lexeme| {
        fields!(
          l,
          Lexeme {
            form_ix == 8,
            parent_ix == 4,
          }
        );
        fields!(l.token, Token::EndMap {});
      },
      //
      // This whitespace is within the meta-data composite.
      //
      |l: Lexeme| {
        fields!(
          l,
          Lexeme {
            form_ix == 10,
            parent_ix == 4,
          }
        );
        fields!(l.token, Token::Whitespace {});
      },
      //
      // Now, the subform of the meta-data composite.
      //
      |l: Lexeme| {
        fields!(
          l,
          Lexeme {
            form_ix == 9,
            parent_ix == 4,
          }
        );
        fields!(l.token, Token::StartMap {});
      },
      |l: Lexeme| {
        fields!(
          l,
          Lexeme {
            form_ix == 9,
            parent_ix == 4,
          }
        );
        fields!(l.token, Token::EndMap {});
      },
      |l: Lexeme| {
        fields!(l.token, Token::Whitespace {});
      },
      //
      // And, finally, the form 1.
      //
      |l: Lexeme| {
        fields!(
          l,
          Lexeme {
            form_ix == 1,
            parent_ix == 0,
          }
        );
        fields!(l.token, Token::StartMap {});
      },
      |l: Lexeme| {
        fields!(
          l,
          Lexeme {
            form_ix == 1,
            parent_ix == 0,
          }
        );
        fields!(l.token, Token::EndMap {});
      },
    ],
  );
}
