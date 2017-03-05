use std::str::Chars;

#[derive(PartialEq, Debug)]
pub struct ExtendedToken {
    token: Token,
    index: usize,
    len: usize,
}

impl ExtendedToken {
    fn new(token: Token, index: usize, len: usize) -> Self {
        ExtendedToken { token: token, index: index, len: len }
    }
}

#[derive(PartialEq, Debug)]
pub enum Token {
    LParen,
    RParen,
    Quote,
    Integer(i64),
    Add,
    Sub,
    Multi,
    Div,
    Eq,
    Not,
    Gt,
    Lt,
    Ge,
    Le,
    Keyword(String),
}

#[derive(Debug)]
struct Context<'a> {
    cs: Chars<'a>,
    read_ahead: String,
    index: usize,
}

impl <'a> Context<'a> {
    pub fn new(s: &'a str) -> Self {
        Context { cs: s.chars(), read_ahead: String::new(), index: 0 }
    }

    pub fn next(&mut self) -> Option<char> {
        let c = 
            if self.read_ahead.is_empty() {
                self.cs.next()
            }
            else {
                Some(self.read_ahead.remove(0))
            };

        self.index += 1;

        c
    }

    pub fn return_char(&mut self, c: char) {
        self.read_ahead.push(c);
        self.index -= 1;
    }

    pub fn pos(&self) -> usize {
        self.index
    }
}

pub struct Lexer<'a> {
    ctx: Context<'a>
}

impl <'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer { ctx: Context::new(input) }
    }

    pub fn tokenize(&mut self) -> Vec<ExtendedToken> {
        let mut tokens = Vec::new();
        while let Some(c) = self.ctx.next() {
            let pos_before_consume = self.ctx.pos() - 1;
            if " \t\n".contains(c) {
            }
            else if c == '(' {
                tokens.push(ExtendedToken::new(Token::LParen, pos_before_consume, 1));
            }
            else if c == ')' {
                tokens.push(ExtendedToken::new(Token::RParen, pos_before_consume, 1));
            }
            else if c == '\'' {
                tokens.push(ExtendedToken::new(Token::Quote, pos_before_consume, 1));
            }
            else if c == '+' {
                tokens.push(ExtendedToken::new(Token::Add, pos_before_consume, 1));
            }
            else if c == '-' {
                tokens.push(ExtendedToken::new(Token::Sub, pos_before_consume, 1));
            }
            else if c == '*' {
                tokens.push(ExtendedToken::new(Token::Multi, pos_before_consume, 1));
            }
            else if c == '/' {
                if let Some(c) = self.ctx.next() {
                    if c == '=' {
                        tokens.push(ExtendedToken::new(Token::Not, pos_before_consume, 2));
                    }
                    else {
                        tokens.push(ExtendedToken::new(Token::Div, pos_before_consume, 1));
                        self.ctx.return_char(c);
                    }
                }
                else {
                    tokens.push(ExtendedToken::new(Token::Div, pos_before_consume, 1));
                }
            }
            else if c == '=' {
                tokens.push(ExtendedToken::new(Token::Eq, pos_before_consume, 1));
            }
            else if c == '>' {
                if let Some(c) = self.ctx.next() {
                    if c == '=' {
                        tokens.push(ExtendedToken::new(Token::Ge, pos_before_consume, 2));
                    }
                    else {
                        tokens.push(ExtendedToken::new(Token::Gt, pos_before_consume, 1));
                        self.ctx.return_char(c);
                    }
                }
                else {
                    tokens.push(ExtendedToken::new(Token::Gt, pos_before_consume, 1));
                }
            }
            else if c == '<' {
                if let Some(c) = self.ctx.next() {
                    if c == '=' {
                        tokens.push(ExtendedToken::new(Token::Le, pos_before_consume, 2));
                    }
                    else {
                        tokens.push(ExtendedToken::new(Token::Lt, pos_before_consume, 1));
                        self.ctx.return_char(c);
                    }
                }
                else {
                    tokens.push(ExtendedToken::new(Token::Lt, pos_before_consume, 1));
                }
            }
            else if c.is_alphanumeric() {
                let mut s = String::new();
                s.push(c);
                while let Some(c) = self.ctx.next() {
                    if c.is_alphanumeric() || c == '-' {
                        s.push(c);
                        continue;
                    }
                    self.ctx.return_char(c);
                    break;
                }
                let len = s.len();
                if s.chars().all(|x| x.is_digit(10)) {
                    tokens.push(
                        ExtendedToken::new(Token::Integer(s.parse().unwrap()), pos_before_consume, len));
                }
                else {
                    tokens.push(ExtendedToken::new(Token::Keyword(s), pos_before_consume, len));
                }
            }
            else {
                panic!("Unexpected charactor: [{}] ({:?})", c, self.ctx);
            }
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, ExtendedToken, Token};

    #[test]
    fn tokenize() {
        assert_eq!(
            vec!(ExtendedToken::new(Token::LParen, 0, 1)),
            Lexer::new("(").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::LParen, 2, 1)),
            Lexer::new("  (   ").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::RParen, 0, 1)),
            Lexer::new(")").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::RParen, 2, 1)),
            Lexer::new("  )   ").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Quote, 0, 1)),
            Lexer::new("'").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Integer(1234), 0, 4)),
            Lexer::new("1234").tokenize());

        assert_eq!(
            vec!(
                ExtendedToken::new(Token::LParen, 0, 1),
                ExtendedToken::new(Token::Integer(0), 1, 1),
                ExtendedToken::new(Token::RParen, 2, 1)
            ),
            Lexer::new("(0)").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Keyword(String::from("defun")), 0, 5)),
            Lexer::new("defun").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Add, 0, 1)),
            Lexer::new("+").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Sub, 0, 1)),
            Lexer::new("-").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Multi, 0, 1)),
            Lexer::new("*").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Div, 0, 1)),
            Lexer::new("/").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Eq, 0, 1)),
            Lexer::new("=").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Not, 0, 2)),
            Lexer::new("/=").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Gt, 0, 1)),
            Lexer::new(">").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Ge, 0, 2)),
            Lexer::new(">=").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Lt, 0, 1)),
            Lexer::new("<").tokenize());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Le, 0, 2)),
            Lexer::new("<=").tokenize());
    }
}
