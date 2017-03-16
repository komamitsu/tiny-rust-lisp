use lexer::*;

#[derive(PartialEq, Debug, Clone)]
pub enum Node {
    Integer(i64),
    Keyword(String),
    List(Vec<Node>),
    QuotedList(Vec<Node>),
    Func(Vec<String>, Vec<Node>),
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

    pub fn parse(&mut self) -> Option<Node> {
        match self.next_token() {
            // Check EOF
            None => None,
            Some(token) => match token.token {
                Token::LParen => Some(Node::List(self.parse_list())),
                Token::RParen => None,
                Token::Integer(i) => Some(Node::Integer(i)),
                Token::Keyword(s) => Some(Node::Keyword(s)),
                Token::Quote => self.parse_quoted_list(),
            }
        }
    }

    fn parse_list(&mut self) -> Vec<Node> {
        let mut list = Vec::new();
        while let Some(node) = self.parse() {
            list.push(node)
        }
        list
    }

    fn parse_quoted_list(&mut self) -> Option<Node> {
        match self.next_token() {
            // Check EOF
            None => None,
            Some(token) => match token.token {
                Token::LParen => {
                    let mut list = Vec::new();
                    while let Some(node) = self.parse() {
                        list.push(node)
                    }
                    Some(Node::QuotedList(list))
                },
                // TODO
                _ => None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use lexer::{ExtendedToken, Token};
    use super::{Node, Parser};

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
                    Node::Keyword(String::from("+")),
                    Node::Integer(1),
                    Node::List(
                        vec![
                            Node::Keyword(String::from("-")),
                            Node::Integer(5),
                            Node::Integer(2),
                        ])]),
            Parser::new(tokens).parse().unwrap()
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
            Node::QuotedList(vec![Node::Integer(1)]),
            Parser::new(tokens).parse().unwrap()
        );
    }
}

