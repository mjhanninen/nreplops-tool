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
