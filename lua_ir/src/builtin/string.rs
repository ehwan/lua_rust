use std::cell::RefCell;
use std::rc::Rc;

use crate::Chunk;
use crate::IntType;
use crate::LuaEnv;
use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;
use crate::Stack;

/// init string module
pub fn init() -> Result<LuaValue, RuntimeError> {
    let mut string = LuaTable::new();
    string.insert("byte".into(), LuaFunction::from_func(byte).into());
    string.insert("sub".into(), LuaFunction::from_func(sub).into());
    string.insert("char".into(), LuaFunction::from_func(char_).into());
    string.insert("len".into(), LuaFunction::from_func(len).into());
    string.insert("lower".into(), LuaFunction::from_func(lower).into());
    string.insert("rep".into(), LuaFunction::from_func(rep).into());
    string.insert("reverse".into(), LuaFunction::from_func(reverse).into());
    string.insert("upper".into(), LuaFunction::from_func(upper).into());
    string.insert("dump".into(), LuaFunction::from_func(dump).into());
    string.insert("format".into(), LuaFunction::from_func(format).into());
    string.insert("gmatch".into(), LuaFunction::from_func(gmatch).into());
    string.insert("gsub".into(), LuaFunction::from_func(gsub).into());
    string.insert("match".into(), LuaFunction::from_func(match_).into());
    string.insert("pack".into(), LuaFunction::from_func(pack).into());
    string.insert("packsize".into(), LuaFunction::from_func(packsize).into());
    string.insert("unpack".into(), LuaFunction::from_func(unpack).into());
    Ok(LuaValue::Table(Rc::new(RefCell::new(string))))
}

pub fn dump(
    _stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("string.dump");
}
pub fn format(
    _stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("string.format");
}
pub fn gmatch(
    _stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("string.gmatch");
}
pub fn gsub(
    _stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("string.gsub");
}
pub fn match_(
    _stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("string.match");
}
pub fn pack(
    _stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("string.pack");
}
pub fn packsize(
    _stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("string.packsize");
}
pub fn unpack(
    _stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("string.unpack");
}

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
pub fn byte(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
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
pub fn sub(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
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

pub fn char_(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
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

pub fn len(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
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

pub fn lower(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
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
pub fn upper(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
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
pub fn rep(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
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

pub fn reverse(
    stack: &mut Stack,
    _env: &mut LuaEnv,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
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
