use std::cell::RefCell;
use std::rc::Rc;

use crate::Chunk;
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

pub fn sub_impl(s: &[u8], mut i: IntType, mut j: IntType) -> &'_ [u8] {
    if i < 0 {
        i = s.len() as i64 + i + 1;
    }
    if i < 1 {
        i = 1;
    } else if i > s.len() as i64 {
        i = s.len() as i64;
    }

    if j < 0 {
        j = s.len() as i64 + j + 1;
    }
    if j < 1 {
        j = 1;
    } else if j > s.len() as i64 {
        j = s.len() as i64;
    }

    if i > j {
        &s[0..0]
    } else {
        &s[((i - 1) as usize)..(j as usize)]
    }
}
pub fn byte(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut it = stack.pop_n(args);
    let s = match it.next().unwrap() {
        LuaValue::String(s) => s,
        _ => return Err(RuntimeError::NotString),
    };
    let i = match it.next() {
        Some(i) => match i.try_to_int() {
            Some(i) => i,
            None => return Err(RuntimeError::NotInteger),
        },
        None => 1,
    };
    let j = match it.next() {
        Some(j) => match j.try_to_int() {
            Some(j) => j,
            None => return Err(RuntimeError::NotInteger),
        },
        None => 1,
    };
    drop(it);

    let sub = sub_impl(&s, i, j);
    for c in sub {
        stack.data_stack.push((*c as IntType).into());
    }
    Ok(sub.len())
}
pub fn sub(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut it = stack.pop_n(args);
    let s = match it.next().unwrap() {
        LuaValue::String(s) => s,
        _ => return Err(RuntimeError::NotString),
    };
    let i = match it.next().unwrap().try_to_int() {
        Some(i) => i,
        None => return Err(RuntimeError::NotInteger),
    };
    let j = match it.next() {
        Some(j) => match j.try_to_int() {
            Some(j) => j,
            None => return Err(RuntimeError::NotInteger),
        },
        None => s.len() as IntType,
    };
    drop(it);

    let sub = sub_impl(&s, i, j);
    stack.data_stack.push(LuaValue::String(sub.to_vec()));
    Ok(1)
}

pub fn char_(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    let chars: Result<Vec<u8>, _> = stack
        .pop_n(args)
        .into_iter()
        .map(|c| match c.try_to_int() {
            Some(i) => {
                if i < 0 || i > 255 {
                    Err(RuntimeError::OutOfRangeChar)
                } else {
                    Ok(i as u8)
                }
            }
            None => Err(RuntimeError::NotInteger),
        })
        .collect();
    let chars = chars?;
    stack.data_stack.push(LuaValue::String(chars));
    Ok(1)
}

pub fn len(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    match stack.pop1(args) {
        LuaValue::String(s) => {
            stack.data_stack.push((s.len() as IntType).into());
            Ok(1)
        }
        _ => Err(RuntimeError::NotString),
    }
}

pub fn lower(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    match stack.pop1(args) {
        LuaValue::String(s) => {
            let ret = LuaValue::String(s.into_iter().map(|c| c.to_ascii_lowercase()).collect());
            stack.data_stack.push(ret);
            Ok(1)
        }
        _ => return Err(RuntimeError::NotString),
    }
}
pub fn upper(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    match stack.pop1(args) {
        LuaValue::String(s) => {
            let ret = LuaValue::String(s.into_iter().map(|c| c.to_ascii_uppercase()).collect());
            stack.data_stack.push(ret);
            Ok(1)
        }
        _ => return Err(RuntimeError::NotString),
    }
}
pub fn rep(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut it = stack.pop_n(args);
    let s = match it.next().unwrap() {
        LuaValue::String(s) => s,
        _ => return Err(RuntimeError::NotString),
    };
    let n = match it.next().unwrap().try_to_int() {
        Some(n) => n,
        None => return Err(RuntimeError::NotInteger),
    };
    if n <= 0 {
        drop(it);
        stack.data_stack.push(LuaValue::String(vec![]));
        return Ok(1);
    }

    let sep = match it.next() {
        Some(LuaValue::String(s)) => s,
        None => vec![],
        _ => return Err(RuntimeError::NotString),
    };
    drop(it);

    let mut ret = Vec::with_capacity(s.len() * n as usize + sep.len() * (n as usize - 1));
    for i in 0..n {
        if i != 0 {
            ret.extend_from_slice(&sep);
        }
        ret.extend_from_slice(&s);
    }
    stack.data_stack.push(LuaValue::String(ret));
    Ok(1)
}

pub fn reverse(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut s = match stack.pop1(args) {
        LuaValue::String(s) => s,
        _ => return Err(RuntimeError::NotString),
    };
    s.reverse();
    stack.data_stack.push(LuaValue::String(s));
    Ok(1)
}
