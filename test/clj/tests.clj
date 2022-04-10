(ns tests
  (:require
    [clojure.test :refer [run-tests]]
    [tests.disconnection]
    [tests.hello]))

(defn run
  [_]
  (run-tests 'tests.hello
             'tests.disconnection))
