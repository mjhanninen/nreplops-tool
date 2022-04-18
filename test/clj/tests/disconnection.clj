(ns tests.disconnection
  (:require
    [clojure.java.shell :refer [sh]]
    [clojure.test :refer [deftest use-fixtures is testing]]
    [nrepl.server :as nrepl]
    [tests.util :refer [*bind* *nr-exe* *port* *server* nrepl-server-fixture q]]))

(use-fixtures :each nrepl-server-fixture)

(defn uuid
  []
  (str (java.util.UUID/randomUUID)))

(defonce state (atom {}))

(deftest host-disconnects
  (testing "nr aborts when the host disconnects"
    ;; Okay, some non-obvious latching mechanism here to ensure proper
    ;; synchronization between the processes before cutting off the nREPL
    ;; server.
    (let [sid (uuid)
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
              :err "Error: unexpected disconnection by host\n"
              :exit 1}
             @nr)
          "... nr process outcome reflects disconnection"))))

