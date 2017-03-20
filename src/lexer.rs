use std::str::Chars;

#[derive(PartialEq, Debug)]
pub struct ExtendedToken {
    pub token: Token,
    pub index: usize,
    pub len: usize,
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
    Keyword(String),
}

#[derive(Debug)]
struct Context<'a> {
    cs: Chars<'a>,
    read_ahead: String,
    index: usize,
}

#[derive(Debug)]
pub struct LexerError(String);

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

    pub fn tokenize(&mut self) -> Result<Vec<ExtendedToken>, LexerError> {
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
                tokens.push(ExtendedToken::new(Token::Keyword(String::from("+")), pos_before_consume, 1));
            }
            else if c == '-' {
                tokens.push(ExtendedToken::new(Token::Keyword(String::from("-")), pos_before_consume, 1));
            }
            else if c == '*' {
                tokens.push(ExtendedToken::new(Token::Keyword(String::from("*")), pos_before_consume, 1));
            }
            else if c == '/' {
                if let Some(c) = self.ctx.next() {
                    if c == '=' {
                        tokens.push(ExtendedToken::new(Token::Keyword(String::from("/=")), pos_before_consume, 2));
                    }
                    else {
                        tokens.push(ExtendedToken::new(Token::Keyword(String::from("/")), pos_before_consume, 1));
                        self.ctx.return_char(c);
                    }
                }
                else {
                    tokens.push(ExtendedToken::new(Token::Keyword(String::from("/")), pos_before_consume, 1));
                }
            }
            else if c == '=' {
                tokens.push(ExtendedToken::new(Token::Keyword(String::from("=")), pos_before_consume, 1));
            }
            else if c == '>' {
                if let Some(c) = self.ctx.next() {
                    if c == '=' {
                        tokens.push(ExtendedToken::new(Token::Keyword(String::from(">=")), pos_before_consume, 2));
                    }
                    else {
                        tokens.push(ExtendedToken::new(Token::Keyword(String::from(">")), pos_before_consume, 1));
                        self.ctx.return_char(c);
                    }
                }
                else {
                    tokens.push(ExtendedToken::new(Token::Keyword(String::from(">")), pos_before_consume, 1));
                }
            }
            else if c == '<' {
                if let Some(c) = self.ctx.next() {
                    if c == '=' {
                        tokens.push(ExtendedToken::new(Token::Keyword(String::from("<=")), pos_before_consume, 2));
                    }
                    else {
                        tokens.push(ExtendedToken::new(Token::Keyword(String::from("<")), pos_before_consume, 1));
                        self.ctx.return_char(c);
                    }
                }
                else {
                    tokens.push(ExtendedToken::new(Token::Keyword(String::from("<")), pos_before_consume, 1));
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
                return Err(LexerError(format!("Unexpected charactor: [{}] ({:?})", c, self.ctx)));
            }
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize() {
        assert_eq!(
            vec!(ExtendedToken::new(Token::LParen, 0, 1)),
            Lexer::new("(").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::LParen, 2, 1)),
            Lexer::new("  (   ").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::RParen, 0, 1)),
            Lexer::new(")").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::RParen, 2, 1)),
            Lexer::new("  )   ").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Quote, 0, 1)),
            Lexer::new("'").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Integer(1234), 0, 4)),
            Lexer::new("1234").tokenize().unwrap());

        assert_eq!(
            vec!(
                ExtendedToken::new(Token::LParen, 0, 1),
                ExtendedToken::new(Token::Integer(0), 1, 1),
                ExtendedToken::new(Token::RParen, 2, 1)
            ),
            Lexer::new("(0)").tokenize().unwrap());

        assert_eq!(
            vec!(
                ExtendedToken::new(Token::Quote , 0, 1),
                ExtendedToken::new(Token::LParen, 1, 1),
                ExtendedToken::new(Token::Integer(0), 2, 1),
                ExtendedToken::new(Token::RParen, 3, 1)
            ),
            Lexer::new("'(0)").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Keyword(String::from("defun")), 0, 5)),
            Lexer::new("defun").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Keyword(String::from("+")), 0, 1)),
            Lexer::new("+").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Keyword(String::from("-")), 0, 1)),
            Lexer::new("-").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Keyword(String::from("*")), 0, 1)),
            Lexer::new("*").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Keyword(String::from("/")), 0, 1)),
            Lexer::new("/").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Keyword(String::from("=")), 0, 1)),
            Lexer::new("=").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Keyword(String::from("/=")), 0, 2)),
            Lexer::new("/=").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Keyword(String::from(">")), 0, 1)),
            Lexer::new(">").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Keyword(String::from(">=")), 0, 2)),
            Lexer::new(">=").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Keyword(String::from("<")), 0, 1)),
            Lexer::new("<").tokenize().unwrap());

        assert_eq!(
            vec!(ExtendedToken::new(Token::Keyword(String::from("<=")), 0, 2)),
            Lexer::new("<=").tokenize().unwrap());
    }
}
