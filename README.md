# expr-parse

A simple to follow "template" recursive descent expression parser project, can be adapted to any kind of grammar.

# Grammar

This adaptation of the parser parses this grammar, using PEG notation:
```
program   <- stmt*

stmt      <- let_stmt
          / expr

func_decl <- 'fn' ident '(' (ident (',' ident)*)? ')' '{' stmt* '}'
struct    <- 'struct' ident '{' (ident ':' type (',' ident ':' type)*)? '}'
let_stmt  <- 'let' ident '=' expr

expr      <- term ([+-] term)*
term      <- factor ([*/] postfix)*
postfix   <- primary ('.' ident)*
primary   <- number
           / string
           / call
           / ident
           / list
           / '(' expr ')'
call      <- ident '(' (expr (',' expr)*)? ')'
list      <- '[' (expr (',' expr)*)? ']'
```