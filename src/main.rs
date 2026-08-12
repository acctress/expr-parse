/// For the lexer, I will be going with a very simple solution.
/// Using Rust's ability of allowing enums to hold values.

#[derive(Debug, Clone, PartialEq)]
enum Token {
    LParen,     /// We don't need to hold a value for tokens alike, their enum names tells us everthing we need
    RParen,
    Plus,
    Minus,
    Mul,
    Div,
    Number(f64),
    Ident(String),
}

/// This lexer will follow a pattern of generating one token at a time, or a vector of tokens.
/// We do need a lifetime annotation since our lexer will be taking a slice of the source code.
/// like a `std::string_view` in C++.
struct Lexer<'a> {
    source: &'a str,
    pos: usize,
}

/// We also need to add the annotations to the impl.
impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, pos: 0 }
    }

    /// Return the next token in the source code.
    pub fn next(&mut self) -> Option<Token> {
        self.skip_ws();

        if !self.not_eof() {
            return None;
        }

        let cur = self.source.as_bytes()[self.pos];
        match cur {
            b'(' => { self.advance(); Some(Token::LParen) }
            b')' => { self.advance(); Some(Token::RParen) }
            b'+' => { self.advance(); Some(Token::Plus) }
            b'-' => { self.advance(); Some(Token::Minus) }
            b'*' => { self.advance(); Some(Token::Mul) }
            b'/' => { self.advance(); Some(Token::Div) }
            _ => {
                if char::is_alphabetic(cur as char) {
                    let start = self.pos;
                    while self.not_eof()
                        && char::is_alphanumeric(self.source.as_bytes()[self.pos] as char)
                    {
                        self.advance();
                    }
                    return Some(Token::Ident(self.source[start..self.pos].to_string()));
                } else if char::is_digit(cur as char, 10) {
                    let start = self.pos;
                    let mut flt = false;
                    while self.not_eof() {
                        let b = self.source.as_bytes()[self.pos];
                        if char::is_digit(b as char, 10) {
                            self.advance();
                        } else if b == b'.' && !flt {
                            flt = true;
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    return Some(Token::Number(self.source[start..self.pos].parse().unwrap()));
                }
                None
            }
        }
    }

    /// Return the whole source code as tokens, in a Vec.
    pub fn all(&mut self) -> Vec<Token> {
        std::iter::from_fn(|| self.next()).collect()
    }

    fn advance(&mut self) {
        if self.pos < self.source.len() { self.pos += 1; }
    }

    fn skip_ws(&mut self) {
        while self.not_eof() && (self.source.as_bytes()[self.pos] as char).is_ascii_whitespace() {
            self.advance();
        }
    }

    fn not_eof(&self) -> bool {
        self.pos < self.source.len()
    }
}

/// Here is a simple AST enum variant.
#[derive(Debug)]
enum Expr {
    Number(f64),
    Ident(String),
    BinOp { op: char, lhs: Box<Expr>, rhs: Box<Expr> },
}

/// It's good to define a custom error enum to handle errors cleanly.
#[derive(Debug)]
enum ParseErr {
    Unexpected { expected: Token, found: Token, msg: &'static str },
    UnexpectedToken(Token),
    UnexpectedEof,
}

/// Now we need to pass the same lifetime annotation through to the parser and lexer
struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Option<Token>
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut p = Self { lexer: Lexer::new(source), current: None };
        p.consume();
        p
    }

    /// Take the current token we have, advance then return the previous token we were at
    fn consume(&mut self) -> Option<Token> {
        let pr = self.current.take();
        self.current = self.lexer.next();
        pr
    }

    /// Expect the current token to match a token of our choice, if it doesn't an error will be returned
    /// If the current token matches, we'll be returned the consumed token and the parser will advance
    fn expect(&mut self, tk: Token, msg: &'static str) -> Result<Token, ParseErr> {
        match &self.current {
            Some(c) if *c == tk => Ok(self.consume().unwrap()),
            Some(c) => Err(ParseErr::Unexpected { expected: tk, found: c.clone(), msg }),
            None => Err(ParseErr::UnexpectedEof)
        }
    }

    /// The main parser driver function, parse the top most parse function.
    /// In this case, the first thing to be parsed in our grammar is expression.
    pub fn parse(&mut self) -> Result<Expr, ParseErr> {
        self.parse_expr()
    }

    /// This is the first parse function of the parser,
    /// Grammar being parsed: expr    <- term     (_ [+-] _ term)*
    fn parse_expr(&mut self) -> Result<Expr, ParseErr> {
        let mut lhs = self.parse_term()?;

        while let Some(t) = &self.current {
            // Because we're parsing an expression,
            // We want to find + or -
            let op = match t {
                Token::Plus => '+',
                Token::Minus => '-',
                _ => break,
            };

            self.consume();
            let rhs = self.parse_term()?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }

        Ok(lhs)
    }

    /// This is the second parse function of the parser,
    /// Grammar being parsed: term    <- factor   (_ [*/] _ factor)*
    fn parse_term(&mut self) -> Result<Expr, ParseErr> {
        let mut lhs = self.parse_factor()?;

        while let Some(t) = &self.current {
            // Because we're parsing an expression,
            // We want to find * or /
            let op = match t {
                Token::Mul => '*',
                Token::Div => '/',
                _ => break,
            };

            self.consume();
            let rhs = self.parse_factor()?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }

        Ok(lhs)
    }

    /// This is the third parse function of the parser,
    /// Grammar being parsed: factor  <- '(' _ expr _ ')' / number
    fn parse_factor(&mut self) -> Result<Expr, ParseErr> {
        match self.consume() {
            Some(Token::Number(n)) => Ok(Expr::Number(n)),
            Some(Token::Ident(s))  => Ok(Expr::Ident(s)),
            Some(Token::LParen) => {
                // Here we're parsing: '(' _ expr _ ')'
                let expr = self.parse_expr()?; // this is what makes this recursive descent
                self.expect(Token::RParen, "expected ')'")?;
                Ok(expr)
            }
            Some(t) => Err(ParseErr::UnexpectedToken(t)),
            None    => Err(ParseErr::UnexpectedEof),
        }
    }
}

fn main() {
    // now lets test it!!
    let source = "1 + 2";
    let mut parser = Parser::new(source);
    let expr = parser.parse();
    println!("{:#?}", expr);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Expr {
        Parser::new(src).parse().unwrap()
    }

    #[test]
    fn test_number() {
        assert!(matches!(parse("42"), Expr::Number(n) if n == 42.0));
    }

    #[test]
    fn test_float() {
        assert!(matches!(parse("23.2354234"), Expr::Number(n) if n == 23.2354234));
    }

    #[test]
    fn test_identifier() {
        assert!(matches!(parse("blinx"), Expr::Ident(s) if s == "blinx"));
    }

    #[test]
    fn test_binop_add() {
        assert!(matches!(parse("1 + 2"), Expr::BinOp { op: '+', .. }));
    }

    #[test]
    fn test_precedence() {
        /// + should be first
        assert!(matches!(parse("1 + 2 * 3"), Expr::BinOp { op: '+', .. }));
    }

    #[test]
    fn test_do_parens_override_precedence_hehehe() {
        /// * should be first
        assert!(matches!(parse("(1 + 2) * 3"), Expr::BinOp { op: '*', .. }));
    }

    #[test]
    fn test_unexpected_token() {
        assert!(Parser::new("1 + ").parse().is_err());
    }

    #[test]
    fn test_missinbg_paren() {
        assert!(Parser::new("(1 + 2").parse().is_err());
    }

}