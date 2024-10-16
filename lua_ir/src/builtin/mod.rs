use lua_tokenizer::IntType;

use std::rc::Rc;

use crate::Chunk;
use crate::LuaEnv;
use crate::LuaFunction;
use crate::LuaNumber;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;
use crate::Stack;

// mod io;
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
    // env.insert("io".into(), io::init()?.into());

    // `_G` will be added in `VM::new_stack()` or `Stack::new()`
    Ok(env)
}

// tonumber
// pcall

pub fn print(
    stack: &mut Stack,
    env: &mut LuaEnv,
    chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    let args: Vec<_> = stack.pop_n(args).collect();
    for (idx, arg) in args.into_iter().enumerate() {
        if idx > 0 {
            print!("\t");
        }
        let to_string_ed = tostring_impl(stack, env, chunk, arg)?;
        print!("{}", String::from_utf8_lossy(&to_string_ed));
    }
    println!();
    Ok(0)
}
pub fn rawequal(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::ValueExpected);
    }
    let (lhs, rhs) = stack.pop2(args);
    stack.data_stack.push(LuaValue::Boolean(lhs == rhs));
    Ok(1)
}
pub fn rawlen(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let arg = stack.pop1(args);
    let len = match arg {
        LuaValue::String(s) => s.len() as IntType,
        LuaValue::Table(t) => t.borrow().len(),
        _ => return Err(RuntimeError::NotTableOrString),
    };
    stack.data_stack.push((len).into());
    Ok(1)
}
pub fn rawget(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::ValueExpected);
    }
    let (table, key) = stack.pop2(args);

    match table {
        LuaValue::Table(t) => {
            let get = t.borrow().get(&key).cloned().unwrap_or(LuaValue::Nil);
            stack.data_stack.push(get);
            Ok(1)
        }
        _ => Err(RuntimeError::NotTable),
    }
}
pub fn rawset(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args < 3 {
        return Err(RuntimeError::ValueExpected);
    }
    let (key, value) = stack.pop2(args - 1);
    let table = stack.top();

    match table {
        LuaValue::Table(t) => {
            if key.is_nil() {
                Err(RuntimeError::TableIndexNil)
            } else if key.is_nan() {
                Err(RuntimeError::TableIndexNan)
            } else {
                t.borrow_mut().insert(key, value);
                Ok(1)
            }
        }
        _ => Err(RuntimeError::NotTable),
    }
}
pub fn select(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut it = stack.pop_n(args);
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
                stack.data_stack.append(&mut rest);

                Ok(rest_len)
            }
        } else {
            if idx as usize > args - 1 {
                Ok(0)
            } else {
                let mut rest: Vec<_> = it.skip((idx - 1) as usize).collect();
                let rest_len = rest.len();
                stack.data_stack.append(&mut rest);

                Ok(rest_len)
            }
        }
    } else {
        drop(it);
        if let LuaValue::String(s) = index {
            if s[0] == b'#' {
                stack.data_stack.push(((args - 1) as IntType).into());
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
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::ValueExpected);
    }
    let meta = stack.pop1(args - 1);
    let table = stack.top();

    if let LuaValue::Table(table) = table {
        // check __metatable is defined
        if let Some(meta_old) = &table.borrow().meta {
            if meta_old
                .borrow()
                .map
                .contains_key(&LuaValue::from("__metatable"))
            {
                return Err(RuntimeError::ProtectedMetatable);
            }
        }
        match meta {
            LuaValue::Nil => {
                table.borrow_mut().meta = None;
                Ok(1)
            }
            LuaValue::Table(meta) => {
                table.borrow_mut().meta = Some(meta);
                Ok(1)
            }
            _ => Err(RuntimeError::NotTable),
        }
    } else {
        Err(RuntimeError::NotTable)
    }
}
pub fn getmetatable(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let value = stack.pop1(args);
    match value {
        LuaValue::Table(table) => {
            if let Some(meta) = &table.borrow().meta {
                // check __metatable is defined
                if let Some(assoc) = meta.borrow().get(&"__metatable".into()) {
                    stack.data_stack.push(assoc.clone());
                } else {
                    stack.data_stack.push(LuaValue::Table(Rc::clone(meta)));
                }
            } else {
                stack.data_stack.push(LuaValue::Nil);
            }
            Ok(1)
        }
        _ => Err(RuntimeError::NotTable),
    }
}

fn tostring_impl(
    stack: &mut Stack,
    env: &mut LuaEnv,
    chunk: &Chunk,
    arg: LuaValue,
) -> Result<Vec<u8>, RuntimeError> {
    match arg.get_metavalue("__tostring") {
        Some(meta) => {
            stack.data_stack.push(arg);
            stack.function_call(env, chunk, 1, meta, Some(1))?;
            let arg = stack.data_stack.pop().unwrap();
            tostring_impl(stack, env, chunk, arg)
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
    stack: &mut Stack,
    env: &mut LuaEnv,
    chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let arg = stack.pop1(args);
    let to_string_ed = LuaValue::String(tostring_impl(stack, env, chunk, arg)?);
    stack.data_stack.push(to_string_ed);
    Ok(1)
}

pub fn type_(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let arg = stack.pop1(args);
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
    stack.data_stack.push(type_str.into());
    Ok(1)
}

pub fn assert(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    if stack.data_stack[stack.data_stack.len() - args].to_bool() {
        Ok(args)
    } else {
        drop(stack.pop_n(args));
        Err(RuntimeError::Error)
    }
}

/// iterator function for `ipairs`
fn ipair_next(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::ValueExpected);
    }

    let (table, key) = stack.pop2(args);
    match table {
        LuaValue::Table(table) => match key {
            LuaValue::Number(LuaNumber::Int(mut n)) => {
                n += 1;
                if let Some(value) = table.borrow().get_arr(n) {
                    stack.data_stack.push((n).into());
                    stack.data_stack.push(value.clone());
                    Ok(2)
                } else {
                    stack.data_stack.push(LuaValue::Nil);
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
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let table = if let LuaValue::Table(table) = stack.pop1(args) {
        table
    } else {
        return Err(RuntimeError::NotTable);
    };

    stack
        .data_stack
        .push(LuaFunction::from_func(ipair_next).into());
    stack.data_stack.push(LuaValue::Table(table));
    stack.data_stack.push((0 as IntType).into());
    Ok(3)
}

pub fn next(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }

    let mut it = stack.pop_n(args);
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
                    if let Some((k, v)) = table.borrow().arr.first_key_value() {
                        stack.data_stack.push((*k).into());
                        stack.data_stack.push(v.clone());
                        Ok(2)
                    } else {
                        // no array part
                        if let Some((k, v)) = table.borrow().map.first() {
                            stack.data_stack.push(k.clone());
                            stack.data_stack.push(v.clone());
                            Ok(2)
                        } else {
                            // no hash part
                            stack.data_stack.push(LuaValue::Nil);
                            Ok(1)
                        }
                    }
                }

                // index is integer, get next element in array part
                LuaValue::Number(LuaNumber::Int(n)) => {
                    let table = table.borrow();
                    let mut range_it = table.arr.range(n..);
                    if range_it.next().map(|(k, _)| *k) == Some(n) {
                        if let Some((k, v)) = range_it.next() {
                            stack.data_stack.push((*k).into());
                            stack.data_stack.push(v.clone());
                            Ok(2)
                        } else {
                            // n is the last element, check hash part
                            if let Some((k, v)) = table.map.first() {
                                stack.data_stack.push(k.clone());
                                stack.data_stack.push(v.clone());
                                Ok(2)
                            } else {
                                // no hash part
                                stack.data_stack.push(LuaValue::Nil);
                                Ok(1)
                            }
                        }
                    } else {
                        Err(RuntimeError::InvalidKey)
                    }
                }

                index => {
                    // hash part
                    if let Some(cur_idx) = table.borrow().map.get_index_of(&index) {
                        if let Some((k, v)) = table.borrow().map.get_index(cur_idx + 1) {
                            stack.data_stack.push(k.clone());
                            stack.data_stack.push(v.clone());
                            Ok(2)
                        } else {
                            // no more elements
                            stack.data_stack.push(LuaValue::Nil);
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
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let table = if let LuaValue::Table(table) = stack.pop1(args) {
        table
    } else {
        return Err(RuntimeError::NotTable);
    };

    stack.data_stack.push(LuaFunction::from_func(next).into());
    stack.data_stack.push(LuaValue::Table(table));
    stack.data_stack.push(LuaValue::Nil);
    Ok(3)
}
