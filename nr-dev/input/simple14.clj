;; simple14.clj -- tagged literals

#inst"2022-01-01"
#inst "2022-01-01"

#; intermittent comments …
inst ; … are allowed.
"2022-01-01"

#
;; intermittent comments …
inst
;; … are allowed.
"2022-01-01"

#foo [:bar :baz]
#foo {:bar "baz"}

# ^:foo tag ^:bar #_ {} "arg"
