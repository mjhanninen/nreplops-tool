;; tests/util.clj
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

(ns tests.util
  (:require
    [nrepl.server :as nrepl]))

(defonce ^:dynamic *nr-exe* "target/debug/nr")

(defonce ^:dynamic *server* nil)

(defonce ^:dynamic *bind* nil)

(defonce ^:dynamic *port* nil)

(defn parse-socket-addr
  [s]
  (when-let [[_ bind port] (re-matches #"/(.+):(\d+)" s)]
    {:bind bind
     :port (Long/parseLong port)}))

(defmacro with-nrepl-server
  [opts & body]
  `(let [server# (nrepl/start-server ~(or opts {}))
         addr# (-> server#
                   :server-socket
                   .getLocalSocketAddress
                   str
                   parse-socket-addr)]
     (binding [*server* server#
               *bind* (:bind addr#)
               *port* (:port addr#)]
       (try
         ~@body
         (finally
           (nrepl/stop-server server#))))))

(defn nrepl-server-fixture
  [f]
  (with-nrepl-server nil (f)))

(defmacro q
  [& body]
  `(pr-str (quote ~@body)))
