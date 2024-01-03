;; simple04.clj -- symbols

:foo
:foo.bar
:foo:bar
:foo/bar
:foo.bar/zip.zap
:foo.bar/zip/zap
:foo.bar/zip/:zap
:foo.bar/zip//:zap
:/
:foo//

::foo
::foo.bar
::foo:bar
::foo/bar
::foo.bar/zip.zap
::foo.bar/zip/:zap
::foo.bar/zip//:zap
::/                   ; not permitted by Clojure
::foo//
