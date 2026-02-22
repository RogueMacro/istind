# claks

A hobby C compiler written in Rust.

## About

claks is a personal project exploring compiler construction from scratch. It covers the full compilation pipeline:

- **Lexer** – tokenizes C source text into a stream of tokens
- **Parser / AST** – parses tokens into an abstract syntax tree
- **IR** – lowers the AST into an architecture-neutral intermediate representation
- **Backend** – emits native machine code (currently targeting AArch64/macOS)

