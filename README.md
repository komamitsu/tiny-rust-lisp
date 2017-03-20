# tiny-rust-lisp
[<img src="https://travis-ci.org/komamitsu/tiny-rust-lisp.svg?branch=master"/>](https://travis-ci.org/komamitsu/tiny-rust-lisp)

Tiny Lisp implementation written in Rust

## Supported keywords

- `lambda`
- `setq`
- `car`
- `cdr`
- `if`
- `+`
- `-`
- `*`
- `/`
- `=`
- `/=`
- `<`
- `<=`
- `>`
- `>=`

## Usage

```
$ cargo run
> (+ 40 2)
Ok(Integer(42))
> (if (= 1 2) 99 42)
Ok(Integer(42))
> (car '(42 0 99))
Ok(Integer(42))
> (cdr '(0 42))
Ok(QuotedList([Integer(42)]))
> (setq double-rec (lambda (n x) (if (<= n 0) x (double-rec (- n 1) (* x 2)))))
Ok(List([Keyword("setq"), Keyword("double-rec"), ....]))
> (double-rec 5 7)
Ok(Integer(224))
> (setq fib (lambda (n) (if (= n 1) 1 (if (= n 0) 1 (+ (fib (- n 1)) (fib (- n 2)))))))
Ok(List([Keyword("setq"), Keyword("fib"), ...]))
> (fib 20)
Ok(Integer(10946))
```