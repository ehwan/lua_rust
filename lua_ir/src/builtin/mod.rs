use lua_tokenizer::IntType;

use std::sync::Arc;
use std::sync::RwLock;

use crate::Chunk;
use crate::LuaEnv;
use crate::LuaFunction;
use crate::LuaNumber;
use crate::LuaTable;
use crate::LuaThread;
use crate::LuaValue;
use crate::RuntimeError;

// mod io;
mod coroutine;
mod math;
mod string;
mod table;

const VERSION: &str = "Lua 5.4 in Rust";

/// generate default `_ENV` table
pub fn init_env() -> Result<LuaTable, RuntimeError> {
    // @TODO
    let mut env = LuaTable::new();
    env.insert("print".into(), LuaFunction::from_func(print).into());
    env.insert("rawequal".into(), LuaFunction::from_func(rawequal).into());
    env.insert("rawlen".into(), LuaFunction::from_func(rawlen).into());
    env.insert("rawget".into(), LuaFunction::from_func(rawget).into());
    env.insert("rawset".into(), LuaFunction::from_func(rawset).into());
    env.insert("type".into(), LuaFunction::from_func(type_).into());
    env.insert("tostring".into(), LuaFunction::from_func(tostring).into());
    env.insert("select".into(), LuaFunction::from_func(select).into());
    env.insert(
        "setmetatable".into(),
        LuaFunction::from_func(setmetatable).into(),
    );
    env.insert(
        "getmetatable".into(),
        LuaFunction::from_func(getmetatable).into(),
    );
    env.insert("assert".into(), LuaFunction::from_func(assert).into());

    env.insert("ipairs".into(), LuaFunction::from_func(ipairs).into());
    env.insert("next".into(), LuaFunction::from_func(next).into());
    env.insert("pairs".into(), LuaFunction::from_func(pairs).into());

    env.insert("_VERSION".into(), VERSION.into());

    env.insert("string".into(), string::init()?.into());
    env.insert("math".into(), math::init()?.into());
    env.insert("table".into(), table::init()?.into());
    env.insert("coroutine".into(), coroutine::init()?.into());
    // env.insert("io".into(), io::init()?.into());

    // `_G` will be added in `VM::new_stack()` or `Stack::new()`
    Ok(env)
}

// tonumber
// pcall

pub fn print(
    env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    let args = {
        let args: Vec<_> = thread.write().unwrap().pop_n(args).collect();
        args
    };
    for (idx, arg) in args.into_iter().enumerate() {
        if idx > 0 {
            print!("\t");
        }
        let to_string_ed = tostring_impl(env, thread, chunk, arg)?;
        print!("{}", String::from_utf8_lossy(&to_string_ed));
    }
    println!();
    Ok(0)
}
pub fn rawequal(
    _env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut t = thread.write().unwrap();
    let (lhs, rhs) = t.pop2(args);
    t.data_stack.push(LuaValue::Boolean(lhs == rhs));
    Ok(1)
}
pub fn rawlen(
    _env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut t = thread.write().unwrap();
    let arg = t.pop1(args);
    let len = match arg {
        LuaValue::String(s) => s.len() as IntType,
        LuaValue::Table(table) => table.read().unwrap().len(),
        _ => return Err(RuntimeError::NotTableOrString),
    };
    t.data_stack.push((len).into());
    Ok(1)
}
pub fn rawget(
    _env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut t = thread.write().unwrap();
    let (table, key) = t.pop2(args);

    match table {
        LuaValue::Table(table) => {
            let get = table
                .read()
                .unwrap()
                .get(&key)
                .cloned()
                .unwrap_or(LuaValue::Nil);
            t.data_stack.push(get);
            Ok(1)
        }
        _ => Err(RuntimeError::NotTable),
    }
}
pub fn rawset(
    _env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args < 3 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut t = thread.write().unwrap();
    let (key, value) = t.pop2(args - 1);
    let table = t.top();

    match table {
        LuaValue::Table(table) => {
            if key.is_nil() {
                Err(RuntimeError::TableIndexNil)
            } else if key.is_nan() {
                Err(RuntimeError::TableIndexNan)
            } else {
                table.write().unwrap().insert(key, value);
                Ok(1)
            }
        }
        _ => Err(RuntimeError::NotTable),
    }
}
pub fn select(
    _env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut t = thread.write().unwrap();
    let mut it = t.pop_n(args);
    let index = it.next().unwrap();
    if let Some(idx) = index.try_to_int() {
        if idx == 0 {
            Err(RuntimeError::OutOfRange)
        } else if idx < 0 {
            if (-idx) as usize > args - 1 {
                Err(RuntimeError::OutOfRange)
            } else {
                let offset = (args as IntType - 1 + idx) as usize;
                let mut rest: Vec<_> = it.skip(offset).collect();
                let rest_len = rest.len();
                t.data_stack.append(&mut rest);

                Ok(rest_len)
            }
        } else {
            if idx as usize > args - 1 {
                Ok(0)
            } else {
                let mut rest: Vec<_> = it.skip((idx - 1) as usize).collect();
                let rest_len = rest.len();
                t.data_stack.append(&mut rest);

                Ok(rest_len)
            }
        }
    } else {
        drop(it);
        if let LuaValue::String(s) = index {
            if s[0] == b'#' {
                t.data_stack.push(((args - 1) as IntType).into());
                Ok(1)
            } else {
                Err(RuntimeError::NotInteger)
            }
        } else {
            Err(RuntimeError::NotInteger)
        }
    }
}

pub fn setmetatable(
    _env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut t = thread.write().unwrap();
    let meta = t.pop1(args - 1);
    let table = t.top();

    if let LuaValue::Table(table) = table {
        // check __metatable is defined
        if let Some(meta_old) = &table.read().unwrap().meta {
            if meta_old
                .read()
                .unwrap()
                .map
                .contains_key(&LuaValue::from("__metatable"))
            {
                return Err(RuntimeError::ProtectedMetatable);
            }
        }
        match meta {
            LuaValue::Nil => {
                table.write().unwrap().meta = None;
                Ok(1)
            }
            LuaValue::Table(meta) => {
                table.write().unwrap().meta = Some(meta);
                Ok(1)
            }
            _ => Err(RuntimeError::NotTable),
        }
    } else {
        Err(RuntimeError::NotTable)
    }
}
pub fn getmetatable(
    _env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut t = thread.write().unwrap();
    let value = t.pop1(args);
    match value {
        LuaValue::Table(table) => {
            if let Some(meta) = &table.read().unwrap().meta {
                // check __metatable is defined
                if let Some(assoc) = meta.read().unwrap().get(&"__metatable".into()) {
                    t.data_stack.push(assoc.clone());
                } else {
                    t.data_stack.push(LuaValue::Table(Arc::clone(meta)));
                }
            } else {
                t.data_stack.push(LuaValue::Nil);
            }
            Ok(1)
        }
        _ => Err(RuntimeError::NotTable),
    }
}

fn tostring_impl(
    env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    chunk: &Chunk,
    arg: LuaValue,
) -> Result<Vec<u8>, RuntimeError> {
    match arg.get_metavalue("__tostring") {
        Some(meta) => {
            {
                thread.write().unwrap().data_stack.push(arg);
            }
            env.function_call(thread, chunk, 1, meta, Some(1))?;
            let arg = thread.write().unwrap().data_stack.pop().unwrap();
            tostring_impl(env, thread, chunk, arg)
        }
        _ => match arg.get_metavalue("__name") {
            Some(name) => match name {
                LuaValue::String(name) => Ok(name),
                _ => Ok(arg.to_string().into_bytes()),
            },
            _ => Ok(arg.to_string().into_bytes()),
        },
    }
}
pub fn tostring(
    env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let arg = thread.write().unwrap().pop1(args);
    let to_string_ed = LuaValue::String(tostring_impl(env, thread, chunk, arg)?);
    thread.write().unwrap().data_stack.push(to_string_ed);
    Ok(1)
}

pub fn type_(
    _env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut t = thread.write().unwrap();
    let arg = t.pop1(args);
    let type_str = match arg {
        LuaValue::Nil => "nil",
        LuaValue::Boolean(_) => "boolean",
        LuaValue::Number(_) => "number",
        LuaValue::String(_) => "string",
        LuaValue::Table(_) => "table",
        LuaValue::Function(_) => "function",
        LuaValue::Thread(_) => "thread",
        LuaValue::UserData(_) => "userdata",
    };
    t.data_stack.push(type_str.into());
    Ok(1)
}

pub fn assert(
    _env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let t = thread.read().unwrap();
    if t.data_stack[t.data_stack.len() - args].to_bool() {
        Ok(args)
    } else {
        drop(t);
        drop(thread.write().unwrap().pop_n(args));
        Err(RuntimeError::Error)
    }
}

/// iterator function for `ipairs`
fn ipair_next(
    _env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::ValueExpected);
    }

    let mut t = thread.write().unwrap();
    let (table, key) = t.pop2(args);
    match table {
        LuaValue::Table(table) => match key {
            LuaValue::Number(LuaNumber::Int(mut n)) => {
                n += 1;
                if let Some(value) = table.read().unwrap().get_arr(n) {
                    t.data_stack.push((n).into());
                    t.data_stack.push(value.clone());
                    Ok(2)
                } else {
                    t.data_stack.push(LuaValue::Nil);
                    Ok(1)
                }
            }
            // this should not happen; since this function is only called from `ipairs`, privately
            _ => Err(RuntimeError::NotInteger),
        },
        _ => Err(RuntimeError::NotTable),
    }
}
pub fn ipairs(
    _env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut t = thread.write().unwrap();
    let table = if let LuaValue::Table(table) = t.pop1(args) {
        table
    } else {
        return Err(RuntimeError::NotTable);
    };

    t.data_stack.push(LuaFunction::from_func(ipair_next).into());
    t.data_stack.push(LuaValue::Table(table));
    t.data_stack.push((0 as IntType).into());
    Ok(3)
}

pub fn next(
    _env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }

    let mut t = thread.write().unwrap();

    let mut it = t.pop_n(args);
    let table = it.next().unwrap();
    let index = it.next().unwrap_or_default();
    drop(it);

    // iterate table through
    // integer keys (array part) first, then hash part

    match table {
        LuaValue::Table(table) => {
            match index {
                // index is nil, get first key for iteration
                LuaValue::Nil => {
                    if let Some((k, v)) = table.read().unwrap().arr.first_key_value() {
                        t.data_stack.push((*k).into());
                        t.data_stack.push(v.clone());
                        Ok(2)
                    } else {
                        // no array part
                        if let Some((k, v)) = table.read().unwrap().map.first() {
                            t.data_stack.push(k.clone());
                            t.data_stack.push(v.clone());
                            Ok(2)
                        } else {
                            // no hash part
                            t.data_stack.push(LuaValue::Nil);
                            Ok(1)
                        }
                    }
                }

                // index is integer, get next element in array part
                LuaValue::Number(LuaNumber::Int(n)) => {
                    let table = table.read().unwrap();
                    let mut range_it = table.arr.range(n..);
                    if range_it.next().map(|(k, _)| *k) == Some(n) {
                        if let Some((k, v)) = range_it.next() {
                            t.data_stack.push((*k).into());
                            t.data_stack.push(v.clone());
                            Ok(2)
                        } else {
                            // n is the last element, check hash part
                            if let Some((k, v)) = table.map.first() {
                                t.data_stack.push(k.clone());
                                t.data_stack.push(v.clone());
                                Ok(2)
                            } else {
                                // no hash part
                                t.data_stack.push(LuaValue::Nil);
                                Ok(1)
                            }
                        }
                    } else {
                        Err(RuntimeError::InvalidKey)
                    }
                }

                index => {
                    // hash part
                    let table = table.read().unwrap();
                    if let Some(cur_idx) = table.map.get_index_of(&index) {
                        if let Some((k, v)) = table.map.get_index(cur_idx + 1) {
                            t.data_stack.push(k.clone());
                            t.data_stack.push(v.clone());
                            Ok(2)
                        } else {
                            // no more elements
                            t.data_stack.push(LuaValue::Nil);
                            Ok(1)
                        }
                    } else {
                        Err(RuntimeError::InvalidKey)
                    }
                }
            }
        }
        _ => Err(RuntimeError::NotTable),
    }
}

// @TODO __pair metamethod
pub fn pairs(
    _env: &mut LuaEnv,
    thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut t = thread.write().unwrap();
    let table = if let LuaValue::Table(table) = t.pop1(args) {
        table
    } else {
        return Err(RuntimeError::NotTable);
    };

    t.data_stack.push(LuaFunction::from_func(next).into());
    t.data_stack.push(LuaValue::Table(table));
    t.data_stack.push(LuaValue::Nil);
    Ok(3)
}
