use lua_tokenizer::IntType;

use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;
use crate::Stack;

mod io;
mod math;
mod string;
mod table;

const VERSION: &str = "Lua 5.4 in Rust";

/// generate default `_ENV` table
pub fn init_env() -> Result<LuaTable, RuntimeError> {
    // @TODO
    let mut env = LuaTable::new();
    env.map
        .insert("print".into(), LuaFunction::from_func(print).into());
    env.map
        .insert("rawequal".into(), LuaFunction::from_func(rawequal).into());
    env.map
        .insert("rawlen".into(), LuaFunction::from_func(rawlen).into());
    env.map
        .insert("rawget".into(), LuaFunction::from_func(rawget).into());
    env.map
        .insert("rawset".into(), LuaFunction::from_func(rawset).into());
    env.map
        .insert("type".into(), LuaFunction::from_func(type_).into());
    env.map
        .insert("tostring".into(), LuaFunction::from_func(tostring).into());
    env.map
        .insert("select".into(), LuaFunction::from_func(select).into());

    env.map.insert("_VERSION".into(), VERSION.into());

    env.map.insert("string".into(), string::init()?.into());
    env.map.insert("table".into(), table::init()?.into());
    env.map.insert("math".into(), math::init()?.into());
    env.map.insert("io".into(), io::init()?.into());
    Ok(env)
}

pub fn print(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    for (idx, arg) in args.into_iter().enumerate() {
        if idx > 0 {
            print!("\t");
        }
        print!("{}", arg);
    }
    println!();
    Ok(vec![])
}
pub fn rawequal(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let lhs = it.next().unwrap_or(LuaValue::Nil);
    let rhs = it.next().unwrap_or(LuaValue::Nil);
    drop(it);
    Ok(vec![LuaValue::Boolean(lhs == rhs)])
}
pub fn rawlen(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let arg = it.next().unwrap_or(LuaValue::Nil);
    drop(it);

    let len = match arg {
        LuaValue::String(s) => s.len(),
        LuaValue::Table(_t) => unimplemented!("table length"),
        _ => 0,
    };

    Ok(vec![(len as IntType).into()])
}
pub fn rawget(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let table = it.next().unwrap_or(LuaValue::Nil);
    let key = it.next().unwrap_or(LuaValue::Nil);
    drop(it);

    match table {
        LuaValue::Table(t) => Ok(vec![t
            .borrow()
            .map
            .get(&key)
            .cloned()
            .unwrap_or(LuaValue::Nil)]),
        _ => Err(RuntimeError::GetOnNonTable),
    }
}
pub fn rawset(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let table = it.next().unwrap_or(LuaValue::Nil);
    let key = it.next().unwrap_or(LuaValue::Nil);
    let value = it.next().unwrap_or(LuaValue::Nil);
    drop(it);

    match &table {
        LuaValue::Table(t) => {
            if key.is_nil() {
                return Err(RuntimeError::TableIndexNil);
            } else if key.is_nan() {
                return Err(RuntimeError::TableIndexNan);
            }
            t.borrow_mut().map.insert(key, value);
        }
        _ => return Err(RuntimeError::SetOnNonTable),
    }
    Ok(vec![table])
}
pub fn select(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let index = it.next().unwrap_or(LuaValue::Nil);
    let mut rest = it.collect::<Vec<_>>();

    if let Some(idx) = index.try_to_int() {
        if idx == 0 {
            return Err(RuntimeError::OutOfRange);
        }
        if idx < 0 {
            let idx = rest.len() as IntType + idx;
            if idx < 0 {
                return Err(RuntimeError::OutOfRange);
            }
            return Ok(rest.drain(idx as usize..).collect());
        } else {
            if idx as usize > rest.len() {
                return Ok(vec![]);
            } else {
                return Ok(rest.drain(idx as usize - 1..).collect());
            }
        }
    } else {
        if let LuaValue::String(s) = index {
            if s[0] == b'#' {
                return Ok(vec![(rest.len() as IntType).into()]);
            } else {
                return Err(RuntimeError::NotInteger);
            }
        } else {
            return Err(RuntimeError::NotInteger);
        }
    }
}

// setmetatable
// tonumber

pub fn tostring(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let arg = it.next().unwrap_or(LuaValue::Nil);
    drop(it);
    let string = match arg {
        LuaValue::Table(_table) => {
            // format!("table: {:p}", Rc::as_ptr(&table))
            unimplemented!("table to string");
        }
        _ => arg.to_string(),
    };

    Ok(vec![LuaValue::String(string.into())])
}

pub fn type_(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let arg = it.next().unwrap_or(LuaValue::Nil);
    drop(it);

    match arg {
        LuaValue::Nil => Ok(vec![LuaValue::String("nil".into())]),
        LuaValue::Boolean(_) => Ok(vec![LuaValue::String("boolean".into())]),
        LuaValue::Number(_) => Ok(vec![LuaValue::String("number".into())]),
        LuaValue::String(_) => Ok(vec![LuaValue::String("string".into())]),
        LuaValue::Table(_) => Ok(vec![LuaValue::String("table".into())]),
        LuaValue::Function(_) => Ok(vec![LuaValue::String("function".into())]),
        LuaValue::Thread(_) => Ok(vec![LuaValue::String("thread".into())]),
        LuaValue::UserData(_) => Ok(vec![LuaValue::String("userdata".into())]),
    }
}
