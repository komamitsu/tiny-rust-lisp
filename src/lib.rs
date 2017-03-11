mod lexer;
mod parser;
mod eval;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use lexer::Lexer;
    use parser::{Node, Parser};
    use eval::Eval;

    #[test]
    fn if_then_else() {
        {
            let mut env = HashMap::new();
            let tokens = Lexer::new("(if (= 7 7) 42 99)").tokenize();
            let nodes = Parser::new(tokens).parse().unwrap();
            assert_eq!(
                Node::Integer(42),
                Eval::new().eval(&mut env, nodes)
            )
        }

        {
            let mut env = HashMap::new();
            let tokens = Lexer::new("(if (= 7 13) 42 99)").tokenize();
            let nodes = Parser::new(tokens).parse().unwrap();
            assert_eq!(
                Node::Integer(99),
                Eval::new().eval(&mut env, nodes)
            )
        }
    }

    #[test]
    fn fib() {
        let mut env = HashMap::new();
        {
            let tokens = Lexer::new(
                "(setq fib (lambda (n) (if (= n 1) 1 (if (= n 0) 1 (+ (fib (- n 1)) (fib (- n 2)))))))").
                tokenize();
            let nodes = Parser::new(tokens).parse().unwrap();
            Eval::new().eval(&mut env, nodes);
        }
        {
            let tokens = Lexer::new("(fib 6)").tokenize();
            let nodes = Parser::new(tokens).parse().unwrap();
            assert_eq!(
                Node::Integer(8),
                Eval::new().eval(&mut env, nodes)
            );
        }
    }
}
