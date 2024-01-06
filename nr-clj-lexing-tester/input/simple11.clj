;; simple11.clj -- maps

{}
{:foo 42}
{"bar" {:foo 42}}
{{:anything "goes"} 42}

#:foo {:bar 42}
#::foo {:bar 42}

#::foo {#_ :baz :bar 42}
#::foo {:bar 42 #_ :baz}
