use super::*;

use Lexeme as L;

#[test]
fn interminled_discards_and_meta() {
  assert_lexemes!(
    "#_ #_{} ^{} {} {}",
    //
    // We are lexing the form 1 and, hence, the discarded for is a diversion and
    // gets the index 2.
    //
    L::Discard {
      form_ix: FormIx { parent: 0, ix: 2 },
      source: "#_",
    },
    L::Whitespace { source: " " },
    //
    // Currently, we are lexing the discarded form 2 and, hence, the this discard
    // starts a new diversion and gets the index 3.
    //
    L::Discard {
      form_ix: FormIx { parent: 0, ix: 3 },
      source: "#_",
    },
    L::StartMap {
      form_ix: FormIx { parent: 0, ix: 3 },
      alias: false,
      namespace: None,
      source: "{",
    },
    L::EndMap {
      form_ix: FormIx { parent: 0, ix: 3 },
      source: "}",
    },
    L::Whitespace { source: " " },
    //
    // We're back lexing the discarded form 2 and, hence, the meta-data relates
    // to it.  The form containing the meta-data itself get the index 4.
    //
    L::Meta {
      form_ix: FormIx { parent: 0, ix: 2 },
      data_ix: FormIx { parent: 0, ix: 4 },
      source: "^",
    },
    L::StartMap {
      form_ix: FormIx { parent: 0, ix: 4 },
      alias: false,
      namespace: None,
      source: "{",
    },
    L::EndMap {
      form_ix: FormIx { parent: 0, ix: 4 },
      source: "}",
    },
    L::Whitespace { source: " " },
    //
    // Now, the discarded form 2.
    //
    L::StartMap {
      form_ix: FormIx { parent: 0, ix: 2 },
      alias: false,
      namespace: None,
      source: "{",
    },
    L::EndMap {
      form_ix: FormIx { parent: 0, ix: 2 },
      source: "}",
    },
    L::Whitespace { source: " " },
    //
    // And, finally, the form 1.
    //
    L::StartMap {
      form_ix: FormIx { parent: 0, ix: 1 },
      alias: false,
      namespace: None,
      source: "{",
    },
    L::EndMap {
      form_ix: FormIx { parent: 0, ix: 1 },
      source: "}",
    }
  );
}
