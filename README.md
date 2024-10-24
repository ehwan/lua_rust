# lua_rust
[![crates.io](https://img.shields.io/crates/v/lua_ir.svg)](https://crates.io/crates/lua_ir)
[![docs.rs](https://docs.rs/lua_ir/badge.svg)](https://docs.rs/lua_ir)

lua syntax parser & runtime interpreter in Rust

 - LALR(1), GLR parser
 - syntax referenced from [lua 5.4 reference manual](https://www.lua.org/manual/5.4/manual.html)
 - ***Greatly in progress***
    - grammar fully implemented
    - std library barely implemented

## Cargo Features
 - `32bit`: use 32bit integer and float for `lua numeric` type

## How to use
As library, add [`lua_ir`](https://crates.io/crates/lua_ir) crate to your `Cargo.toml`
```toml
[dependencies]
lua_ir = "..."
```

```rust
let mut env = lua_ir::LuaEnv::new();

env.eval_chunk( b"var_hello = 'Hello'" )?;
env.eval_chunk( b"var_world = 'World'" )?;
env.eval_chunk( b"print( var_hello .. ', ' .. var_world .. '!' )" )?;
// Hello, World!

let hello_value = env.get_global( "var_hello" )?;
let world_value = env.get_global( "var_world" )?;
env.set_global( "var_hello", 10.into() )?;
```

Simply running
```
$ cargo run <source_file.lua>
```
or
```
$ cargo run
```
will start lua REPL. Note that this executable is not `cargo publish`ed.

