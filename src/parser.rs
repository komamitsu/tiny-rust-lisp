use std::rc::Rc;
use lexer::*;

#[derive(PartialEq, Debug, Clone)]
pub enum Node {
    Integer(i64),
    Keyword(String),
    List(Vec<Rc<Node>>),
    QuotedList(Vec<Rc<Node>>),
    Func(Vec<String>, Vec<Rc<Node>>),
    True,
    False,
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<ExtendedToken>
}

impl Parser {
    pub fn new(tokens: Vec<ExtendedToken>) -> Self {
        Parser { tokens: tokens }
    }

    fn next_token(&mut self) -> Option<ExtendedToken> {
        if self.tokens.is_empty() {
            None
        }
        else {
            Some(self.tokens.remove(0))
        }
    }

    pub fn parse(&mut self) -> Option<Rc<Node>> {
        match self.next_token() {
            // Check EOF
            None => None,
            Some(token) => match token.token {
                Token::LParen => Some(Rc::new(Node::List(self.parse_list()))),
                Token::RParen => None,
                Token::Integer(i) => Some(Rc::new(Node::Integer(i))),
                Token::Keyword(s) => Some(Rc::new(Node::Keyword(s))),
                Token::Quote => self.parse_quoted_list(),
            }
        }
    }

    fn parse_list(&mut self) -> Vec<Rc<Node>> {
        let mut list = Vec::new();
        while let Some(node) = self.parse() {
            list.push(node)
        }
        list
    }

    fn parse_quoted_list(&mut self) -> Option<Rc<Node>> {
        match self.next_token() {
            // Check EOF
            None => None,
            Some(token) => match token.token {
                Token::LParen => {
                    let mut list = Vec::new();
                    while let Some(node) = self.parse() {
                        list.push(node)
                    }
                    Some(Rc::new(Node::QuotedList(list)))
                },
                // TODO
                _ => None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse0() {
        let tokens = vec![
            ExtendedToken {
                token: Token::LParen, index: 0, len: 1
            },
            ExtendedToken {
                token: Token::Keyword(String::from("+")), index: 1, len: 1
            },
            ExtendedToken {
                token: Token::Integer(1), index: 3, len: 1
            },
            ExtendedToken {
                token: Token::LParen, index: 5, len: 1
            },
            ExtendedToken {
                token: Token::Keyword(String::from("-")), index: 6, len: 1
            },
            ExtendedToken {
                token: Token::Integer(5), index: 8, len: 1
            },
            ExtendedToken {
                token: Token::Integer(2), index: 10, len: 1
            },
            ExtendedToken {
                token: Token::RParen, index: 11, len: 1
            },
            ExtendedToken {
                token: Token::RParen, index: 12, len: 1
            },
        ];
        assert_eq!(
            Node::List(
                vec![
                    Rc::new(Node::Keyword(String::from("+"))),
                    Rc::new(Node::Integer(1)),
                    Rc::new(Node::List(
                        vec![
                            Rc::new(Node::Keyword(String::from("-"))),
                            Rc::new(Node::Integer(5)),
                            Rc::new(Node::Integer(2)),
                        ]))]),
            *Parser::new(tokens).parse().unwrap()
        );
    }

    #[test]
    fn parse1() {
        let tokens = vec![
            ExtendedToken {
                token: Token::Quote, index: 0, len: 1
            },
            ExtendedToken {
                token: Token::LParen, index: 1, len: 1
            },
            ExtendedToken {
                token: Token::Integer(1), index: 2, len: 1
            },
            ExtendedToken {
                token: Token::RParen, index: 3, len: 1
            },
        ];
        assert_eq!(
            Node::QuotedList(vec![Rc::new(Node::Integer(1))]),
            *Parser::new(tokens).parse().unwrap()
        );
    }
}

