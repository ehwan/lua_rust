# lua_rust
lua syntax parser in Rust

 - Greatly in progress
 - LALR(1), GLR parser

## project structure
 - `tokenizer` - tokenizing lua code string
 - `parser` - parsing tokenized lua code into AST
 - `exec` - executable version of the `parser`


## how to run
```
$ cargo run <source_file.lua>
```