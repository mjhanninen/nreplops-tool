# Parsing Clojure

Firstly, our goal is not to fully parse Clojure; you would need a Clojure to
do that.  Instead we want to perform a [lexical analysis][wp:lexan] that is
sufficient for carrying out the following:

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

## Note from reading Clojure parser code

### Symbols

Clojure reads symbols (including keywords) as follows:

1. Take a contiguous string of characters that are neither whitespace nor any of
   `,`, `;`, `!`, `"`, `^`, `` ` ``, `~`, `(`, `)`, `[`, `]`, `{`, `}`, or `\`
2. Check that the string matches the following Java regular expression pattern:
   ```.java
   Pattern.compile("[:]?([\\D&&[^/]].*/)?(/|[\\D&&[^/]][^/]*)")
   ```
3. Check that the string does **not** satisfy any of the following conditions:
   - The name part ends with `:`
   - The string matching the namespace part of the pattern (including also the
     `/`) ends with `:/`.
   - There a `::` somewhere else than at the very start of the symbol
4. After this the symbol is interned.  It is notable that at this stage the
   symbol may be split into a namespace and name in a way that is in
   disagreement with the way the pattern in the step 2 divides them.  This is
   probably a bug in the reader code.

Some legal symbols:

- `:123/foo`: the `:` is captured by the first `\D`
- `:/`
- `://foo`
- `:foo:bar`
- `:foo//`
- `'foo//`
- `'foo//bar`: The **last** `/` matches the namespace separator in the pattern
  and all `/`s leading to it belong to the namespace portion. Interestingly
  later on when the symbol is being interned it is split into namespace and name
  parts at the **first** `/`. As a result `(name 'foo//bar)` is `"/bar"`. Funky
  stuff.
- `'foo://bar`: At first sight this seems to be in conflict with the checks in
  the step 3. However the namespace portion of the pattern matches `foo://` and,
  hence, it is perfectly okay.  On the other hand `'foo:/bar` does not pass the
  checks.
- `'foo/123/bar`: `(name 'foo/123/bar)` is `123/bar`, lol

Some **illegal** symbols:

- `::/`
- `::/foo`
- `:foo:/`
- `:foo::bar`
- `:/foo`
- `foo:`
- `//foo`
- `'foo:/bar`

### Metadata and namespaced maps

- Metadata "accumulates". For example, `(def ^:foo ^:bar hello "world")` sets
  both metadata entries `:foo` `:bar` for `#'hello`.

Can metadata map be a `#:{}` or `#::{}`?  Yes, it can.  Try, for example

```.clj
(def ^#:foo{:bar 42} hello "world")
```

It works just fine and, `(meta #'hello)` contains an entry for `:foo/bar`.

### Discard macro

It is notable that the discard macro does not ignore itself but instead
"accumulates" like so:

```.clj
[#_ #_ 1 2 3]
;; ↳ [3]
```

### Metadata and disard macro

The discard macro `#^` tosses the next form as well as all metas between it and
that form:

```.clj
(map (fn [x] {:value x :meta (meta x)}) [#^:foo #^:bar [1] [2]])
;; => ({:value [1], :meta {:bar true, :foo true}} {:value [2], :meta nil})
(map (fn [x] {:value x :meta (meta x)}) [#^:foo #^:bar #_ [1] [2]])
;; => ({:value [2], :meta {:bar true, :foo true}})
(map (fn [x] {:value x :meta (meta x)}) [#^:foo #_ #^:bar [1] [2]])
;; => ({:value [2], :meta {:foo true}})
(map (fn [x] {:value x :meta (meta x)}) [#_ #^:foo #^:bar [1] [2]])
;; => ({:value [2], :meta nil})
```

### Quoting

Some examples:

```clojure
' ^:foo ()                          ; => ()
(meta ' ^:foo ())                   ; => {:foo true}
' ^:foo ' ^:bar ()                  ; => (quote ())
(meta ' ^:foo ' ^:bar ())           ; => {:foo true}
(meta (unquote ' ^:foo ' ^:bar ())) ; => {:bar true}
```

Also:

```clojure
' #_ () (bar)                       ; => (bar)
#_ ' () (bar)                       ; => syntax error: unresolvable symbol bar
```

Implications:

- We need to distinguish between the meta data expressions somehow

### Reader conditionals

```clojure
#?(:clj 1 :cljs 2)
[#?()]                    ; => [] i.e. conditional produces nothing, not even `nil`
#?(:clj               1
   :foo               2
   {:anything "goes"} 3)  ; keys can be anything

#? , , , , , , ,(:clj 1)  ; horizonal whitespace allowed prefix and opening `(`

#?(:clj #?(:clj 1))       ; => 1 i.e. can be nested
#?(:clj 'foo)             ; => foo i.e. reader quotes before replacement
(meta #?(:clj ^:bar {}))  ; => {:bar true}

[#?@(:clj (range 3))]     ; => [#object[range] 3], not [0 1 2]
```

Notable:

- Keys can be anything
- Value forms are not evaluated by reader conditional

Simplifications:

- Clojure permits horizontal whitespace between the prefix and the opening `(`
  but not vertical whitespace or comments.  We allow general whitespace and
  comments also.
- Clojure does not permit splicing at the top level; we do.
- Clojure does not permit non-sequential values the form to splice; we do.

### Tagged literals

```
#inst"2022-01-01"                       ; => #inst "2022-01-01T00:00:00.000-00:00"

#
;; comments …
inst
;; … welcomed
"2022-01-01"                            ; => #inst "2022-01-01T00:00:00.000-00:00"

# ^:foo #_ ^:bar [] inst "2022-01-01"   ; => #inst "2022-01-01T00:00:00.000-00:00"
```

Notes:

- `deftype`, `defrecord`, and object constructor calls can be understood as
  special case of tagged literals

### Numeric literals

Clojure recognizes the following numeric literal types:

| Literal                    | Clojure type           | Examples                  | Counterexamples |
|:---------------------------|:-----------------------|:--------------------------|:----------------|
| Integer                    | `java.lang.Long`       | `123`                     | `0123`          |
| Octal integer              | `java.lang.Long`       | `0123`                    |                 |
| Hexadecimal integer        | `java.lang.Long`       | `0x123`                   |                 |
| Radix integer              | `java.lang.Long`       | `4r123`, `-36r123N`       |                 |
| Big integer                | `clojure.lang.BigInt`  | `123N`                    |                 |
| Big octal integer          | `clojure.lang.BigInt`  | `0123N`                   |                 |
| Big hexadecimal integer    | `clojure.lang.BigInt`  | `0x123N`                  |                 |
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

Also a floating point number **must** have a whole part:

```
.123
;; summons a syntax error
```

### Dispatch macros, constructors, and tagged literals

Upon hitting a `#` character the reader peeks the following character and
dispatches according to the following dispatch macro table:

```.java
dispatchMacros['^'] = new MetaReader();
dispatchMacros['#'] = new SymbolicValueReader();
dispatchMacros['\''] = new VarReader();
dispatchMacros['"'] = new RegexReader();
dispatchMacros['('] = new FnReader();
dispatchMacros['{'] = new SetReader();
dispatchMacros['='] = new EvalReader();
dispatchMacros['!'] = new CommentReader();
dispatchMacros['<'] = new UnreadableReader();
dispatchMacros['_'] = new DiscardReader();
dispatchMacros['?'] = new ConditionalReader();
dispatchMacros[':'] = new NamespaceMapReader();
```

Most of the dispatch macros are probably quite familiar.  The comment reader
acts just like an end-of-line comment.

If the following character does not match any of the above macro then the
following **symbol** and the **form** following that are interpreted either as
a **tagged literal** or, if the symbol contains a `.` character, as a type or
record constructor.

## Parsing expresssion grammar

**Note:** Work in progress. This is very far from ready.

Simplifications:

- the parsing of symbols (qualified by namespace or otherwise) is simplified
- anonymous functions are allowed to nest
- the arguments to anonymous are not treated specially; they are just symbols
- meta-data is applicable to all forms (e.g. to `42`)

Other assumptions:

- no need to track the line or column position nor facilitate it

Few notes about PEG grammar used:

- The order is meaningful in (any) PEG grammar
- Parenthesis `( ... )` group terms
- Compound rule: `my-rule = [ ... ]`; implicits are suppressed
- Atomic rule: `my-rule = { ... }`; inner rules produduce no tokens, implicits are suppressed
- Forced token rule: `my-rule = < ... >`; produces tokens even within an outer atomic rule
- Negative lookaehd: `! my-rule`; does not consume

### Comments, whitespace, and end of line

The grammar contains **implicit** rules for comments and whitespace meaning that
they are **implicitly** permitted between the terms of any non-atomic rule.

```
COMMENT        = { comment-prefix comment-char* }

comment-prefix = ';' | '#!'

comment-char   = ! ( CR | LF ) ANY
```

Note how the line-end comment does not consume the actual end of the line.  That
will be consumed by the whitespace rule:

```
WHITESPACE       = { ( ascii-whitespace | ',' )+ }

ascii-whitespace = { ( HT | LF | VT | FF | CR | ' ' )+ }
```

### Top-level, forms, meta data, and discarding

```
top-level              = start-of-input form* discarded-form? end-of-input

form                   = expr
                       | quoted-form
                       | synquoted-form
                       | splicing-unquoted-form
                       | unquoted-form
                       | preform+ form

preform                = meta-expr | discarded-form
discarded-form         = '#_' preform* form
meta-expr              = ( '^' | '#^' ) form

quoted-form            = '\'' preform* form
synquoted-form         = '`'  preform* form
splicing-unquoted-form = '~@' preform* form
unquoted-form          = '~'  preform* form
```

Note that in Clojure the meta data expression can be either a map or one of a
keyword, symbol, or string in which case promoted into a map.

Here the meta data expression is allowed to contain any Clojure form.  This
simplifies the grammar, although constraining the value should not add very
much compelxity.

Simplifications:

- We allow unquoted forms outside a synquoted form. Clojure parser rejects
  these.  Obviously, these are not valid Clojure but from the point of view of
  **lexical** analysis this is a practical simplification.

- Likewise we allow nested unquoted forms with a synquoted form between them on
  similar ground.

### Expressions

Expression is either a value or a program that evaluates into a value (even if
the result is a nil value).

```
expr = nil
     | booleam
     | number
     | string
     | symbolic-value
     | symbol
     | var-quoted-symbol
     | keyword
     | list
     | vector
     | map
     | set
     | reader-conditional
```

### Nil

```
nil = { "nil" ! symbol-char }
```

### Boolean

```
boolean = { ( "true" | "false" ) ! symbol-char }
```

### Numeric literal

```
number             = [ sign? unsigned-number ! ( ascii-alpha | dec-digit )]

sign               = '+' | '-'

unsigned-number    = unsigned-bigfloat
                   | unsigned-float
                   | unsigned-rational
                   | unsigned-radix-int
                   | unsigned-bigint
                   | unsigned-int

unsigned-bigfloat  = [ ( dec-digits | unsigned-float ) 'M' ]

unsigned-float     = [ dec-digits ( fractional base10-exp? | base10-exp ) ]
fractional         = { '.' dec-digit* }
base10-exp         = [ ( 'e' | 'E' ) sign? dec-digits ]

unsigned-rational  = [ dec-digits '/' dec-digits ]

unsigned-radix-int = [ radix ( 'r' | 'R' ) radix-arg ]
radix              = { unsigned-dec }
radix-arg          = { ( ascii-alpha | dec-digit )+ }

unsigned-bigint    = [ unsigned-int 'N' ]

unsigned-int       = unsigned-oct
                   | unsigned-hex
                   | unsigned-dec
unsigned-oct       = [ oct-prefix oct-digits ]
unsigned-hex       = [ hex-prefix hex-digits ]
unsigned-dec       = { dec-non-zero dec-digit* | '0' }

oct-prefix         = '0'
oct-digits         = { '0'..'7'+ }

hex-prefix         = '0x' | '0X'
hex-digits         = { ( '0'..'9' | 'a'..'f' | 'A..F' )+ }

dec-digits         = { dec-digit+ }
dec-non-zero       = '1'..'9'
dec-digit          = '0'..'9'
```

- The order of the choice is very important inside both `unsigned-number` and
  `unsigned-int`.

Simplifications:

- Radix is permitted to be any non-negative integer.  This should be constrained
  to the range 2…36 at some later stage.  Clojure reads only two digits of the
  radix. (Try `36r1`, `99r1`, and `100r1` to see the difference.)

### Character literals

```
char         = [ '\\'
                 ( char-name
                 | 'o' char-octal
                 | 'u' char-unicode
                 | char-simple
                 )
                 ! symbol-char
               ]
char-name    = 'newline' | 'space' | 'tab' | 'formfeed' | 'backspace'
char-octal   = { oct-digit oct-digit? oct-digit? }
char-unicode = { hex-digit hex-digit hex-digit hex-digit }
char-simple  = ! ( control-char | ' ' ) ANY
```

### String literals

```
string      = [ '"' ( unescaped | escape-seq )* '"' ]

unescaped   = { string-char+ }

string-char = ascii-whitespace
            | ! ( '\\' | '"' | control-char ) ANY

escape-seq  = { '\\' ( 'b' | 't' | 'n' | 'f' | 'r' | '"' | '\\'
                     | oct-octet
                     | 'u' hex-word
                     )
              }
```

Simplifications:

- Clojure does not permit octal octets shorter than 3 chars unless immediately
  followed by a `\` or `"` char.

### Symbolic value

```
symbolic-value = "##" symbol
```

Note:

- that only the three symbols `Inf`, `-Inf`, and `NaN` are recognized.  But
  as far as lexical analysis is concerned lexing it the way described above
  is accuracte.
- Both Clojure and this grammar permit whitespace (incl. comma) between `##` and
  the symbol.

### Symbols

The symbol reading code in the Clojure parser is particularly messy (and, dare I
say, broken).  The following grammar does not even try replicate all the corner
cases. As an example the following is legal Clojure but rejected by our grammar:

```.clj
'foo//bar
```

However, in practice you should not notice any difference if your code is
reasonably sane:

```
symbol             = [ namespace '/' qualified-symbol
                     | unqualified-symbol
                     ]

namespace          = { symbol-first-char symbol-char* ( ':' symbol-char+ )* }

unqualified-symbol = { namespace
                     | '/' ! ( symbol-char | '/' | ':' )
                     }

qualified-symbol   = { ( '/'* ':'? symbol-char+ )+
                     | '/'+ ! ':'
                     }

symbol-char        = { symbol-first-char | dec-digit }

symbol-first-char  = ! ( WHITESPACE
                       | control-char
                       | dec-digit
                       | ';' | '"' | '^' | '`' |  '~' | '(' | ')'
                       | '[' | ']' | '{' | '}' | '\\' | ':' | '/'
                       )
                     ANY
```

```
continuation-char  = ! ( WHITESPACE
                       | control-char
                       | ';' | '"' | '^' | '`' |  '~' | '(' | ')'
                       | '[' | ']' | '{' | '}' | '\\'
                       )
                     ANY
```

Keywords build directly on symbols:

```
keyword               = [ keyword-prefix ( namespace '/' qualified-symbol
                                         | unqualified-keyword
                                         )
                        ]

keyword-prefix        = ':'   ; namespaced
                      | '::'  ; alias

unqualified-keyword = { symbol-char+ ( ':' symbol-char+ )*
                      | '/' ! ( symbol-char | '\' | ':' )
                      }
```

Some observations from Clojure:

- `:456` is a valid keyword
- `456` is a number
- `:456abc` is a valid keyword
- `456abc` is an invalid number
- `:123/456` is an invalid keyword
- `:123/def` is a valid keyword
- `:abc/456` is an invalid keyword
- `#:123{:def "foo"}` is an invalid namespaced map
- `#:abs{:123 "foo"}` is a valid namespaced map
- likewise for `::`

Go figure.  In any case we make the following simplifications (or clarifications):

- Permit unqualified keywords (but not symbols) starting with a digit
- Always reject namespaces (and aliases) starting with a digit

### Lists

```
list = '(' form* discarded-form? ')'
```

### Vectors

```
vector = '[' form* discarded-form? ']'
```

### Sets

```
set = '#{' form* discarded-form? '}'
```

### Maps

```
map             = map-qualifier? unqualified-map

map-qualifier   = [ '#:' namespace   ; namespaced
                  | '#::' namespace  ; alias
                  ]

unqualified-map = '{' map-entry* discarded-form? '}'

map-entry       = form form
```

Simplifications:

- the non-atomic "binding" between the qualification and the
  unqualified map permits comments (through implicit comment rule) while Clojure
  does not.

### Grammar

```
form ← lit | sym | list | vector | set | map | ignore-form

quot ← '\''

varquot ← '#\''

synquot         ← '`'
unquot-splicing ← '~@'
unquot          ← '~'

comment ← ';' ( ! eol ) *

deref ← '@'


list ← '(' form* ')'

vector ← '[' form* ']'

set ← "#{" form* '}'

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

```

Notes:

- `'/'` is in `sym` because ["'/' by itself names the division function"][clj-reader]

- Should we be accurate about the arguments of an anonymous functions (`%` and
  friends); or can we get away by being sloppy?

### Anonymous function

- Nested `#(...)` are not allowed
- `%` is alias for `%1`
- `%n` where `n` is positive integer
- `%&` is remaining args
- but since `%`, `%1`, and `%&` are normal symbols outside function reader we
  could get away by treating an anonymous function like a list form
- as far as lexical analysis is concerned this is would be the right thing to do
  anyway ...
- ... although Clojure is not able to parse `(let [%foo 42] #(+ %foo %1))` while
  our simplified version would happily accept it

```
anon-fn      ← '#(' anon-fn-body ')'
anon-fn-body ← (* TODO *)
anon-arg     ← `%&` | `%` pos-dec-int | `%`
```

### Character literal

```
char-lit ← named-char-lit
         | utf16-char-lit
         | octal-char-lit
         | simple-char-lit

named-char-lit ← '\' ( 'newline' | 'space' | 'tab' | 'formfeed' | 'backspace' )

utf16-char-lit ← '\u' hex-digit{4}

octal-char-lit ← '\o' oct-octet

simple-char-lit ← '\' ( ! control-char )
```

### Helpers

```
bin-digit = '0' | '1'

oct-digit = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7'

oct-octet = { ( '0' | '1' | '2' | '3' ) oct-digit oct-digit
            | oct-digit oct-digit?
            }

dec-digit = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'

dec-non-zero = '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'

hex-digit = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
          | 'a' | 'A' | 'b' | 'B' | 'c' | 'C' | 'd' | 'D' | 'e' | 'E'
          | 'f' | 'F'

hex-octet = { hex-digit hex-digit }

hew-word  = { hex-octet hex-octet }

ascii-alnum = ascii-alpha | dec-digit

ascii-alpha = 'a' | 'A' | 'b' | 'B' | 'c' | 'C' | 'd' | 'D' | 'e' | 'E'
            | 'f' | 'F' | 'g' | 'G' | 'h' | 'H' | 'i' | 'I' | 'j' | 'J'
            | 'k' | 'K' | 'l' | 'L' | 'm' | 'M' | 'n' | 'N' | 'o' | 'O'
            | 'p' | 'P' | 'q' | 'Q' | 'r' | 'R' | 's' | 'S' | 't' | 'T'
            | 'u' | 'U' | 'v' | 'V' | 'w' | 'W' | 'x' | 'X' | 'y' | 'Y'
            | 'z' | 'Z'

control-char = NUL | SOH | STX | ETX | EOT | ENQ | ACK | BEL | BS | HT | LF | VT
             | FF | CR | SO | SI | DLE | DC1 | DC2 | DC3 | DC4 | NAK | SYN | ETB
             | CAN | EM | SUB | ESC | FS | GS | RS | US | DEL

whitespace-control-char = HT | LF | VT | FF | CR

whitespace = whitespace-control-char | ' ' | ','
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
