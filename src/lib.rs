pub mod lexer;
pub mod parser;
pub mod eval;

use std::rc::Rc;
use lexer::{Lexer, LexerError};
use parser::{Node, Parser};
use eval::{Env, Eval, EvalError};

#[derive(Debug)]
pub enum LispError {
    Lexer(LexerError),
    Eval(EvalError),
    EOF,
}

impl From<LexerError> for LispError {
    fn from(err: LexerError) -> Self {
        LispError::Lexer(err)
    }
}

impl From<EvalError> for LispError {
    fn from(err: EvalError) -> Self {
        LispError::Eval(err)
    }
}

pub struct Lisp {
    eval: Eval,
    env: Env
}

impl Lisp {
    pub fn new() -> Self {
        Lisp {
            eval: Eval::new(),
            env: Env::new(),
        }
    }

    pub fn eval_line(&mut self, line: &str) -> Result<Node, LispError> {
        let tokens = try!(Lexer::new(line).tokenize());
        match Parser::new(tokens).parse() {
            Some(nodes) => Ok({
                let nd : Rc<Node> = try!(self.eval.eval(&mut self.env, nodes));
                (*nd.clone()).clone()
            }),
            None => Err(LispError::EOF),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn if_then_else() {
        {
            let mut env = Env::new();
            let tokens = Lexer::new("(if (= 7 7) 42 99)").tokenize().unwrap();
            let nodes = Parser::new(tokens).parse().unwrap();
            assert_eq!(
                Node::Integer(42),
                *Eval::new().eval(&mut env, nodes).unwrap()
            )
        }

        {
            let mut env = Env::new();
            let tokens = Lexer::new("(if (= 7 13) 42 99)").tokenize().unwrap();
            let nodes = Parser::new(tokens).parse().unwrap();
            assert_eq!(
                Node::Integer(99),
                *Eval::new().eval(&mut env, nodes).unwrap()
            )
        }
    }

    #[test]
    fn fib() {
        let mut env = Env::new();
        {
            let tokens = Lexer::new(
                "(setq fib (lambda (n) (if (= n 1) 1 (if (= n 0) 1 (+ (fib (- n 1)) (fib (- n 2)))))))").
                tokenize().unwrap();
            let nodes = Parser::new(tokens).parse().unwrap();
            Eval::new().eval(&mut env, nodes).unwrap();
        }
        {
            let tokens = Lexer::new("(fib 7)").tokenize().unwrap();
            let nodes = Parser::new(tokens).parse().unwrap();
            assert_eq!(
                Node::Integer(21),
                *Eval::new().eval(&mut env, nodes).unwrap()
            );
        }
    }

    #[test]
    fn recursive() {
        let mut env = Env::new();
        {
            let tokens = Lexer::new(
                "(setq rec (lambda (n x) (if (<= n 0) x (rec (- n 1) (* x 2)))))").
                tokenize().unwrap();
            let nodes = Parser::new(tokens).parse().unwrap();
            Eval::new().eval(&mut env, nodes).unwrap();
        }
        {
            let tokens = Lexer::new("(rec 4 3)").tokenize().unwrap();
            let nodes = Parser::new(tokens).parse().unwrap();
            assert_eq!(
                Node::Integer(48),
                *Eval::new().eval(&mut env, nodes).unwrap()
            );
        }
    }
}
