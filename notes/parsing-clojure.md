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
form ← lit | sym | list | vector | set | map | ignore-form

quot ← '\''

varquot ← '#\''

synquot ← '`'
unquot-splicing ← '~@'
unquot ← '~'

anon-fn ← '#(' ??? ')'

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

lit ← number | string-lit | char-lit | nil-lit | bool-lit | symval-lit | kw-lit

string-lit ← ???

regex-lit ← '#"' regex-pat '"'
regex-pat ← ???

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
```

Notes:

- `'/'` is in `sym` because ["'/' by itself names the division
  function"][clj-reader]
- can metadata map be a `#:{}` or `#::{}`
- should we be accurate about the arguments of an anonymous functions (`%` and
  friends); or can we get away by being sloppy?

### Ignore macro

It is notable that the ignore macro does not ignore itself but instead
"accumulates":

```.clj
[#_ #_ 1 2 3]
;; ↳ [3]
```

The grammar is:

```
form ← ignored-form* unignored-form

ignored-form ← '#_' ignored-form? unignored-form

unignored-form ← ???
```

### Symbols

The regular expression that matches symbols is defined as:

```.java
static Pattern symbolPat = Pattern.compile("[:]?([\\D&&[^/]].*/)?(/|[\\D&&[^/]][^/]*)");
```

- An optional `:` character signifying a keyword
- An optional namespace part starting with a character that is neither a digit nor
  `/` and terminating at the first `/` character.
- A name part starting with a character that is neither a digit nor a `/` and
  continuing with non-`/` characters

There are some additional rules:

- The pattern is matched on a "token" that is a string of non-whitespace (incl.
  `,`) chars
- The name part cannot end with `:`
- The namespace part cannot end with `:` (or `:/` if you want to include the `/`
  in it)
- There cannot be `::` anywhere except at the very start of the symbol

Some legal symbols:

- `:/`
- `:foo:bar`
- `:foo//`
- `'foo//`
- `'foo//foo` (why is this is possible?!?)

Some **illegal** symbols:

- `::/`
- `::/foo`
- `:foo:/`
- `:foo::bar`
- `:/foo`
- `foo:`
- `//foo`

Clojure parses a the division function is in `sym` because ["'/' by itself names the division
  function"][clj-reader]

```
symbol ← first-symbol-char first sym-tail* | '/'
       | '/'
       | '.'

leading-symbol-char ← alpha | sym-extra

sym-tail  ← alnum | sym-extra
sym-extra ← '*' | '+' | '!' | '-' | '_' | '\'' | '?' | '<' | '>' | '='
```

### String literals

```
string ← '"' ( unescaped-string-content | escape-seq )* '"'

unescaped-string-content ← string-char+

escape-seq ← '\' ( 'b' | 't' | 'n' | 'f' | 'r' | '"' | "'" | '\'
                 | oct-octet
                 | 'u' hex-digit{4}
                 )

string-char ← ! ( '\' | '"' | control-char ) | whitespace-control-char
```

### Numeric literal

Clojure recognizes the following numeric literal types:

| Literal                    | Clojure type           | Examples                  | Counterexamples |
|:---------------------------|:-----------------------|:--------------------------|:----------------|
| Integer                    | `java.lang.Long`       | `123`                     | `0123`          |
| Octal integer              | `java.lang.Long`       | `0123`                    |                 |
| Hexadecimal integer        | `java.lang.Long`       | `0x123`                   |                 |
| Radix integer              | `java.lang.Long`       | `4r123`, `-36r123N`       |                 |
| Big integer                | `clojure.lang.BigIng`  | `123N`                    |                 |
| Big octal integer          | `clojure.lang.BigIng`  | `0123N`                   |                 |
| Big hexadecimal integer    | `clojure.lang.BigIng`  | `0x123N`                  |                 |
| Rational                   | `clojure.lang.Ratio`   | `1/2`, `0123/02`          |                 |
| Floating point             | `java.lang.Double`     | `1.`, `1.2e3`             | `.1`            |
| Big decimals               | `java.math.BigDecimal` | `123M`, `0123M`, `1.2e3M` |                 |

There seems to be few sharp corners that might come as a surprise.  Some of
these are due to keeping up with old conventions (e.g. octals) and others just
by-products of, hmm, the GSD style of the Clojure codebase.

As an example the leading zero of an integer signifies an octal number:

```.clj
0123
;; ↳ 83

08
;; summons syntax error
```

However, the numerator and denominator of a rational may also begin with a
leading zero but both are still regarded as decimal (base) numbers:

```.clj
0123/02
;; ↳ 123/2
```

Also a floating point number must have a whole part:

```
.123
;; summons a syntax error
```

The grammar for parsing a number is:

```
number ← sign? unsigned-number

unsigned-number ← unsigned-big-decimal
                | unsigned-floating-point
                | unsigned-rational
                | unsigned-radix-integer
                | unsigned-big-integer
                | unsigned-integer

unsigned-big-decimal ← ( whole | unsigned-floating-point ) 'M'

unsigned-floating-point ← whole ( fractional base10-exp? | base10-exp )

whole ← dec-digit+

fractional ← '.' dec-digit*

base10-exp ← ( 'e' | 'E' ) sign? dec-digit+

unsigned-rational ← dec-digit+ '/' dec-digit+

unsigned-radix-integer ← radix ( 'r' | 'R' ) ascii-alnum+

radix ← '3' ( '0' | '1' | '2' | '3' | '4' | '5' | '6' )
      | ( '1' | '2' ) dec-digit
      | dec-non-zero

unsigned-big-integer ← unsigned-integer 'N'

unsigned-integer ← unsigned-oct
                 | unsigned-hex
                 | unsigned-dec

unsigned-oct ← '0' oct-digit+

unsigned-hex ← '0' ( 'x' | 'X' ) hex-digit+

sign ← '+' | '-'
```

Again, the ordered choice in `unsigned-number` needs to be ordered with some
care.

### Character literal

```
char-lit ← named-char-lit
         | utf16-char-lit
         | octal-char-lit
         | simple-char-lit

named-char-lit ← '\' ( 'newline' | 'space' | 'tab' | 'formfeed' | 'backspace' )

utf16-char-lit ← '\u' hex-digit{4}

octal-char-lit ← '\o' oct-octet

simple-char-lit ← '\' ( ! control-char | LF | CR )
```

### Helpers

```
bin-digit ← '0' | '1'

oct-digit ← '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7'

oct-octet ← ( '0' | '1' | '2' | '3' ) oct-digit oct-digit
          | oct-digit oct-digit?

dec-digit ← '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'

dec-non-zero ← '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'

hex-digit ← '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
          | 'a' | 'A' | 'b' | 'B' | 'c' | 'C' | 'd' | 'D' | 'e' | 'E'
          | 'f' | 'F'

ascii-alnum ← ascii-alpha | dec-digit

ascii-alpha ← 'a' | 'A' | 'b' | 'B' | 'c' | 'C' | 'd' | 'D' | 'e' | 'E'
            | 'f' | 'F' | 'g' | 'G' | 'h' | 'H' | 'i' | 'I' | 'j' | 'J'
            | 'k' | 'K' | 'l' | 'L' | 'm' | 'M' | 'n' | 'N' | 'o' | 'O'
            | 'p' | 'P' | 'q' | 'Q' | 'r' | 'R' | 's' | 'S' | 't' | 'T'
            | 'u' | 'U' | 'v' | 'V' | 'w' | 'W' | 'x' | 'X' | 'y' | 'Y'
            | 'z' | 'Z'

control-char ← NUL | SOH | STX | ETX | EOT | ENQ | ACK | BEL | BS | HT | LF | VT
             | FF | CR | SO | SI | DLE | DC1 | DC2 | DC3 | DC4 | NAK | SYN | ETB
             | CAN | EM | SUB | ESC | FS | GS | RS | US | DEL

whitespace-control-char ← HT | LF | VT | FF | CR
```

## Related resources (uncurated)

- ["The Reader"][clj-reader]. Clojure.org.
- the [LispReader.java][clojure:lisp-reader] source in the Clojure codebase

[clojure:lisp-reader]: https://github.com/clojure/clojure/blob/35bd89f05f8dc4aec47001ca10fe9163abc02ea6/src/jvm/clojure/lang/LispReader.java

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
