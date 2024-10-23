# lua_rust
lua syntax parser & interpreter in Rust

 - Greatly in progress
 - LALR(1), GLR parser
 - syntax referenced from [lua 5.4 reference manual](https://www.lua.org/manual/5.4/manual.html)

## project structure
 - `tokenizer`: tokenizing lua code string.
 - `parser`: parsing tokenized lua code into AST.
 - `semantics`: semantic analysis of generated AST. It generates a `Enhanced AST` which contains more information than the original AST.
      - stack offset of local variables
      - scope checking for `return`, `break`, `goto`, `label`, ...
      - split function definition into separated Chunks
 - `lua_ir` : generate IRs from enhanced AST, provide VM interface for running IRs (WIP)

## Cargo Features
 - `32bit`: use 32bit integer and float for `lua numeric` type
 - `diag`: enable `to_diag()` function for `ParseError`


## how to run
Simply running
```
$ cargo run <source_file.lua>
```
or
```
$ cargo run
```
will start lua interpreter.

No command line arguments are supported yet, some(many) of std functions are not implemented yet.