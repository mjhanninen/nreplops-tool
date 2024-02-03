;; tests/hello.clj
;; Copyright 2022 Matti Hänninen
;;
;; Licensed under the Apache License, Version 2.0 (the "License"); you may not
;; use this file except in compliance with the License. You may obtain a copy of
;; the License at
;;
;;     http://www.apache.org/licenses/LICENSE-2.0
;;
;; Unless required by applicable law or agreed to in writing, software
;; distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
;; WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
;; License for the specific language governing permissions and limitations under
;; the License.

(ns tests.hello
  (:require
    [clojure.java.shell :refer [sh]]
    [clojure.test :refer [deftest use-fixtures is testing]]
    [tests.util :refer [nrepl-server-fixture *bind* *port* *nr-exe* q]]))

(use-fixtures :each nrepl-server-fixture)

(deftest hello
  (testing "Minimal test"
    (is (=  {:exit 0
             :out "Hello, world!\nnil\n"
             :err ""}
            (sh *nr-exe*
                "-p" (str *bind* ":" *port*)
                "-e" (q (println "Hello, world!")))))))
