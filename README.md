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