# expr-parse

A simple to follow "template" recursive descent expression parser project, can be adapted to any kind of grammar.

# Grammar

This adaptation of the parser parses this grammar, using PEG notation:
```
expr    <- term     (_ [+-] _ term)*
term    <- factor   (_ [*/] _ factor)*
factor  <- '(' _ expr _ ')' / number
number  <- [0-9]+ ('.' [0-9]+)?
_       <- [ \t\n\r]*
```

Let's expand on `expr    <- term     (_ [+-] _ term)*`, this essentially says: parse a `term`, then zero or more repetitions of: optional whitespace, `+` or `-`, optional whitespace, then another `term`.
