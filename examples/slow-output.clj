#!/usr/bin/env -S nr -!

(->> (range 10)
     (reduce (fn [acc x]
               (println "adding" x)
               (Thread/sleep 1000)
               (+ acc x))
             0)
     (println "total is"))
