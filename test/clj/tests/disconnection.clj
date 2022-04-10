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
  (testing "nr aborts when host disconnects"
    ;; Okay, some non-obvious latching mechanism here to ensure proper
    ;; synchronization between the processes before cutting off the nREPL
    ;; server.
    (let [sid (uuid)
          latch (promise)
          _ (swap! state assoc sid latch)
          fut (future
                (sh *nr-exe*
                    "-p" (str *bind* ":" *port*)
                    "-e" (pr-str
                           `(do
                              (-> state
                                  deref
                                  (get ~sid)
                                  (deliver :okay))
                              (while true)))))]
      (is (= :okay (deref latch 1000 :timeout)) "... nr has started")
      (is (not (future-done? fut)) "... nr is hung waiting")
      (nrepl/stop-server *server*)
      (Thread/sleep 1000)
      (is (= :aborted
             (if (future-done? fut)
               :aborted
               (do
                 (future-cancel fut)
                 :hung)))
          "... nr aborts as server is stopped"))))

