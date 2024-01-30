(letfn [(fibo
          [x]
          (cond
            (> x 1) (let [res-1 (fibo (- x 1))
                          res-2 (fibo (- x 2))]
                      {:value (+ (:value res-1) (:value res-2))
                       :expr  (list '+ (:value res-1) (:value res-2))
                       :subs  [res-1 res-2]})
            (= x 1) {:value 1
                     :expr  1}
            :else   {:value 0
                     :expr  0}))]
  (fibo 8))
