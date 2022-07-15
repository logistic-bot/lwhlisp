(define eq? =)

(define (abs x)
   (if (< x 0) (- 0 x) x))

(define (foldl proc init list)
   (if list
      (foldl proc (proc init (car list)) (cdr list))
      init))

(define (foldr proc init list)
   (if list
      (proc (car list) (foldr proc init (cdr list)))
      init))

(define (list . items) (foldr cons nil items))

(define (reverse list)
   (foldl
      (lambda (a x) (cons x a))
      nil
      list))

(define (unary-map proc list)
   (foldr
      (lambda (x rest) (cons (proc x) rest))
      nil
      list))

(define (map proc . arg-lists)
   (if (car arg-lists)
      (cons
         (apply proc (unary-map car arg-lists))
         (apply (quote map) (cons proc (unary-map cdr arg-lists))))
      nil))

(define (append a b) (foldr cons b a))

(define (caar x) (car (car x)))

(define (cadr x) (car (cdr x)))

(defmacro (quasiquote x)
   (if (pair? x)
      (if (eq? (car x) (quote unquote))
         (cadr x)
         (if (eq? (caar x) (quote unquote-splicing))
            (list
               (quote append)
               (cadr (car x))
               (list (quote quasiquote) (cdr x)))
            (list
               (quote cons)
               (list (quote quasiquote) (car x))
               (list (quote quasiquote) (cdr x)))))
      (list (quote quote) x)))

(defmacro (let defs . body)
   (quasiquote
      ((lambda (unquote (map car defs)) (unquote-splicing body))
         (unquote-splicing (map cadr defs)))))

(define +
   (let
      ((old+ +))
      (lambda xs (foldl old+ 0 xs))))

(define -
   (let
      ((old- -))
      (lambda xs (foldl old- (car xs) (cdr xs)))))

(define *
   (let
      ((old* *))
      (lambda xs (foldl old* (car xs) (cdr xs)))))

(define /
   (let
      ((old/ /))
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
      (if (= nil x) 0 1)))

(define (length x)
   (if (pair? x)
      (list-length x)
      (if (string? x) (string-length x) nil)))

