# lwhlisp
lwhlisp is a lisp interpreter written in rust, based on [this tutorial](https://www.lwh.jp/lisp/index.html "Building LISP").

# Getting started

Install [Rust](https://www.rust-lang.org/ "Rust Programming Language") and clone this repository.
Then, inside the repository:

```sh
cargo run
```

This will compile the project, then start an interactive session (no compilation is required on subsequent runs).
The interactive session will start by loading the small included standard library (you can find the library in lib/lib.lisp).

It should look something like this:

```common-lisp
Loading standard library...
(define eq? =) => eq?
(define (abs x) (if (< x 0) (- 0 x) x)) => abs
(define (foldl proc init list) (if list (foldl proc (proc init (car list)) (cdr list)) init)) => foldl
[snip]
Finished.
user> 
```

As you can see, each s-expression from the library file is parsed, evaluated, and then the parsed expression is printed, along with the result.

Once you are at the `user>` prompt, you may enter lisp code to be evaluated.

**Note:**
`()` is converted into `nil` at parse time.

# Special forms

## `quote`

Takes a single argument, and returns it without evaluating
``` common-lisp
(quote (a b c)) => (a b c)
```

Since this is a bit long to write, a shorthand is provided:
``` common-lisp
'(a b c) => (a b c)
```

The shorthand gets converted into the full version at parse time.

## `lambda`

``` common-lisp
(lambda (x) (* x x)) => (lambda (x) ((* x x)))
(lambda (x) (+ x x) (* x x)) => (lambda (x) ((+ x x) (* x x)))
```

**NOTE:**
The printing of lambda expressions is flawed.
Since lambda expressions can contain multiple statements, they are internally stored as a list, and the printing reflects that.
Ignore the extra set of parenthesis around the body.

Evaluation:
``` common-lisp
((lambda (x) (* x x)) 7) => 49
((lambda (x) (+ x x) (* x x)) 7) => 49
```

Only the last s-expression is returned.

## `define`

Binds a symbol to a value.

Basic syntax:
``` common-lisp
(define x 7) => x
x => 7
```

Creating a function:
``` common-lisp
(define square (lambda (x) (* x x))) => square
(square 7) => 49
```

Since this is a frequent action, there is special syntax for this:
``` common-lisp
(define (square x) (* x x)) => square
(square 7) => 49
```

You can choose to get the arguments as a list instead:
``` common-lisp
(define (x . a) a) => x
(x 1 2 3) => (1 2 3)
```

Or have one (or more) required arguments, and get the rest as a list:

``` common-lisp
(define (x a . b) (list a b)) => x
(x 1 2 3) => (1 (2 3))
```

(list is a function from the standard library that constructs a list will all of its arguments)

## `defmacro`

Macros work the exact same way as function, except that the arguments to macros are not evaluated.

Macros use the following syntax:
``` common-lisp
(defmacro (name arg...) body...)
```

For example, consider the following macro:
``` common-lisp
(defmacro (ignore x) (cons 'quote (cons x nil))) => ignore
```

If we then evaluate the expression
``` common-lisp
(ignore foo) => foo
```
where foo is a (potentially unbound) symbol, the body of `ignore` will be evaluated with the argument `x` bound to the *unevaluated* symbol `foo`.
The result of this is:
``` common-lisp
(quote . (foo . nil))
```
which is equivalent to:
``` common-lisp
(quote foo)
```
or
``` common-lisp
'foo
```

Finally, evaluating this value will give us the result of evaluating the macro body:
``` common-lisp
foo
```

## `if`

The syntax is as follows:
``` common-lisp
(if test true-expr false-expr)
```

If `test` is not nil, the result of evaluating this expression will be `false-expr`. Else, it will be `true-expr`.

# Example
This is a simple program that calculates factorials in a recursive fashion:
``` common-lisp
(define (factorial x)
        (if (= x 0)
            1
            (* x (factorial (- x 1)))))
```

The base case, if `x=0`, will return `1`.
In all other cases, we will return `fact(x - 1) * x`.

``` common-lisp
(factorial 10) => 3628800
```


# TODO
- [ ] When redefining recursive functions, the old version persists in the environment of the new functions, causing recursion to use the old version of the function.
