/// For the lexer, I will be going with a very simple solution.
/// Using Rust's ability of allowing enums to hold values.

#[derive(Debug, Clone, PartialEq)]
enum Token {
    LParen,     /// We don't need to hold a value for tokens alike, their enum names tells us everthing we need
    RParen,
    LBrace,
    RBrace,
    LCurl,
    RCurl,
    Plus,
    Minus,
    Mul,
    Div,
    Lt,
    Gt,
    LtEq,
    GtEq,
    EqEq,
    NotEq,
    Comma,
    Colon,
    Let,
    Fn,
    Eq,
    Dot,
    Struct,
    If,
    Else,
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
            b'{' => { self.advance(); Some(Token::LCurl) }
            b'}' => { self.advance(); Some(Token::RCurl) }
            b'+' => { self.advance(); Some(Token::Plus) }
            b'-' => { self.advance(); Some(Token::Minus) }
            b'*' => { self.advance(); Some(Token::Mul) }
            b'/' => { self.advance(); Some(Token::Div) }
            b',' => { self.advance(); Some(Token::Comma) }
            b':' => { self.advance(); Some(Token::Colon) }
            b'.' => { self.advance(); Some(Token::Dot) }
            b'=' => {
                self.advance();
                if self.not_eof() && self.current() == b'=' {
                    self.advance();
                    Some(Token::EqEq)
                } else {
                    Some(Token::Eq)
                }
            }
            b'!' => {
                self.advance();
                if self.not_eof() && self.current() == b'=' {
                    self.advance();
                    Some(Token::NotEq)
                } else {
                    None
                }
            }
            b'<' => {
                self.advance();
                if self.not_eof() && self.current() == b'=' {
                    self.advance();
                    Some(Token::LtEq)
                } else {
                    Some(Token::Lt)
                }
            }
            b'>' => {
                self.advance();
                if self.not_eof() && self.current() == b'=' {
                    self.advance();
                    Some(Token::GtEq)
                } else {
                    Some(Token::Gt)
                }
            }
            _ => {
                if char::is_alphabetic(cur as char) {
                    let start = self.pos;
                    while self.not_eof()
                        && char::is_alphanumeric(self.current() as char)
                    {
                        self.advance();
                    }

                    return Some(match self.source[start..self.pos].as_ref() {
                        "let"       => Token::Let,
                        "fn"        => Token::Fn,
                        "struct"    => Token::Struct,
                        "if"        => Token::If,
                        "else"      => Token::Else,
                        s           => Token::Ident(s.to_string())
                    });
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

    fn advance(&mut self) {
        if self.not_eof() { self.pos += 1; }
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

/// An enum for binary operations
#[derive(Debug, PartialEq, Clone)]
enum BinOps {
    Add, Sub, Mul, Div,
    Lt, Gt, LtEq, GtEq, EqEq, NotEq,
}

/// Here is a simple AST enum variant.
#[derive(Debug)]
#[derive(PartialEq)]
enum Expr {
    Number(f64),
    String(String),
    Ident(String),
    List(Vec<Expr>),
    BinOp { op: BinOps, lhs: Box<Expr>, rhs: Box<Expr> },
    Call { callee: String, args: Vec<Expr> },
    FieldAccess { receiver: Box<Expr>, field: String },
    If {
        cond: Box<Expr>,
        then: Vec<Stmt>,
        else_: Option<Vec<Stmt>>,
    },
    Lambda {
        params: Option<Vec<Expr>>,
        value: Box<Expr>
    }
}

#[derive(Debug, PartialEq)]
struct Field {
    name: String,
    ty: String,
}

#[derive(Debug, PartialEq)]
enum Stmt {
    Expr(Expr),
    Let {
        name: String,
        value: Expr,
    },
    FnDecl {
        name: String,
        params: Vec<Expr>,
        body: Vec<Stmt>,
    },
    Struct {
        name: String,
        fields: Vec<Field>
    }
}

#[derive(Debug, PartialEq)]
struct Program {
    stmts: Vec<Stmt>
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

    /// The main parser driver function, it parses all statements
    /// and returns a program.
    pub fn parse(&mut self) -> Result<Program, ParseErr> {
        let mut stmts: Vec<Stmt> = vec![];
        while self.current.is_some() {
            stmts.push(self.parse_stmt()?);
        }

        Ok(Program { stmts })
    }

    /// Now we're adding the ability to parse statements
    /// Expr statements are just expressions as statements, so wrap parse_expr.
    fn parse_stmt(&mut self) -> Result<Stmt, ParseErr> {
        match self.current {
            Some(Token::Let) => self.parse_let(),
            Some(Token::Fn) => self.parse_fn_decl(),
            Some(Token::Struct) => self.parse_struct(),
            _ => Ok(Stmt::Expr(self.parse_expr()?)),
        }
    }

    /// This is the first parse function of the parser,
    /// Grammar being parsed: expr        <- comparison
    fn parse_expr(&mut self) -> Result<Expr, ParseErr> {
        self.parse_comparison()
    }

    /// This is the second parse function of the parser,
    /// Grammar being parsed: term    <- factor   (_ [*/] _ factor)*
    fn parse_comparison(&mut self) -> Result<Expr, ParseErr> {
        let mut lhs = self.parse_additive()?;

        while let Some(t) = &self.current {
            let op = match t {
                Token::Lt    => BinOps::Lt,
                Token::Gt    => BinOps::Gt,
                Token::LtEq  => BinOps::LtEq,
                Token::GtEq  => BinOps::GtEq,
                Token::EqEq  => BinOps::EqEq,
                Token::NotEq => BinOps::NotEq,
                _ => break,
            };

            self.consume();
            let rhs = self.parse_term()?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }

        Ok(lhs)
    }

    /// This is the third parse function of the parser,
    /// Gramamr being parsed: <- term ([+-] term)*
    fn parse_additive(&mut self) -> Result<Expr, ParseErr> {
        let mut lhs = self.parse_term()?;

        while let Some(t) = &self.current {
            let op = match t {
                Token::Plus => BinOps::Add,
                Token::Minus => BinOps::Sub,
                _ => break
            };

            self.consume();
            let rhs = self.parse_term()?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }

        Ok(lhs)
    }

    /// This is the third parse function of the parser,
    /// Grammar being parsed: term    <- factor   (_ [*/] _ factor)*
    fn parse_term(&mut self) -> Result<Expr, ParseErr> {
        // the reason why we're calling parse_factor her is to get the left side
        // postfix needs to wrap that result before */
        // the chain of parsing looks like this: factor -> postfix -> term
        // term is using postfix expression as lhs
        let primary = self.parse_factor()?;
        let mut lhs = self.parse_postfix(primary)?;

        while let Some(t) = &self.current {
            // Because we're parsing an expression,
            // We want to find * or /
            let op = match t {
                Token::Mul => BinOps::Mul,
                Token::Div => BinOps::Div,
                _ => break,
            };

            self.consume();
            let factor = self.parse_factor()?;
            let rhs = self.parse_postfix(factor)?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }

        Ok(lhs)
    }

    /// This is the fourth parse function of the parser,
    /// Grammar being parsed: factor  <- '(' _ expr _ ')' / number
    fn parse_factor(&mut self) -> Result<Expr, ParseErr> {
        match self.consume() {
            Some(Token::Number(n))  => Ok(Expr::Number(n)),
            Some(Token::String(s))  => Ok(Expr::String(s)),
            Some(Token::LBrace)     => Ok(self.parse_list()?),
            Some(Token::If)         => Ok(self.parse_if()?),
            Some(Token::Ident(s))   => {
                if matches!(self.peek(), Some(Token::LParen)) {
                    self.parse_call(s)
                } else {
                    Ok(Expr::Ident(s))
                }
            },
            Some(Token::LParen)     => {
                // Here we're parsing: '(' _ expr _ ')'
                let expr = self.parse_expr()?; // this is what makes this recursive descent
                self.expect(Token::RParen, "expected ')'")?;
                Ok(expr)
            },
            Some(Token::Fn)         => {
                self.expect(Token::LParen, "expected '(' after `fn` to create an anonymous function")?;
                let params = if self.peek() == Some(&Token::RParen) {
                    self.consume();
                    None
                } else {
                    Some(self.parse_list_of_exprs(Token::RParen)?)
                };

                self.expect(Token::RParen, "expected ')' after params")?;
                self.expect(Token::LCurl, "expected '{' after function params")?;
                
                let value = self.parse_expr()?;
                
                self.expect(Token::RCurl, "expected '}' to close function body")?;
                
                Ok(Expr::Lambda {
                    params,
                    value: Box::new(value)
                })
            },
            Some(t) => Err(ParseErr::UnexpectedToken(t)),
            None    => Err(ParseErr::UnexpectedEof),
        }
    }

    /// This is the fifth parse function of the parser,
    /// It handles postfix operations on a primary expr
    /// Grammar: postfix <- primary ('.' ident)*
    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, ParseErr> {
        while matches!(self.peek(), Some(Token::Dot)) {
            self.consume();
            let field = self.get_identifier_value()?;
            expr = Expr::FieldAccess { receiver: Box::new(expr), field };
        }

        Ok(expr)
    }

    /// Parse a function call
    /// Grammar being parsed: call  <- ident '(' (_ expr ',' _)* ')'
    fn parse_call(&mut self, callee: String) -> Result<Expr, ParseErr> {
        self.expect(Token::LParen, "expected '('")?;

        let args = self.parse_list_of_exprs(Token::RParen)?;

        // Expect the closing parenthesis
        self.expect(Token::RParen, "expected ')'")?;

        Ok(Expr::Call { callee, args })
    }

    /// Parse a list
    /// Grammar being parsed: list  <- '[' (_ expr ',' _)* ']'
    fn parse_list(&mut self) -> Result<Expr, ParseErr> {
        let elements = self.parse_list_of_exprs(Token::RBrace)?;
        self.expect(Token::RBrace, "expected ']'")?;

        Ok(Expr::List(elements))
    }

    /// Parse an if expression
    /// Grammar being parsed: if <- 'if' expr '{' stmt* '}' ('else' '{' stmt* '}')?
    fn parse_if(&mut self) -> Result<Expr, ParseErr> {
        // if is already consumed so we dont need to expect it here
        let cond = self.parse_expr()?;
        let then = self.parse_block()?;
        let else_ = if matches!(self.current, Some(Token::Else)) {
            self.consume();
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Expr::If {
            cond: Box::new(cond),
            then,
            else_
        })
    }

    /// Parse a let decl
    /// Grammar: let    <- 'let' ident '=' expr
    fn parse_let(&mut self) -> Result<Stmt, ParseErr> {
        self.expect(Token::Let, "expected 'let'")?;

        // the next token would be an identifier, so expect it.
        let name = self.get_identifier_value()?;

        // expect '=' after let name.
        self.expect(Token::Eq, "expected '=' after name")?;

        // now simply parse an expression for the value
        let value = self.parse_expr()?;

        Ok(Stmt::Let {
            name,
            value
        })
    }

    /// Parse a function declaration and body
    /// Grammar: func_decl <- 'fn' ident '(' (ident (',' ident)*)? ')' '{' stmt* '}'
    fn parse_fn_decl(&mut self) -> Result<Stmt, ParseErr> {
        self.expect(Token::Fn, "expected 'fn'")?;

        // the next token would be an identifier, so expect it.
        let name = self.get_identifier_value()?;

        self.expect(Token::LParen, "expected '(' after name")?;

        let params = self.parse_list_of_exprs(Token::RParen)?;

        // expect the closing parenthesis
        self.expect(Token::RParen, "expected ')'")?;

        let body = self.parse_block()?;

        Ok(Stmt::FnDecl {
            name,
            params,
            body,
        })
    }

    /// Parser a structure declaration
    /// Grammar: struct <- 'struct' ident '{' (ident ':' type (',' ident ':' type)*)? '}'
    fn parse_struct(&mut self) -> Result<Stmt, ParseErr> {
        self.consume();

        let name = self.get_identifier_value()?;

        self.expect(Token::LCurl, "expected '{' after struct name")?;

        let mut fields = vec![];

        loop {
            if self.current.as_ref() == Some(&Token::RCurl) { break; }

            let name = self.get_identifier_value()?;
            self.expect(Token::Colon, "expected ':' after field name")?;
            let ty = self.get_identifier_value()?;

            fields.push(Field { name, ty });

            if self.current.as_ref() == Some(&Token::RCurl) { break; }
            self.expect(Token::Comma, "expected ','")?;
        }

        self.expect(Token::RCurl, "expected '}'")?;

        Ok(Stmt::Struct { name, fields })
    }

    fn parse_list_of_exprs(&mut self, delim: Token) -> Result<Vec<Expr>, ParseErr> {
        let mut exprs: Vec<Expr> = vec![];

        if self.current.as_ref() == Some(&delim) {
            return Ok(exprs);
        }

        loop {
            exprs.push(self.parse_expr()?);

            if self.current.as_ref() == Some(&delim) {
                break;
            }

            self.expect(Token::Comma, "expected ','")?;
        }

        Ok(exprs)
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseErr> {
        self.expect(Token::LCurl, "expected '{'")?;

        let mut stmts = vec![];
        // So while we're not matching against a '}' currently...
        while !matches!(self.current, Some(Token::RCurl)) {
            if self.current.is_none() {
                return Err(ParseErr::UnexpectedEof);
            }

            stmts.push(self.parse_stmt()?);
        }

        self.expect(Token::RCurl, "expected '}'")?;

        Ok(stmts)
    }

    fn get_identifier_value(&mut self) -> Result<String, ParseErr> {
        match self.consume() {
            Some(Token::Ident(s)) => Ok(s),
            Some(t) => Err(ParseErr::UnexpectedToken(t)),
            None => Err(ParseErr::UnexpectedEof),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.current.as_ref()
    }
}

fn main() {
    // now lets test it!!
    let source = r#"let x = fn(a) { if a == 1 { a } else { 2 } }"#;
    println!("{source}");
    let mut parser = Parser::new(source);
    let program = parser.parse().unwrap();
    println!("{:#?}", program.stmts);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Program {
        Parser::new(src).parse().unwrap()
    }

    fn expr(src: &str) -> Expr {
        match parse(src).stmts.remove(0) {
            Stmt::Expr(e) => e,
            s => panic!("expected expr stmt, got {s:?}"),
        }
    }

    #[test]
    fn test_symbols() {
        let tokens = Lexer::new("( ) [ ] { } + - * / , : . = == != < > <= >=").all();
        assert_eq!(tokens, vec![
            Token::LParen, Token::RParen, Token::LBrace, Token::RBrace,
            Token::LCurl,  Token::RCurl,  Token::Plus,   Token::Minus,
            Token::Mul,    Token::Div,    Token::Comma,  Token::Colon,
            Token::Dot,    Token::Eq,     Token::EqEq,   Token::NotEq,
            Token::Lt,     Token::Gt,     Token::LtEq,   Token::GtEq,
        ]);
    }

    #[test]
    fn test_keywords() {
        assert_eq!(Lexer::new("let fn struct").all(), vec![Token::Let, Token::Fn, Token::Struct]);
    }

    #[test]
    fn test_ident() {
        assert_eq!(Lexer::new("foo").all(), vec![Token::Ident("foo".into())]);
    }

    #[test]
    fn test_number_int() {
        assert_eq!(Lexer::new("42").all(), vec![Token::Number(42.0)]);
    }

    #[test]
    fn test_number_float() {
        assert_eq!(Lexer::new("3.14").all(), vec![Token::Number(3.14)]);
    }

    #[test]
    fn test_string() {
        assert_eq!(Lexer::new(r#""hello""#).all(), vec![Token::String("hello".into())]);
    }

    #[test]
    fn test_string_escape() {
        assert_eq!(Lexer::new(r#""a\"b""#).all(), vec![Token::String(r#"a\"b"#.into())]);
    }

    #[test]
    fn parse_number() {
        assert_eq!(expr("1"), Expr::Number(1.0));
    }

    #[test]
    fn parse_string() {
        assert_eq!(expr(r#""hi""#), Expr::String("hi".into()));
    }

    #[test]
    fn parse_ident() {
        assert_eq!(expr("x"), Expr::Ident("x".into()));
    }

    #[test]
    fn parse_binop_add() {
        assert_eq!(expr("1 + 2"), Expr::BinOp {
            op: BinOps::Add,
            lhs: Box::new(Expr::Number(1.0)),
            rhs: Box::new(Expr::Number(2.0)),
        });
    }

    #[test]
    fn parse_binop_precedence() {
        assert_eq!(expr("1 + 2 * 3"), Expr::BinOp {
            op: BinOps::Add,
            lhs: Box::new(Expr::Number(1.0)),
            rhs: Box::new(Expr::BinOp {
                op: BinOps::Mul,
                lhs: Box::new(Expr::Number(2.0)),
                rhs: Box::new(Expr::Number(3.0)),
            }),
        });
    }

    #[test]
    fn parse_grouped() {
        assert_eq!(expr("(1 + 2) * 3"), Expr::BinOp {
            op: BinOps::Mul,
            lhs: Box::new(Expr::BinOp {
                op: BinOps::Add,
                lhs: Box::new(Expr::Number(1.0)),
                rhs: Box::new(Expr::Number(2.0)),
            }),
            rhs: Box::new(Expr::Number(3.0)),
        });
    }

    #[test]
    fn parse_comparison() {
        assert_eq!(expr("a == b"), Expr::BinOp {
            op: BinOps::EqEq,
            lhs: Box::new(Expr::Ident("a".into())),
            rhs: Box::new(Expr::Ident("b".into())),
        });
    }

    #[test]
    fn parse_list() {
        assert_eq!(expr("[1, 2, 3]"), Expr::List(vec![
            Expr::Number(1.0), Expr::Number(2.0), Expr::Number(3.0),
        ]));
    }

    #[test]
    fn parse_call_no_args() {
        assert_eq!(expr("foo()"), Expr::Call { callee: "foo".into(), args: vec![] });
    }

    #[test]
    fn parse_call_args() {
        assert_eq!(expr("foo(1, 2)"), Expr::Call {
            callee: "foo".into(),
            args: vec![Expr::Number(1.0), Expr::Number(2.0)],
        });
    }

    #[test]
    fn parse_field_access() {
        assert_eq!(expr("a.b"), Expr::FieldAccess {
            receiver: Box::new(Expr::Ident("a".into())),
            field: "b".into(),
        });
    }

    #[test]
    fn parse_field_access_chain() {
        assert_eq!(expr("a.b.c"), Expr::FieldAccess {
            receiver: Box::new(Expr::FieldAccess {
                receiver: Box::new(Expr::Ident("a".into())),
                field: "b".into(),
            }),
            field: "c".into(),
        });
    }

    #[test]
    fn parse_let() {
        assert_eq!(parse("let x = 1").stmts, vec![Stmt::Let {
            name: "x".into(),
            value: Expr::Number(1.0),
        }]);
    }

    #[test]
    fn parse_fn_no_params() {
        assert_eq!(parse("fn f() {}").stmts, vec![Stmt::FnDecl {
            name: "f".into(),
            params: vec![],
            body: vec![],
        }]);
    }

    #[test]
    fn parse_fn_with_body() {
        assert_eq!(parse("fn f() { let x = 1 }").stmts, vec![Stmt::FnDecl {
            name: "f".into(),
            params: vec![],
            body: vec![Stmt::Let { name: "x".into(), value: Expr::Number(1.0) }],
        }]);
    }

    #[test]
    fn parse_struct_empty() {
        assert_eq!(parse("struct A {}").stmts, vec![Stmt::Struct {
            name: "A".into(),
            fields: vec![],
        }]);
    }

    #[test]
    fn parse_struct_fields() {
        assert_eq!(parse("struct A { x: int, y: int }").stmts, vec![Stmt::Struct {
            name: "A".into(),
            fields: vec![
                Field { name: "x".into(), ty: "int".into() },
                Field { name: "y".into(), ty: "int".into() },
            ],
        }]);
    }

    #[test]
    fn parse_if_no_else() {
        assert_eq!(expr("if x { let y = 1 }"), Expr::If {
            cond: Box::new(Expr::Ident("x".into())),
            then: vec![Stmt::Let { name: "y".into(), value: Expr::Number(1.0) }],
            else_: None,
        });
    }

    #[test]
    fn parse_if_else() {
        assert_eq!(expr("if x { let y = 1 } else { let y = 2 }"), Expr::If {
            cond: Box::new(Expr::Ident("x".into())),
            then: vec![Stmt::Let { name: "y".into(), value: Expr::Number(1.0) }],
            else_: Some(vec![Stmt::Let { name: "y".into(), value: Expr::Number(2.0) }]),
        });
    }

    #[test]
    fn parse_if_as_value() {
        assert_eq!(parse("let x = if a { let y = 1 } else { let y = 2 }").stmts, vec![
            Stmt::Let {
                name: "x".into(),
                value: Expr::If {
                    cond: Box::new(Expr::Ident("a".into())),
                    then: vec![Stmt::Let { name: "y".into(), value: Expr::Number(1.0) }],
                    else_: Some(vec![Stmt::Let { name: "y".into(), value: Expr::Number(2.0) }]),
                },
            }
        ]);
    }

    #[test]
    fn parse_struct_trailing_comma() {
        assert!(Parser::new("struct A { x: int, }").parse().is_ok());
    }

    #[test]
    fn parse_unexpected_eof() {
        assert!(matches!(Parser::new("let x =").parse(), Err(ParseErr::UnexpectedEof)));
    }

    #[test]
    fn parse_unexpected_token() {
        assert!(matches!(Parser::new("let 123").parse(), Err(ParseErr::UnexpectedToken(_))));
    }
}