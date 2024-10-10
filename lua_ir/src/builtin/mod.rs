use lua_tokenizer::IntType;

use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;
use crate::Stack;

mod math;

/// generate default `_ENV` table
pub fn init_env() -> Result<LuaTable, RuntimeError> {
    // @TODO
    let mut env = LuaTable::new();
    env.map
        .insert("print".into(), LuaFunction::from_func(builtin_print).into());
    env.map.insert(
        "rawequal".into(),
        LuaFunction::from_func(builtin_rawequal).into(),
    );
    env.map.insert(
        "rawlen".into(),
        LuaFunction::from_func(builtin_rawlen).into(),
    );
    env.map
        .insert("type".into(), LuaFunction::from_func(builtin_type).into());
    env.map.insert("math".into(), math::init_math()?.into());
    Ok(env)
}

pub fn builtin_print(
    stack: &mut Stack,
    args: Vec<LuaValue>,
) -> Result<Vec<LuaValue>, RuntimeError> {
    for (idx, arg) in args.into_iter().enumerate() {
        if idx > 0 {
            print!("\t");
        }
        print!("{}", arg);
    }
    println!();
    Ok(vec![])
}
pub fn builtin_rawequal(
    stack: &mut Stack,
    args: Vec<LuaValue>,
) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let lhs = it.next().unwrap_or(LuaValue::Nil);
    let rhs = it.next().unwrap_or(LuaValue::Nil);
    drop(it);

    let eq = lhs.eq_raw(&rhs);

    Ok(vec![LuaValue::Boolean(eq)])
}
pub fn builtin_rawlen(
    stack: &mut Stack,
    args: Vec<LuaValue>,
) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let arg = it.next().unwrap_or(LuaValue::Nil);
    drop(it);

    let len = arg.len_raw()?;

    Ok(vec![LuaValue::Int(len as IntType)])
}

// rawset
// select
// setmetatable
// tonumber

// pub fn builtin_tostring(args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
//     let mut it = args.into_iter();
//     let arg = it.next().unwrap_or(LuaValue::Nil);
//     drop(it);

//     Ok(vec![LuaValue::String(arg.to_string())])
// }

pub fn builtin_type(stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
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
