use std::cell::RefCell;
use std::rc::Rc;

use crate::IntType;
use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;
use crate::Stack;

/// init string module
pub fn init() -> Result<LuaValue, RuntimeError> {
    let mut string = LuaTable::new();
    string
        .map
        .insert("byte".into(), LuaFunction::from_func(byte).into());
    string
        .map
        .insert("sub".into(), LuaFunction::from_func(sub).into());
    string
        .map
        .insert("char".into(), LuaFunction::from_func(char_).into());
    string
        .map
        .insert("len".into(), LuaFunction::from_func(len).into());
    string
        .map
        .insert("lower".into(), LuaFunction::from_func(lower).into());
    string
        .map
        .insert("rep".into(), LuaFunction::from_func(rep).into());
    string
        .map
        .insert("reverse".into(), LuaFunction::from_func(reverse).into());
    string
        .map
        .insert("upper".into(), LuaFunction::from_func(upper).into());
    Ok(LuaValue::Table(Rc::new(RefCell::new(string))))
}

// @TODO
// dump
// format
// gmatch
// gsub
// match
// pack
// packsize
// unpack

pub fn byte(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let sub = sub_impl(_stack, args)?;
    sub.into_iter()
        .map(|c| Ok(LuaValue::Number((c as IntType).into())))
        .collect()
}

pub fn sub_impl(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<u8>, RuntimeError> {
    let mut it = args.into_iter();
    let s = match it.next() {
        Some(LuaValue::String(s)) => s,
        _ => return Err(RuntimeError::NotString),
    };
    let mut i = match it.next() {
        None => 1,
        Some(val) => match val.try_to_int() {
            Some(i) => i,
            None => return Err(RuntimeError::NotInteger),
        },
    };
    if i < 0 {
        i = s.len() as i64 + i + 1;
    }
    if i < 1 {
        i = 1;
    } else if i > s.len() as i64 {
        i = s.len() as i64;
    }

    let mut j = match it.next() {
        None => 1,
        Some(val) => match val.try_to_int() {
            Some(j) => j,
            None => return Err(RuntimeError::NotInteger),
        },
    };
    if j < 0 {
        j = s.len() as i64 + j + 1;
    }
    if j < 1 {
        j = 1;
    } else if j > s.len() as i64 {
        j = s.len() as i64;
    }

    if i > j {
        i = j;
    }

    Ok(s[((i - 1) as usize)..(j as usize)].to_vec())
}
pub fn sub(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    sub_impl(_stack, args).map(|s| vec![LuaValue::String(s)])
}

pub fn char_(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let chars: Result<Vec<u8>, _> = args
        .into_iter()
        .map(|c| match c.try_to_int() {
            Some(i) => {
                if i < 0 || i > 255 {
                    Err(RuntimeError::OutOfRange)
                } else {
                    Ok(i as u8)
                }
            }
            None => Err(RuntimeError::NotInteger),
        })
        .collect();
    Ok(vec![LuaValue::String(chars?)])
}

pub fn len(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let s = match args.into_iter().next() {
        Some(LuaValue::String(s)) => s,
        _ => return Err(RuntimeError::NotString),
    };
    Ok(vec![(s.len() as IntType).into()])
}

pub fn lower(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let s = match args.into_iter().next() {
        Some(LuaValue::String(s)) => s,
        _ => return Err(RuntimeError::NotString),
    };
    let ret = LuaValue::String(s.into_iter().map(|c| c.to_ascii_lowercase()).collect());
    Ok(vec![ret])
}
pub fn upper(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let s = match args.into_iter().next() {
        Some(LuaValue::String(s)) => s,
        _ => return Err(RuntimeError::NotString),
    };
    let ret = LuaValue::String(s.into_iter().map(|c| c.to_ascii_uppercase()).collect());
    Ok(vec![ret])
}
pub fn rep(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let s = match it.next() {
        Some(LuaValue::String(s)) => s,
        _ => return Err(RuntimeError::NotString),
    };
    let n = match it.next().unwrap_or_default().try_to_int() {
        Some(n) => n,
        None => return Err(RuntimeError::NotInteger),
    };
    if n <= 0 {
        return Ok(vec![LuaValue::String(vec![])]);
    }
    let sep = match it.next() {
        None => vec![],
        Some(LuaValue::String(s)) => s,
        _ => return Err(RuntimeError::NotString),
    };

    let mut ret = Vec::with_capacity(s.len() * n as usize + sep.len() * (n as usize - 1));
    for i in 0..n {
        if i != 0 {
            ret.extend_from_slice(&sep);
        }
        ret.extend_from_slice(&s);
    }
    Ok(vec![LuaValue::String(ret)])
}

pub fn reverse(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let mut s = match it.next() {
        Some(LuaValue::String(s)) => s,
        _ => return Err(RuntimeError::NotString),
    };
    s.reverse();
    Ok(vec![LuaValue::String(s)])
}
