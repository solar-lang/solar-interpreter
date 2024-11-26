# Solar Compiler

Performs Statical Type checking on raw Solar AST and emits fully linked bytecode

## TODO

### Prio 1

- create a simplified AST Type, that can be derived from the normal AST
- Instead of having a huge compiler context implementation, maybe implement compilation as AST methods.

### Prio 2

- save code-comments in normal AST, so it can be used for formatting.
- include a type of block that is "unparsable" for formatting and language server
