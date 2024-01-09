;; simple11.clj -- maps and map-like forms

{}
{:foo 42}
{"bar" {:foo 42}}
{{:anything "goes"} 42}

#:foo {:bar 42}
#:foo , {:bar 42}
#::foo {:bar 42}
#::foo , {:bar 42}

#:foo{#_ :discarded :bar 42}
#:foo{:bar 42 #_ :discarded}

#?()
#?(:clj "clojure" :cljs "clojurescript")
#?({:whatever "goes"} 42 #_ :discarded)

#?@(:clj ["splicing"])

#? , ()
#?@ , ()
