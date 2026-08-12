# expr-parse

A simple to follow "template" recursive descent expression parser project, can be adapted to any kind of grammar.

# Grammar

This adaptation of the parser parses this grammar, using PEG notation:
```
program  <- stmt*

stmt     <- let_stmt
          / expr

let_stmt <- 'let' ident '=' expr

expr     <- term ([+-] term)*
term     <- factor ([*/] factor)*
factor   <- number
          / string
          / ident
          / call
          / list
          / '(' expr ')'
```