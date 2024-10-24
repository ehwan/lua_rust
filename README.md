# lua_rust
lua syntax parser & runtime interpreter in Rust

 - ***Greatly in progress***
 - LALR(1), GLR parser
 - syntax referenced from [lua 5.4 reference manual](https://www.lua.org/manual/5.4/manual.html)

## Cargo Features
 - `32bit`: use 32bit integer and float for `lua numeric` type

## how to run
Simply running
```
$ cargo run <source_file.lua>
```
or
```
$ cargo run
```
will start lua REPL. Note that this executable is not `cargo publish`ed.


As library, add this to your `Cargo.toml`
```toml
[dependencies]
lua_ir = "..."
```

```rust
let mut lua_env = lua_ir::LuaEnv::new();

lua_env.eval_chunk( "print('Hello, World!')" );
lua_env.eval_chunk( "a = {1, 2, 3}" );
```