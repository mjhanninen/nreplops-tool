use std::rc::Rc;

use super::*;

use Lexeme as L;

#[test]
fn interminled_discards_and_meta() {
  assert_lexemes!(
    "#_ #_{} ^{} {} {}",
    [
      //
      // We are lexing the form 1 and, hence, the discarded for is a diversion and
      // gets the index 2.
      //
      L::Discard {
        form_ix == FormIx { parent: 0, ix: 2 }
      },
      L::Whitespace { source => |s: &str| { s == " " } },
      //
      // Currently, we are lexing the discarded form 2 and, hence, the this discard
      // starts a new diversion and gets the index 3.
      //
      L::Discard {
        form_ix == FormIx { parent: 0, ix: 3 },
        source => |s: &str| { s == "#_" }
      },
      L::StartMap {
        form_ix == FormIx { parent: 0, ix: 3 },
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        source => |s: &str| { s == "{" }
      },
      L::EndMap {
        form_ix == FormIx { parent: 0, ix: 3 },
        source => |s: &str| { s == "}" }
      },
      L::Whitespace { source => |s: &str| { s == " " } },
      //
      // We're back lexing the discarded form 2 and, hence, the meta-data relates
      // to it.  The form containing the meta-data itself get the index 4.
      //
      L::Meta {
        form_ix == FormIx { parent: 0, ix: 2 },
        data_ix == FormIx { parent: 0, ix: 4 },
        source => |s: &str| { s == "^" }
      },
      L::StartMap {
        form_ix == FormIx { parent: 0, ix: 4 },
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        source => |s: &str| { s == "{" }
      },
      L::EndMap {
        form_ix == FormIx { parent: 0, ix: 4 },
        source => |s: &str| { s == "}" }
      },
      L::Whitespace { source => |s: &str| { s == " " } },
      //
      // Now, the discarded form 2.
      //
      L::StartMap {
        form_ix == FormIx { parent: 0, ix: 2 },
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        source => |s: &str| { s == "{" }
      },
      L::EndMap {
        form_ix == FormIx { parent: 0, ix: 2 },
        source => |s: &str| { s == "}" }
      },
      L::Whitespace { source => |s: &str| { s == " " } },
      //
      // And, finally, the form 1.
      //
      L::StartMap {
        form_ix == FormIx { parent: 0, ix: 1 },
        alias == false,
        namespace => |ns: &Option<Rc<str>>| { ns.is_none() },
        source => |s: &str| { s == "{" }
      },
      L::EndMap {
        form_ix == FormIx { parent: 0, ix: 1 },
        source => |s: &str| { s == "}" }
      }
    ]
  );
}
