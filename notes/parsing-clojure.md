# Parsing Clojure

Firstly, our goal is not to fully parse Clojure; you would need a Clojure to do
that.  Instead we want to perform a [lexical analysis][wp:lexan] that sufficient
for carrying out the following:

- Locate and parse the "template" arguments in the source and allow them to have
  relatively rich features set (e.g. descriptions, default values, splicing
  remaining positional arguments, etc.).
- Parse evaluation results and convert them into into JSON.
- Parse socket prepl's responses.
- Extract the leading comment block, if present, and be able to use that to
  produce a `--help` message for the script.
- Perform rewrites based on simple rules (e.g. replacing `clojure.pprint/pprint`
  with `puget.printer/cprint`).

[wp:lexan]: https://en.wikipedia.org/wiki/Lexical_analysis

## Parsing expresssion grammar

**Note:** The order is meaningful in a PEG grammar.

**Note:** This is very far from ready. Quite sketchy.

```
form ← lit | sym | list | vector | set | map

quot ← '\''

varquot ← '#\''

synquot ← '`'
unquot-splicing ← '~@'
unquot ← '~'

anon-fn ← '#(' ??? ')'

ignore ← '#_'

comment ← ';' ( ! eol ) *

deref ← '@'


meta ← '^' ( map | sym | keyword | string )

list ← '(' form* ')'

vector ← '[' form* ']'

set ← "#{" form* '}'

map       ← ns-map | alias-map | plain-map
ns-map    ← ':' ns '{' map-entry* '}'
alias-map ← '::' alias '{' map-entry* '}'
plain-map ← '{' map-entry* '}'
map-entry ← form form

alias ← ???

lit ← number-lit | string-lit | char-lit | nil-lit | bool-lit | symval-lit | kw-lit

number-lit ← ???

string-lit ← ???

regex-lit ← '#"' regex-pat '"'
regex-pat ← ???

char-lit         ← named-char-lit
                 | unicode-char-lit
                 | octal-char-lit
                 | simple-char-lit
named-char-lit   ← '\' ( "newline" | "space" | "tab" | "formfeed" | "backspace" )
unicode-char-lit ← "\u" hex{4}
octal-char-lit   ← "\o" oct{3}
simple-char-lit  ← '\' ???

nil-lit ← "nil"

bool-lit ← "true" | "false"

symval-lit ← "##Inf" | "##-Inf" | "##NaN"

kw-lit ← ???

reader-cond       ← rc-direct | rc-splicing
rc-direct         ← '#?(' rc-entry* ')'
rc-splicing       ← '#@?(' rc-entry-splicing* ')'
rc-key            ← simple-keyword (* or ':clj' and so on *)
rc-entry          ← rc-key form
rc-entry-splicing ← rc-key coll

tagged-lit ← '#' alpha-sym form
alpha-sym  ← (* like sym but starts with alpha; could be called "tag" *)

qualified-sym ← ns '/' sym

ns ← ns-part ( '.' ns-part )*
ns-part ← ns-char+
ns-char ← ???

sym       ← sym-first sym-tail* | '/'
sym-first ← alpha | sym-extra
sym-tail  ← alnum | sym-extra
sym-extra ← '*' | '+' | '!' | '-' | '_' | '\'' | '?' | '<' | '>' | '='

alnum ← alpha | num
num   ← '0' | … | '9'
alpha ← 'a' | … | 'z' | 'A' | … | 'Z'
```

Notes:

- `'/'` is in `sym` because ["'/' by itself names the division
  function"][clj-reader]
- can metadata map be a `#:{}` or `#::{}`
- should we be accurate about the arguments of an anonymous functions (`%` and
  friends); or can we get away by being sloppy?

## Related resources (uncurated)

- ["The Reader"][clj-reader]. Clojure.org.

## Related resources (uncurated)

- [`clojure-emacs/parseclj`][parseclj]: A Clojure Parser for Emacs. GitHub.
- Feichtinger, Thomas (2015) ["TruffleClojure: A self-optimizing
  AST-interpreter for Clojure"][feichtinger:2015]. Master's Thesis. Johannes
  Keppler Universitä Linz.
- [`clojure/tools.analyzer`][tools.analyzer]: An analyzer for host agnostic
  Clojure code. GitHub.
- [`timothypratley/clojure-ebnf-grammar`][timothypratley]: An Instaparse EBNF
  grammar for the Clojure language. GitHub.
- [`jpmonettas/clindex`][clindex]: Indexes Clojure project into a datascript
  database. GitHub.
- [`oakes/parinferish`][parinferish]: Parses Clojure and applies parinfer(ish)
  transformation to it. GitHub.
- Raku Advent Calendar (2020) ["Day 4: Parsing Clojure namespace forms using
  Raku grammars"][rac]
- [`naomijub/edn-rs`][edn-rs]: A crate for parsing and emitting EDN. GitHub.
- [`clojure-rs/ClojureRS`][clojure-rs]: a Clojure interpreter implemented in
  Rust. GitHub.

[clindex]: https://github.com/jpmonettas/clindex
[clojure-rs]: https://github.com/clojure-rs/ClojureRS
[edn-rs]: https://github.com/naomijub/edn-rs
[feichtinger:2015]: https://epub.jku.at/obvulihs/download/pdf/501665
[parinferish]: https://github.com/oakes/parinferish
[parseclj]: https://github.com/clojure-emacs/parseclj
[rac]: https://raku-advent.blog/2020/12/04/day-4-parsing-clojure-namespaces-with-grammars/
[timothypratley]: https://github.com/timothypratley/clojure-ebnf-grammar
[tools.analyzer]: https://github.com/clojure/tools.analyzer
[clj-reader]: https://clojure.org/reference/reader
