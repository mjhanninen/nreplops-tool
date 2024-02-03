;; simple13.clj -- var-quoted symbols (and forms)

#'foo
#' foo
#'; comment allowed
foo
#'
;; comment allowed
foo

#'foo/bar

;; Departure: Clojure rejects forms other than symbols
#':foo
#'{}

;; Departure: Clojure allows meta-data between var-quote and its object but loses the data.
#' ^:foo ^:bar foo/bar

