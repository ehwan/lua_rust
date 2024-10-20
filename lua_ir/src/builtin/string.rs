use std::cell::RefCell;
use std::rc::Rc;

use crate::IntType;
use crate::LuaEnv;
use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;

/*
@TODO
The string library provides all its functions inside the table string.
It also sets a metatable for strings where the __index field points to the string table.
Therefore, you can use the string functions in object-oriented style.
For instance, string.byte(s,i) can be written as s:byte(i).
*/

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

pub fn dump(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("string.dump");
}
pub fn format(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("string.format");
}
pub fn gmatch(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("string.gmatch");
}
pub fn gsub(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("string.gsub");
}
pub fn match_(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("string.match");
}
pub fn pack(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("string.pack");
}
pub fn packsize(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("string.packsize");
}
pub fn unpack(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
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
pub fn byte(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    let (s, i, j) = match args {
        0 => return Err(RuntimeError::ValueExpected),
        1 => {
            let s = match env.pop() {
                LuaValue::String(s) => s,
                LuaValue::Number(n) => n.to_string().into_bytes(),
                _ => return Err(RuntimeError::NotString),
            };
            (s, 1, 1)
        }
        2 => {
            let (s, i) = env.pop2();

            let s = match s {
                LuaValue::String(s) => s,
                LuaValue::Number(n) => n.to_string().into_bytes(),
                _ => return Err(RuntimeError::NotString),
            };
            let i = match i.try_to_int() {
                Some(i) => i,
                None => return Err(RuntimeError::NotInteger),
            };

            (s, i, i)
        }
        _ => {
            env.pop_n(args - 3);
            let (s, i, j) = env.pop3();
            let s = match s {
                LuaValue::String(s) => s,
                _ => return Err(RuntimeError::NotString),
            };
            let i = match i.try_to_int() {
                Some(i) => i,
                None => return Err(RuntimeError::NotInteger),
            };
            let j = match j.try_to_int() {
                Some(j) => j,
                None => return Err(RuntimeError::NotInteger),
            };
            (s, i, j)
        }
    };
    let sub = sub_impl(&s, i, j);
    env.borrow_running_thread_mut()
        .data_stack
        .extend(sub.iter().map(|c| (*c as IntType).into()));
    Ok(sub.len())
}
pub fn sub(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::ValueExpected);
    }
    let (s, i, j) = match args {
        0 => return Err(RuntimeError::ValueExpected),
        1 => {
            env.pop();
            return Err(RuntimeError::ValueExpected);
        }
        2 => {
            let (s, i) = env.pop2();
            let s = match s {
                LuaValue::String(s) => s,
                LuaValue::Number(n) => n.to_string().into_bytes(),
                _ => return Err(RuntimeError::NotString),
            };
            let i = match i.try_to_int() {
                Some(i) => i,
                None => return Err(RuntimeError::NotInteger),
            };
            let j = s.len() as IntType;
            (s, i, j)
        }
        _ => {
            env.pop_n(args - 3);

            let (s, i, j) = env.pop3();
            let s = match s {
                LuaValue::String(s) => s,
                LuaValue::Number(n) => n.to_string().into_bytes(),
                _ => return Err(RuntimeError::NotString),
            };
            let i = match i.try_to_int() {
                Some(i) => i,
                None => return Err(RuntimeError::NotInteger),
            };
            let j = match j.try_to_int() {
                Some(j) => j,
                None => return Err(RuntimeError::NotInteger),
            };

            (s, i, j)
        }
    };

    let sub = sub_impl(&s, i, j);
    env.push(LuaValue::String(sub.to_vec()));
    Ok(1)
}

pub fn char_(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    let mut s = Vec::with_capacity(args);
    let mut thread_mut = env.borrow_running_thread_mut();
    let len = thread_mut.data_stack.len();
    for ch in thread_mut.data_stack.drain(len - args..) {
        match ch.try_to_int() {
            Some(i) => {
                if i < 0 || i > 255 {
                    return Err(RuntimeError::OutOfRangeChar);
                }
                s.push(i as u8);
            }
            None => return Err(RuntimeError::NotInteger),
        }
    }
    thread_mut.data_stack.push(LuaValue::String(s));
    Ok(1)
}

pub fn len(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    match arg {
        LuaValue::String(s) => {
            env.push((s.len() as IntType).into());
            Ok(1)
        }
        LuaValue::Number(n) => {
            let s = n.to_string().into_bytes();
            env.push((s.len() as IntType).into());
            Ok(1)
        }
        _ => Err(RuntimeError::NotString),
    }
}

pub fn lower(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    match arg {
        LuaValue::String(s) => {
            let ret = LuaValue::String(s.into_iter().map(|c| c.to_ascii_lowercase()).collect());
            env.push(ret);
            Ok(1)
        }
        LuaValue::Number(n) => {
            let ret = LuaValue::String(n.to_string().into_bytes());
            env.push(ret);
            Ok(1)
        }
        _ => return Err(RuntimeError::NotString),
    }
}
pub fn upper(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    match arg {
        LuaValue::String(s) => {
            let ret = LuaValue::String(s.into_iter().map(|c| c.to_ascii_uppercase()).collect());
            env.push(ret);
            Ok(1)
        }
        LuaValue::Number(n) => {
            let ret = LuaValue::String(n.to_string().into_bytes());
            env.push(ret);
            Ok(1)
        }
        _ => return Err(RuntimeError::NotString),
    }
}
pub fn rep(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    match args {
        0 => Err(RuntimeError::ValueExpected),
        1 => {
            env.pop();
            Err(RuntimeError::ValueExpected)
        }
        2 => {
            let (s, n) = env.pop2();
            let s = match s {
                LuaValue::String(s) => s,
                LuaValue::Number(n) => n.to_string().into_bytes(),
                _ => return Err(RuntimeError::NotString),
            };
            let n = match n.try_to_int() {
                Some(n) => n,
                None => return Err(RuntimeError::NotInteger),
            };
            if n <= 0 {
                env.push(LuaValue::String(Vec::new()));
            } else {
                env.push(LuaValue::String(s.repeat(n as usize)));
            }

            Ok(1)
        }
        _ => {
            env.pop_n(args - 3);
            let (s, n, sep) = env.pop3();
            let s = match s {
                LuaValue::String(s) => s,
                LuaValue::Number(n) => n.to_string().into_bytes(),
                _ => return Err(RuntimeError::NotString),
            };
            let n = match n.try_to_int() {
                Some(n) => n,
                None => return Err(RuntimeError::NotInteger),
            };
            let sep = match sep {
                LuaValue::String(s) => s,
                LuaValue::Number(n) => n.to_string().into_bytes(),
                _ => return Err(RuntimeError::NotString),
            };

            if n <= 0 {
                env.push(LuaValue::String(Vec::new()));
            } else {
                env.push(LuaValue::String(s.repeat(n as usize)));
            }

            let mut ret = Vec::with_capacity(s.len() * n as usize + sep.len() * (n as usize - 1));
            for i in 0..n {
                if i != 0 {
                    ret.extend_from_slice(&sep);
                }
                ret.extend_from_slice(&s);
            }
            env.push(LuaValue::String(ret));

            Ok(1)
        }
    }
}

pub fn reverse(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    let mut s = match arg {
        LuaValue::String(s) => s,
        LuaValue::Number(n) => n.to_string().into_bytes(),
        _ => return Err(RuntimeError::NotString),
    };
    s.reverse();
    env.push(LuaValue::String(s));
    Ok(1)
}
