(define eq? =)

(define (abs x) (if (< x 0) (- 0 x) x))

(define (foldl proc init list)
  (if list
      (foldl proc
             (proc init (car list))
             (cdr list))
      init))

(define (foldr proc init list)
  (if list
      (proc (car list)
            (foldr proc init (cdr list)))
      init))

(define (list . items)
  (foldr cons nil items))

(define (reverse list)
  (foldl (lambda (a x) (cons x a )) nil list))

(define (unary-map proc list)
  (foldr (lambda (x rest) (cons (proc x) rest))
         nil
         list))

(define (map proc . arg-lists)
  (if (car arg-lists)
      (cons (apply proc (unary-map car arg-lists))
            (apply 'map (cons proc
                             (unary-map cdr arg-lists))))
      nil))

(define (append a b) (foldr cons b a))

(define (caar x) (car (car x)))

(define (cadr x) (car (cdr x)))

(defmacro (quasiquote x)
  (if (pair? x)
      (if (eq? (car x) 'unquote)
          (cadr x)
          (if (eq? (caar x) 'unquote-splicing)
              (list 'append
                    (cadr (car x))
                    (list 'quasiquote (cdr x)))
              (list 'cons
                    (list 'quasiquote (car x))
                    (list 'quasiquote (cdr x)))))
      (list 'quote x)))

(defmacro (let defs . body)
  `((lambda ,(map car defs) ,@body)
    ,@(map cadr defs)))

(define +
  (let ((old+ +))
    (lambda xs (foldl old+ 0 xs))))

(define -
  (let ((old- -))
    (lambda xs (foldl old- (car xs) (cdr xs)))))

(define *
  (let ((old* *))
    (lambda xs (foldl old* (car xs) (cdr xs)))))

(define /
  (let ((old/ /))
    (lambda xs (foldl old/ (car xs) (cdr xs)))))


(define (last x)
  (if (pair? x)
      (if (pair? (cdr x))
          (last (cdr x))
          (if (= nil (cdr x))
              (car x)
              (cdr x)))
      (x)))

(define (list-length x)
  (if (pair? x)
      (+ 1 (list-length (cdr x)))
      (if (= nil x)
          0
          1)))

(define (length x)
  (if (pair? x)
      (list-length x)
      (if (string? x)
          (string-length x)
          nil)))

(define (pp x)
  (println (into-pretty-string x)))

(define (integers-down-to-zero x)
    (if (= x 0)
        '(0 . nil)
        (cons x (integers-down-to-zero (- x 1)))))

(define (integers-up-to x)
  (reverse (integers-down-to-zero x)))

(define (filter lst func)
   (if (nilp lst)
      nil
      (if (= (func (car lst)) t)
         (cons (car lst) (filter (cdr lst) func))
         (filter (cdr lst) func))))

(define (nilp x) (= nil x))

(define %
   (let
      ((old% %))
      (lambda (x y)
         (if (nilp x) nil (old% x y)))))

(define (or x y)
  (if x
      t
      (if y
          t
          nil)))

(define (sum x)
  (foldl + 0 x))
