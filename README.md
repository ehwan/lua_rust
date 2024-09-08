# lua_rust
[![crates.io](https://img.shields.io/crates/v/lua_parser.svg)](https://crates.io/crates/lua_parser)
[![docs.rs](https://docs.rs/lua_parser/badge.svg)](https://docs.rs/lua_parser)

lua syntax parser in Rust

 - Greatly in progress
 - LALR(1), GLR parser
 - syntax referenced from [lua 5.4 reference manual](https://www.lua.org/manual/5.4/manual.html)

## project structure
 - `tokenizer`: tokenizing lua code string
 - `parser`: parsing tokenized lua code into AST
 - `exec`: executable version of the `parser`

## Cargo Features
 - `32bit`: use 32bit integer and float for `lua numeric` type
 - `diag`: enable `to_diag()` function for `ParseError`


## how to run
```
$ cargo run <source_file.lua>
```

will print the pretty-formatted `Debug` output of the AST ( `"{:#?}"` )