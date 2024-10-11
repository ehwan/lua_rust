use lua_tokenizer::IntType;

use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;
use crate::Stack;

mod math;
mod string;

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
    env.map.insert("math".into(), math::init()?.into());
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
        LuaValue::Table(t) => unimplemented!("table length"),
        _ => 0,
    };

    Ok(vec![LuaValue::Int(len as IntType)])
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
        LuaValue::Table(t) => match &key {
            LuaValue::Nil => return Err(RuntimeError::SetOnNonTable),
            LuaValue::Float(f) => {
                if f.is_nan() {
                    return Err(RuntimeError::TableIndexNan);
                } else {
                    t.borrow_mut().map.insert(key, value);
                }
            }
            _ => {
                t.borrow_mut().map.insert(key, value);
            }
        },
        _ => return Err(RuntimeError::SetOnNonTable),
    }
    Ok(vec![table])
}
pub fn select(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let index = it.next().unwrap_or(LuaValue::Nil);
    let mut rest = it.collect::<Vec<_>>();

    let index = match index {
        LuaValue::Int(i) => i,
        LuaValue::Float(f) => LuaValue::float_to_int(f)?,
        LuaValue::String(s) => {
            if s[0] == b'#' {
                return Ok(vec![LuaValue::Int(rest.len() as IntType)]);
            } else {
                return Err(RuntimeError::InvalidArgument(0));
            }
        }
        _ => return Err(RuntimeError::InvalidArgument(0)),
    };
    if index == 0 {
        return Err(RuntimeError::InvalidArgument(0));
    }

    let index = if index < 0 {
        let idx = -index;
        if idx > rest.len() as IntType {
            return Err(RuntimeError::InvalidArgument(0));
        }
        rest.len() as IntType - idx
    } else {
        index
    };
    Ok(rest.drain(index as usize..).collect())
}

// setmetatable
// tonumber

pub fn tostring(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let arg = it.next().unwrap_or(LuaValue::Nil);
    drop(it);
    let string = match arg {
        LuaValue::Table(table) => {
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
        LuaValue::Int(_) => Ok(vec![LuaValue::String("number".into())]),
        LuaValue::Float(_) => Ok(vec![LuaValue::String("number".into())]),
        LuaValue::String(_) => Ok(vec![LuaValue::String("string".into())]),
        LuaValue::Table(_) => Ok(vec![LuaValue::String("table".into())]),
        LuaValue::Function(_) => Ok(vec![LuaValue::String("function".into())]),
        LuaValue::Thread(_) => Ok(vec![LuaValue::String("thread".into())]),
        LuaValue::UserData(_) => Ok(vec![LuaValue::String("userdata".into())]),
    }
}
