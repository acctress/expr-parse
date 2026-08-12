use crate::Expr::Ident;

/// For the lexer, I will be going with a very simple solution.
/// Using Rust's ability of allowing enums to hold values.

#[derive(Debug, Clone, PartialEq)]
enum Token {
    LParen,     /// We don't need to hold a value for tokens alike, their enum names tells us everthing we need
    RParen,
    LBrace,
    RBrace,
    Plus,
    Minus,
    Mul,
    Div,
    Comma,
    Number(f64),
    Ident(String),
    String(String),
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

        let cur = self.current();
        match cur {
            b'(' => { self.advance(); Some(Token::LParen) }
            b')' => { self.advance(); Some(Token::RParen) }
            b'[' => { self.advance(); Some(Token::LBrace) }
            b']' => { self.advance(); Some(Token::RBrace) }
            b'+' => { self.advance(); Some(Token::Plus) }
            b'-' => { self.advance(); Some(Token::Minus) }
            b'*' => { self.advance(); Some(Token::Mul) }
            b'/' => { self.advance(); Some(Token::Div) }
            b',' => { self.advance(); Some(Token::Comma) }
            _ => {
                if char::is_alphabetic(cur as char) {
                    let start = self.pos;
                    while self.not_eof()
                        && char::is_alphanumeric(self.current() as char)
                    {
                        self.advance();
                    }
                    return Some(Token::Ident(self.source[start..self.pos].to_string()));
                } else if char::is_digit(cur as char, 10) {
                    let start = self.pos;
                    let mut flt = false;
                    while self.not_eof() {
                        let b = self.current();
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
                } else if cur == b'"' {
                    self.advance();
                    let start = self.pos;

                    while self.not_eof() {
                        match self.current() {
                            b'\\' => {
                                self.advance();
                                if self.not_eof() {
                                    self.advance();
                                }
                            }
                            b'"' => break,
                            _ => self.advance(),
                        }
                    }

                    let value = self.source[start..self.pos].to_string();
                    self.advance();

                    return Some(Token::String(value));
                }
                None
            }
        }
    }

    /// Return the whole source code as tokens, in a Vec.
    pub fn all(&mut self) -> Vec<Token> {
        std::iter::from_fn(|| self.next()).collect()
    }

    fn current(&self) -> u8 {
        self.source.as_bytes()[self.pos]
    }

    /// Non consuming next token return
    /// Simply save the previous position, advance, and reset
    pub fn peek(&mut self) -> Option<Token> {
        let prev_pos = self.pos;
        let token = self.next();
        self.pos = prev_pos;
        token
    }

    fn advance(&mut self) {
        if self.pos < self.source.len() { self.pos += 1; }
    }

    fn skip_ws(&mut self) {
        while self.not_eof() && (self.current() as char).is_ascii_whitespace() {
            self.advance();
        }
    }

    fn not_eof(&self) -> bool {
        self.pos < self.source.len()
    }

}

/// Here is a simple AST enum variant.
#[derive(Debug)]
#[derive(PartialEq)]
enum Expr {
    Number(f64),
    String(String),
    Ident(String),
    List(Vec<Expr>),
    BinOp { op: char, lhs: Box<Expr>, rhs: Box<Expr> },
    Call { callee: String, args: Vec<Expr> }
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
            Some(Token::String(s)) => Ok(Expr::String(s)),
            Some(Token::LBrace) => Ok(self.parse_list()?),
            Some(Token::Ident(s))  => {
                if matches!(self.peek(), Some(Token::LParen)) {
                    self.parse_call(s)
                } else {
                    Ok(Expr::Ident(s))
                }
            },
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

    /// Parse a function call
    /// Grammar being parsed: call  <- ident '(' (_ expr ',' _)* ')'
    fn parse_call(&mut self, callee: String) -> Result<Expr, ParseErr> {
        self.expect(Token::LParen, "expected '('")?;

        let mut args: Vec<Expr> = vec![];
        // So while we're not matching against a ')' currently...
        while !matches!(self.current, Some(Token::RParen)) {
            // push args onto the vector
            args.push(self.parse_expr()?);

            // if we are a comma, then consume, else that's the end of the args list.
            if matches!(self.current, Some(Token::Comma)) {
                self.consume();
            } else {
                break;
            }
        }

        // Expect the closing parenthesis
        self.expect(Token::RParen, "expected ')'")?;

        Ok(Expr::Call { callee, args })
    }

    /// Parse a list
    /// Grammar being parsed: list  <- '[' (_ expr ',' _)* ']'
    fn parse_list(&mut self) -> Result<Expr, ParseErr> {
        let mut elements: Vec<Expr> = vec![];
        // So while we're not matching against a ']' currently...
        while !matches!(self.current, Some(Token::RBrace)) {
            elements.push(self.parse_expr()?);

            // if we are a comma, then consume, else that's the end of the list.
            if matches!(self.current, Some(Token::Comma)) {
                self.consume();
            } else {
                break;
            }
        }

        Ok(Expr::List(elements))
    }

    fn peek(&self) -> Option<&Token> {
        self.current.as_ref()
    }
}

/// This simply evals all binary operations
fn eval(expr: Expr) -> f64 {
    match expr {
        Expr::Number(n) => n,
        Expr::BinOp { lhs, rhs, op } => {
            let lhs = eval(*lhs);
            let rhs = eval(*rhs);
            
            match op {
                '+' => lhs + rhs,
                '-' => lhs - rhs,
                '*' => lhs * rhs,
                '/' => lhs / rhs,
                _ => unreachable!("invalid bin op"),
            }
        },
        _ => todo!("eval not implemented for anything else")
    }
}

fn main() {
    // now lets test it!!
    let source = r#"print("hello world")"#;
    let mut parser = Parser::new(source);
    let expr = parser.parse().unwrap();
    println!("{:#?}", expr);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Expr {
        Parser::new(src).parse().unwrap()
    }

    #[test]
    fn test_string() {
        assert!(matches!(parse(r#""hi""#), Expr::String(s) if s == "hi"));
    }

    #[test]
    fn test_list() {
        assert_eq!(
            parse("[1, 2, 3, 4]"),
            Expr::List(vec![
                Expr::Number(1.0),
                Expr::Number(2.0),
                Expr::Number(3.0),
                Expr::Number(4.0),
            ])
        );
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