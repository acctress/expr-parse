# expr-parse

A simple recursive descent parser project, can be adapted to any kind of grammar.

# Grammar

This adaptation of the parser parses this grammar, using PEG notation:
```
program     <- stmt*

stmt        <- let_stmt
             / expr

func_decl   <- 'fn' ident '(' (ident (',' ident)*)? ')' '{' stmt* '}'
struct      <- 'struct' ident '{' (ident ':' type (',' ident ':' type)*)? '}'
let_stmt    <- 'let' ident '=' expr

expr         <- comparison
comparison   <- additive ([< > <= >= == !=] additive)*
additive     <- term ([+-] term)*
term         <- postfix ([*/] postfix)*
postfix      <- primary ('.' ident)*
primary      <- number
              / string
              / call
              / ident
              / list
              / '(' expr ')'
              
if          <- 'if' expr '{' stmt* '}' ('else' '{' stmt* '}')?
call        <- ident '(' (expr (',' expr)*)? ')'
list        <- '[' (expr (',' expr)*)? ']'
```

# How Recursive Descent Works
By nature a recursive descent parser is a top down parser built from a set of mutually recursive functions, 
each function implements one of the nonterminals of the grammar.

For example:
```rust
fn parse_expr(&mut self) {
    self.parse_term()
}

fn parse_term(&mut self) {
    self.parse_factor() // this calls back into the recursive chain
}

fn parse_factor(&mut self) {
    match self.current {
        Token::LParen => {
            self.consume();
            self.parse_expr()   // recurse back to the TOP
        }
    }
}
```

Each function in this example is a *nonterminal*, they call each other which is what the "mutually recursive" statement
talks about.

So in my implementation, the grammar is ordered in the correct way a source file would be parsed:
`expr` -> `comparison` -> `additive` -> `term` -> `postfix` -> `primary`.
