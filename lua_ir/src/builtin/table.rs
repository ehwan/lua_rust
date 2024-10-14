use std::cell::RefCell;
use std::rc::Rc;

use crate::Chunk;
use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;
use crate::Stack;

/// init table module
pub fn init() -> Result<LuaValue, RuntimeError> {
    let mut table = LuaTable::new();

    table.insert("concat".into(), LuaFunction::from_func(concat).into());

    Ok(LuaValue::Table(Rc::new(RefCell::new(table))))
}

pub fn concat(stack: &mut Stack, chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    let mut it = stack.pop_n(args);
    let list = match it.next() {
        Some(LuaValue::Table(t)) => t,
        _ => return Err(RuntimeError::NotTable),
    };

    let sep = it.next();
    let i = match it.next() {
        Some(i) => match i.try_to_int() {
            Some(i) => i,
            _ => return Err(RuntimeError::NotNumber),
        },
        None => 1,
    };
    let j = match it.next() {
        Some(j) => match j.try_to_int() {
            Some(j) => j,
            _ => return Err(RuntimeError::NotNumber),
        },
        _ => list.borrow().len() as i64,
    };
    drop(it);

    if i > j {
        stack.data_stack.push(LuaValue::String(Default::default()));
        return Ok(1);
    }

    let sep = match sep {
        Some(sep) => super::tostring_impl(stack, chunk, sep)?,
        None => Vec::new(),
    };

    let mut ret = Vec::with_capacity(sep.len() * (j - i) as usize + ((j - i + 1) * 4) as usize);
    for k in i..=j {
        if k != i {
            ret.extend(sep.iter().copied());
        }
        match list.borrow().get_arr(k) {
            Some(LuaValue::String(s)) => {
                ret.extend(s.iter().copied());
            }
            Some(LuaValue::Number(n)) => {
                ret.extend(n.to_string().into_bytes());
            }
            _ => {
                return Err(RuntimeError::NotStringOrNumber);
            }
        }
    }

    stack.data_stack.push(LuaValue::String(ret));
    Ok(1)
}

// @TODO
