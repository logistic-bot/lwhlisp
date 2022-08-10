# lwhlisp
lwhlisp is a lisp interpreter written in rust, based on [this tutorial](https://www.lwh.jp/lisp/index.html "Building LISP").

## Getting started

Install [Rust](https://www.rust-lang.org/ "Rust Programming Language") and clone this repository.
Then, inside the repository:

```sh
cargo run --release
```

This will compile the project and launch a REPL. The finished binary is `target/release/lwhlisp`.

**NOTE**:
The interactive session will start by loading the small included standard library (you can find the library in lib/lib.lisp).

If no such file is found, it will fail to load and you will get an error that looks like this:

```sh
Error: 
   0: While opening library file
   1: While opening file lib/lib.lisp
   2: No such file or directory (os error 2)
```

You can solve this problem by manually indicating where lwhlisp can find the library file:

```sh
cargo run --release -- --library /path/to/library/file.lisp
```
(The `--` separates arguments to cargo and arguments to lwhlisp. It can be omitted when calling the `lwhlisp` binary directly.)

The REPL should look something like this:

```common-lisp
user> 
```

Once you are at the `user>` prompt, you may enter lisp code to be evaluated.

```common-lisp
user> (+ 1 2 3)
=> 6
user> (if (> 5 4) (println "Five is bigger than Four") (println "Five is smaller than Four"))
Five is bigger than Four
=> "Five is bigger than Four"
```

You can also run files:

```sh
cargo run --release -- -f file.lisp
```

**factorial.lisp**:

```common-lisp
(define (factorial x)
   (if (= x 0)
      1
      (* x (factorial (- x 1)))))

(println (factorial 10))
```

```sh
$ cargo run --release -- -f file.lisp
3628800
```

## Syntax
`()` is converted into `nil` at parse time.

### `quote`

Takes a single argument, and returns it without evaluating
```common-lisp
user> (quote (a b c))
=> (a b c)
```

Since this is used frequently, a shorthand is provided:
```common-lisp
user> '(a b c)
=> (a b c)
```

The shorthand gets converted into the full version at parse time.

### `lambda`

```common-lisp
user> (lambda (x) (* x x)) 
=> (lambda (x) (* x x))
```

You can have multiple s-expressions in the body:

```common-lisp
user> (lambda (x) (println x) (* x x)) 
=> (lambda (x) (println x) (* x x))
```

Lambdas can be directly evaluated:
```common-lisp
user> ((lambda (x) (* x x)) 7)
=> 49
user> ((lambda (x) (println x) (* x x)) 7)
7
=> 49
```

Note that only the last s-expression is returned.

### `define`

Binds a symbol to a value.

Basic syntax:
```common-lisp
user> (define x 7)
=> x
user> x
=> 7
```

Creating a function:
```common-lisp
user> (define square (lambda (x) (* x x)))
=> square
user> (square 7)
=> 49
```

Since this is a frequent action, there is special syntax for this:
```common-lisp
user> (define (square x) (* x x))
=> square
user> (square 7)
=> 49
```

You can choose to get the arguments as a list instead:
```common-lisp
user> (define (x . a) a)
=> x
user> (x 1 2 3)
=> (1 2 3)
```

Or have one (or more) required arguments, and get the rest as a list:

```common-lisp
user> (define (x a . b) (println a) (println b))
=> x
user> (x 1 2 3)
1
(2 3)
=> "(2 3)"
```

(list is a function from the standard library that constructs a list from all of its arguments)

### `defmacro`

Macros work the same way as function, except that the arguments to macros are not evaluated.

Macros use the following syntax:
```common-lisp
(defmacro (name arg...) body...)
```

For example, consider the following macro:
```common-lisp
(defmacro (ignore x)
   (cons 'quote (cons x nil)))
```

If we then evaluate the expression
```common-lisp
user> (ignore foo) 
=> foo
```
where foo is a (potentially unbound) symbol, the body of `ignore` will be evaluated with the argument `x` bound to the *unevaluated* symbol `foo`.
The result of this is:
```common-lisp
(quote . (foo . nil))
```
which is equivalent to:
```common-lisp
(quote foo)
```
or
```common-lisp
'foo
```

Finally, evaluating this value will give us the result of evaluating the macro body:
```common-lisp
foo
```

### `if`

The syntax is as follows:
```common-lisp
(if test true-expr false-expr)
```

If `test` is not nil, the result of evaluating this expression will be `false-expr`. Else, it will be `true-expr`.

## Example
This is a simple program that calculates factorials in a recursive fashion:
```common-lisp
(define (factorial x)
   (if (= x 0)
      1
      (* x (factorial (- x 1)))))
```

The base case, if `x=0`, will return `1`.
In all other cases, we will return `fact(x - 1) * x`.

```common-lisp
user> (factorial 10) 
=> 3628800
```
