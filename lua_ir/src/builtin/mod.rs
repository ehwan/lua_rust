use lua_tokenizer::IntType;

use std::cell::RefCell;
use std::rc::Rc;

use crate::Chunk;
use crate::LuaFunction;
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

    env.insert("_VERSION".into(), VERSION.into());

    env.insert("string".into(), string::init()?.into());
    env.insert("math".into(), math::init()?.into());
    env.insert("table".into(), table::init()?.into());
    // env.insert("io".into(), io::init()?.into());

    // @TODO _G
    env.insert(
        "_G".into(),
        LuaValue::Table(Rc::new(RefCell::new(env.clone()))),
    );
    Ok(env)
}

// tonumber
// pcall

pub fn print(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    let args: Vec<_> = stack.pop_n(args).collect();
    for (idx, arg) in args.into_iter().enumerate() {
        if idx > 0 {
            print!("\t");
        }
        let to_string_ed = tostring_impl(stack, _chunk, arg)?;
        print!("{}", String::from_utf8_lossy(&to_string_ed));
    }
    println!();
    Ok(0)
}
pub fn rawequal(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::ValueExpected);
    }
    let (lhs, rhs) = stack.pop2(args);
    stack.data_stack.push(LuaValue::Boolean(lhs == rhs));
    Ok(1)
}
pub fn rawlen(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
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
pub fn rawget(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
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
pub fn rawset(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
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
pub fn select(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
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

pub fn setmetatable(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::ValueExpected);
    }
    let meta = stack.pop1(args - 1);
    let table = stack.top();

    if let LuaValue::Table(table) = table {
        // check __metatable is defined
        if let Some(meta_old) = &table.borrow().meta {
            if meta_old.borrow().get(&"__metatable".into()).is_some() {
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
pub fn getmetatable(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
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

fn tostring_impl(stack: &mut Stack, chunk: &Chunk, arg: LuaValue) -> Result<Vec<u8>, RuntimeError> {
    match arg.get_metavalue("__tostring") {
        Some(meta) => {
            stack.data_stack.push(arg);
            stack.function_call(chunk, 1, meta, Some(1))?;
            let arg = stack.data_stack.pop().unwrap();
            tostring_impl(stack, chunk, arg)
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
pub fn tostring(stack: &mut Stack, chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let arg = stack.pop1(args);
    let to_string_ed = LuaValue::String(tostring_impl(stack, chunk, arg)?);
    stack.data_stack.push(to_string_ed);
    Ok(1)
}

pub fn type_(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
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

pub fn assert(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
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
