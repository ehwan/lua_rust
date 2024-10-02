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
 - `lua_ir` : generate IRs from enhanced AST, and run on virtual machine (WIP)

## Cargo Features
 - `32bit`: use 32bit integer and float for `lua numeric` type
 - `diag`: enable `to_diag()` function for `ParseError`


## how to run
```
$ cargo run <source_file.lua>
```

will print the pretty-formatted `Debug` output of the AST ( `"{:#?}"` )