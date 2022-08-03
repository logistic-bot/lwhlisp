(println
   (sum
    (filter
        (cdr (integers-up-to 999))
        (lambda (x) (or (= (% x 3) 0)
                    (= (% x 5) 0))))))
