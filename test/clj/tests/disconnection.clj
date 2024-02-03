;; tests/disconnection.clj
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

(ns tests.disconnection
  (:require
    [clojure.java.shell :refer [sh]]
    [clojure.test :refer [deftest use-fixtures is testing]]
    [nrepl.server :as nrepl]
    [tests.util :refer [*bind* *nr-exe* *port* *server* nrepl-server-fixture q]]))

(use-fixtures :each nrepl-server-fixture)

(defonce state (atom {}))

(deftest host-disconnects
  (testing "nr aborts when the host disconnects"
    ;; Okay, some non-obvious latching mechanism here to ensure proper
    ;; synchronization between the processes before cutting off the nREPL
    ;; server.
    (let [sid (random-uuid)
          latch (promise)
          _ (swap! state assoc sid latch)
          nr (future
               (sh *nr-exe*
                   "-p" (str *bind* ":" *port*)
                   "-e" (pr-str
                          `(do
                             (-> state
                                 deref
                                 (get ~sid)
                                 (deliver :started))
                             (while true)))))]
      (is (= :started (deref latch 1000 :timeout)) "... nr has started")
      (is (not (future-done? nr)) "... nr is waiting in infinite loop")
      (nrepl/stop-server *server*)
      (Thread/sleep 1000)
      (is (= :aborted
             (if (future-done? nr)
               :aborted
               (do
                 (future-cancel nr)
                 :hung)))
          "... nr aborts as the server is stopped")
      (is (= {:out ""
              :err "Error: host disconnected unexpectedly\n"
              :exit 1}
             @nr)
          "... nr process outcome reflects disconnection"))))

