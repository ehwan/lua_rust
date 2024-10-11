use std::cell::RefCell;
use std::rc::Rc;

use lua_tokenizer::FloatType;

use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;
use crate::Stack;

/// init math module
pub fn init() -> Result<LuaValue, RuntimeError> {
    let mut math = LuaTable::new();
    math.map
        .insert("abs".into(), LuaFunction::from_func(abs).into());
    math.map
        .insert("acos".into(), LuaFunction::from_func(acos).into());
    math.map
        .insert("asin".into(), LuaFunction::from_func(asin).into());
    Ok(LuaValue::Table(Rc::new(RefCell::new(math))))
}

pub fn abs(stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let arg = it.next().unwrap_or(LuaValue::Nil);
    drop(it);

    match arg {
        LuaValue::Int(i) => {
            let abs = i.abs();
            Ok(vec![LuaValue::Int(abs)])
        }
        LuaValue::Float(f) => {
            let abs = f.abs();
            Ok(vec![LuaValue::Float(abs)])
        }
        _ => Err(RuntimeError::TypeError),
    }
}
pub fn acos(stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let arg = it.next().unwrap_or(LuaValue::Nil);
    drop(it);

    match arg {
        LuaValue::Int(i) => {
            let res = (i as FloatType).acos();
            Ok(vec![LuaValue::Float(res)])
        }
        LuaValue::Float(f) => {
            let res = f.acos();
            Ok(vec![LuaValue::Float(res)])
        }
        _ => Err(RuntimeError::TypeError),
    }
}
pub fn asin(stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let arg = it.next().unwrap_or(LuaValue::Nil);
    drop(it);

    match arg {
        LuaValue::Int(i) => {
            let res = (i as FloatType).asin();
            Ok(vec![LuaValue::Float(res)])
        }
        LuaValue::Float(f) => {
            let res = f.asin();
            Ok(vec![LuaValue::Float(res)])
        }
        _ => Err(RuntimeError::TypeError),
    }
}
