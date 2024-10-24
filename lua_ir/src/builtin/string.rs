use std::cell::RefCell;
use std::rc::Rc;

use crate::IntType;
use crate::LuaEnv;
use crate::LuaFunction;
use crate::LuaString;
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
    if s.is_empty() {
        return s;
    }
    if i < 0 {
        i = s.len() as IntType + i + 1;
    }
    if i < 1 {
        i = 1;
    } else if i > s.len() as IntType {
        i = s.len() as IntType;
    }

    if j < 0 {
        j = s.len() as IntType + j + 1;
    }
    if j < 1 {
        j = 1;
    } else if j > s.len() as IntType {
        j = s.len() as IntType;
    }

    if i > j {
        &s[0..0]
    } else {
        &s[((i - 1) as usize)..(j as usize)]
    }
}
pub fn byte(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    let (s, i, j) = match args {
        0 => return Err(RuntimeError::new_empty_argument(1, "string")),
        1 => {
            let s = env.pop();
            let s = match s {
                LuaValue::String(s) => s,
                LuaValue::Number(n) => LuaString::from_string(n.to_string()),
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("string", s.type_str().into())),
                    ))
                }
            };
            (s, 1, 1)
        }
        2 => {
            let (s, i) = env.pop2();

            let s = match s {
                LuaValue::String(s) => s,
                LuaValue::Number(n) => LuaString::from_string(n.to_string()),
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("string", s.type_str().into())),
                    ))
                }
            };
            let i = i
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;

            (s, i, i)
        }
        _ => {
            env.pop_n(args - 3);
            let (s, i, j) = env.pop3();
            let s = match s {
                LuaValue::String(s) => s,
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("string", s.type_str().into())),
                    ))
                }
            };
            let i = i
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;
            let j = j
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(3, Box::new(e)))?;
            (s, i, j)
        }
    };
    let sub = sub_impl(s.as_bytes(), i, j);
    env.borrow_running_thread_mut()
        .data_stack
        .extend(sub.iter().map(|c| (*c as IntType).into()));
    Ok(sub.len())
}
pub fn sub(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
    } else if args == 1 {
        env.pop();
    }
    let (s, i, j) = match args {
        0 => return Err(RuntimeError::new_empty_argument(1, "string")),
        1 => {
            env.pop();
            return Err(RuntimeError::new_empty_argument(1, "number"));
        }
        2 => {
            let (s, i) = env.pop2();
            let s = match s {
                LuaValue::String(s) => s,
                LuaValue::Number(n) => LuaString::from_string(n.to_string()),
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("string", s.type_str().into())),
                    ))
                }
            };
            let i = i
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;
            let j = s.len() as IntType;
            (s, i, j)
        }
        _ => {
            env.pop_n(args - 3);

            let (s, i, j) = env.pop3();
            let s = match s {
                LuaValue::String(s) => s,
                LuaValue::Number(n) => LuaString::from_string(n.to_string()),
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("string", s.type_str().into())),
                    ))
                }
            };
            let i = i
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;
            let j = j
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(3, Box::new(e)))?;

            (s, i, j)
        }
    };

    let sub = sub_impl(s.as_bytes(), i, j);
    env.push(LuaString::from_slice(sub).into());
    Ok(1)
}

pub fn char_(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    let mut s = Vec::with_capacity(args);
    let mut thread_mut = env.borrow_running_thread_mut();
    let len = thread_mut.data_stack.len();
    for (idx, ch) in thread_mut.data_stack.drain(len - args..).enumerate() {
        let ch = ch
            .try_to_int()
            .map_err(|e| RuntimeError::BadArgument(idx + 1, Box::new(e)))?;
        if ch < 0 || ch > 255 {
            return Err(RuntimeError::BadArgument(
                idx + 1,
                Box::new(RuntimeError::ValueOutOfRange),
            ));
        }
        s.push(ch as u8);
    }
    thread_mut.data_stack.push(LuaString::from_vec(s).into());
    Ok(1)
}

pub fn len(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "string"));
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
        _ => {
            return Err(RuntimeError::BadArgument(
                1,
                Box::new(RuntimeError::Expected("string", arg.type_str().into())),
            ))
        }
    }
}

pub fn lower(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "string"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    match arg {
        LuaValue::String(s) => {
            let ret = LuaValue::String(s.into_mapped(|c| c.to_ascii_lowercase()));
            env.push(ret);
            Ok(1)
        }
        LuaValue::Number(n) => {
            let ret = LuaValue::String(LuaString::from_string(n.to_string()));
            env.push(ret);
            Ok(1)
        }
        _ => {
            return Err(RuntimeError::BadArgument(
                1,
                Box::new(RuntimeError::Expected("string", arg.type_str().into())),
            ))
        }
    }
}
pub fn upper(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "string"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    match arg {
        LuaValue::String(s) => {
            let ret = LuaValue::String(s.into_mapped(|c| c.to_ascii_uppercase()));
            env.push(ret);
            Ok(1)
        }
        LuaValue::Number(n) => {
            let ret = LuaValue::String(LuaString::from_string(n.to_string()));
            env.push(ret);
            Ok(1)
        }
        _ => {
            return Err(RuntimeError::BadArgument(
                1,
                Box::new(RuntimeError::Expected("string", arg.type_str().into())),
            ))
        }
    }
}
pub fn rep(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    match args {
        0 => Err(RuntimeError::new_empty_argument(1, "string")),
        1 => {
            env.pop();
            Err(RuntimeError::new_empty_argument(2, "number"))
        }
        2 => {
            let (s, n) = env.pop2();
            let s = match s {
                LuaValue::String(s) => s,
                LuaValue::Number(n) => LuaString::from_string(n.to_string()),
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("string", s.type_str().into())),
                    ))
                }
            };
            let n = n
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;
            if n <= 0 {
                env.push(LuaString::from_static_str("").into());
            } else {
                env.push(LuaString::from_vec(s.into_vec().repeat(n as usize)).into());
            }

            Ok(1)
        }
        _ => {
            env.pop_n(args - 3);
            let (s, n, sep) = env.pop3();
            let s = match s {
                LuaValue::String(s) => s,
                LuaValue::Number(n) => LuaString::from_string(n.to_string()),
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("string", s.type_str().into())),
                    ))
                }
            };
            let n = n
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;
            let sep = match sep {
                LuaValue::Nil => LuaString::from_static_str(""),
                LuaValue::String(s) => s,
                LuaValue::Number(n) => LuaString::from_string(n.to_string()),
                _ => {
                    return Err(RuntimeError::BadArgument(
                        3,
                        Box::new(RuntimeError::Expected("string", sep.type_str().into())),
                    ))
                }
            };

            if n <= 0 {
                env.push(LuaString::from_static_str("").into());
            } else {
                let mut ret =
                    Vec::with_capacity(s.len() * n as usize + sep.len() * (n as usize - 1));
                for i in 0..n {
                    if i != 0 {
                        ret.extend_from_slice(sep.as_bytes());
                    }
                    ret.extend_from_slice(s.as_bytes());
                }
                env.push(LuaString::from_vec(ret).into());
            }

            Ok(1)
        }
    }
}

pub fn reverse(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "string"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    let s = match arg {
        LuaValue::String(s) => s,
        LuaValue::Number(n) => LuaString::from_string(n.to_string()),
        _ => {
            return Err(RuntimeError::BadArgument(
                1,
                Box::new(RuntimeError::Expected("string", arg.type_str().into())),
            ))
        }
    };
    let mut s = s.into_vec();
    s.reverse();
    env.push(LuaString::from_vec(s).into());
    Ok(1)
}
