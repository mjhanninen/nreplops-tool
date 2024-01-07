;; simple10.clj -- lists and lisk-like forms

()
(foo)
(foo bar)
((foo) (bar))

[]
[foo]
[foo bar]
[[foo] [bar]]

#{}
#{foo}
#{foo bar}
#{#{foo} #{bar}}

[(#{foo} #{bar}) (#{baz})]

#()
#(%)
#(+ % %1 %100)
