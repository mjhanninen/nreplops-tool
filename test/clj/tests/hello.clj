(ns tests.hello
  (:require
    [clojure.java.shell :refer [sh]]
    [clojure.test :refer [deftest use-fixtures is testing]]
    [tests.util :refer [nrepl-server-fixture *bind* *port* *nr-exe* q]]))

(use-fixtures :each nrepl-server-fixture)

(deftest hello
  (testing "Minimal test"
    (is (sh *nr-exe*
            "-p" (str *bind* ":" *port*)
            "-e" (q (println "Hello, world!")))
        {:exit 0
         :out "Hello, world!\nnil\n"
         :err ""})))
