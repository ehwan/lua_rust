use lua_tokenizer::IntType;

use std::rc::Rc;

use crate::LuaEnv;
use crate::LuaFunction;
use crate::LuaNumber;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;

mod coroutine;
mod io;
mod math;
mod os;
mod string;
mod table;

const VERSION: &str = "Lua 5.4 in Rust";

/// generate default `_ENV` table
pub fn init_env() -> Result<LuaTable, RuntimeError> {
    // @TODO
    let mut env: LuaTable = LuaTable::new();
    env.insert(
        "pcall".into(),
        LuaFunction::from_func_with_expected(pcall).into(),
    );
    env.insert("xpcall".into(), LuaFunction::from_func(xpcall).into());
    env.insert("print".into(), LuaFunction::from_func(print).into());
    env.insert("rawequal".into(), LuaFunction::from_func(rawequal).into());
    env.insert("rawlen".into(), LuaFunction::from_func(rawlen).into());
    env.insert("rawget".into(), LuaFunction::from_func(rawget).into());
    env.insert("rawset".into(), LuaFunction::from_func(rawset).into());
    env.insert("type".into(), LuaFunction::from_func(type_).into());
    env.insert("tostring".into(), LuaFunction::from_func(tostring).into());
    env.insert("select".into(), LuaFunction::from_func(select).into());
    env.insert(
        "setmetatable".into(),
        LuaFunction::from_func(setmetatable).into(),
    );
    env.insert(
        "getmetatable".into(),
        LuaFunction::from_func(getmetatable).into(),
    );
    env.insert("assert".into(), LuaFunction::from_func(assert).into());
    env.insert("error".into(), LuaFunction::from_func(error).into());

    env.insert("ipairs".into(), LuaFunction::from_func(ipairs).into());
    env.insert("next".into(), LuaFunction::from_func(next).into());
    env.insert("pairs".into(), LuaFunction::from_func(pairs).into());
    env.insert("tonumber".into(), LuaFunction::from_func(tonumber).into());
    env.insert(
        "collectgarbage".into(),
        LuaFunction::from_func(collectgarbage).into(),
    );
    env.insert("load".into(), LuaFunction::from_func(load).into());
    env.insert("loadfile".into(), LuaFunction::from_func(loadfile).into());
    env.insert("dofile".into(), LuaFunction::from_func(dofile).into());

    env.insert("_VERSION".into(), VERSION.into());

    env.insert("string".into(), string::init()?.into());
    env.insert("math".into(), math::init()?.into());
    env.insert("table".into(), table::init()?.into());
    env.insert("coroutine".into(), coroutine::init()?.into());
    env.insert("os".into(), os::init()?.into());
    env.insert("io".into(), io::init()?.into());

    // `_G` will be added in `VM::new_stack()` or `Stack::new()`
    Ok(env)
}

fn load(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("load");
}
fn loadfile(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("loadfile");
}
fn dofile(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("dofile");
}
fn tonumber(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("tonumber");
}
fn collectgarbage(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("collectgarbage");
}
fn xpcall(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("xpcall");
}

pub fn pcall(
    env: &mut LuaEnv,
    args: usize,
    expected_ret: Option<usize>,
) -> Result<(), RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "value"));
    }

    let coroutine_count = env.coroutines.len();
    let thread_borrow = env.running_thread().borrow();
    let mut thread_state = thread_borrow.to_state();
    thread_state.data_stack -= args;
    let func = thread_borrow.data_stack[thread_state.data_stack].clone();
    drop(thread_borrow);

    // @TODO use call stack frame
    match expected_ret {
        Some(0) => match env.function_call(args - 1, func, Some(0)) {
            Ok(_) => Ok(()),
            Err(_e) => {
                env.coroutines.truncate(coroutine_count);
                env.running_thread().borrow_mut().from_state(thread_state);
                Ok(())
            }
        },
        Some(expected_ret) => match env.function_call(args - 1, func, Some(expected_ret - 1)) {
            Ok(_) => {
                env.running_thread().borrow_mut().data_stack[thread_state.data_stack] =
                    LuaValue::Boolean(true);
                Ok(())
            }
            Err(e) => {
                env.coroutines.truncate(coroutine_count);
                let error_obj = e.into_lua_value(env);
                let mut thread_mut = env.running_thread().borrow_mut();
                thread_mut.from_state(thread_state);
                thread_mut.data_stack.push(false.into());
                if expected_ret > 1 {
                    thread_mut.data_stack.push(error_obj);
                }
                if expected_ret > 2 {
                    thread_mut
                        .data_stack
                        .extend(std::iter::repeat(LuaValue::Nil).take(expected_ret - 2));
                }
                Ok(())
            }
        },
        None => match env.function_call(args - 1, func, None) {
            Ok(_) => {
                env.running_thread().borrow_mut().data_stack[thread_state.data_stack] =
                    LuaValue::Boolean(true);
                Ok(())
            }
            Err(e) => {
                env.coroutines.truncate(coroutine_count);
                let error_obj = e.into_lua_value(env);
                let mut thread_mut = env.running_thread().borrow_mut();
                thread_mut.from_state(thread_state);
                thread_mut.data_stack.push(false.into());
                thread_mut.data_stack.push(error_obj);
                Ok(())
            }
        },
    }
}
pub fn print(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    for i in 0..args {
        if i > 0 {
            print!("\t");
        }
        let ith = env.top_i(args - i - 1);
        env.push(ith);
        env.tostring()?;
        let s = env.pop();
        if let LuaValue::String(s) = s {
            print!("{}", s);
        } else {
            unreachable!("string expected");
        }
    }
    println!();
    env.pop_n(args);
    Ok(0)
}
pub fn rawequal(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "value"));
    } else if args == 1 {
        env.pop();
        return Err(RuntimeError::new_empty_argument(2, "value"));
    } else if args > 2 {
        env.pop_n(args - 2);
    }
    let (lhs, rhs) = env.pop2();
    env.push(LuaValue::Boolean(lhs == rhs));
    Ok(1)
}
pub fn rawlen(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "table or string"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    let len = match arg {
        LuaValue::String(s) => s.len() as IntType,
        LuaValue::Table(t) => t.borrow().len(),
        _ => {
            return Err(RuntimeError::BadArgument(
                1,
                Box::new(RuntimeError::Expected(
                    "table or string",
                    arg.type_str().into(),
                )),
            ))
        }
    };
    env.push(len.into());
    Ok(1)
}
pub fn rawget(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "table"));
    } else if args == 1 {
        env.pop();
        return Err(RuntimeError::new_empty_argument(2, "value"));
    } else if args > 2 {
        env.pop_n(args - 2);
    }
    let (table, key) = env.pop2();
    let table = match table {
        LuaValue::Table(table) => table,
        _ => {
            return Err(RuntimeError::BadArgument(
                1,
                Box::new(RuntimeError::Expected("table", table.type_str().into())),
            ));
        }
    };
    env.push(table.borrow().get(&key).cloned().unwrap_or_default());
    Ok(1)
}
pub fn rawset(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    match args {
        0 => return Err(RuntimeError::new_empty_argument(1, "table")),
        1 => {
            env.pop();
            return Err(RuntimeError::new_empty_argument(2, "value"));
        }
        2 => {
            env.pop2();
            return Err(RuntimeError::new_empty_argument(3, "value"));
        }
        args => {
            env.pop_n(args - 3);
        }
    }
    let (table, key, value) = env.pop3();

    let table = match table {
        LuaValue::Table(table) => table,
        _ => {
            return Err(RuntimeError::BadArgument(
                1,
                Box::new(RuntimeError::Expected("table", table.type_str().into())),
            ));
        }
    };
    if key.is_nil() {
        return Err(RuntimeError::TableIndexNil);
    } else if key.is_nan() {
        return Err(RuntimeError::TableIndexNan);
    }
    table.borrow_mut().insert(key, value);
    env.push(LuaValue::Table(table));
    Ok(1)
}
pub fn select(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    }

    let index = env.top_i(args - 1);
    if let LuaValue::String(s) = &index {
        if s[0] == b'#' {
            env.pop_n(args);
            env.push(((args - 1) as IntType).into());
            return Ok(1);
        }
    }
    let index = index
        .try_to_int()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    let index = if index == 0 {
        env.pop_n(args);
        return Err(RuntimeError::BadArgument(
            1,
            Box::new(RuntimeError::IndexOutOfRange),
        ));
    } else if index < 0 {
        if (-index) as usize > args - 1 {
            env.pop_n(args);
            return Err(RuntimeError::BadArgument(
                1,
                Box::new(RuntimeError::IndexOutOfRange),
            ));
        } else {
            (args as IntType + index - 1) as usize
        }
    } else {
        if index as usize > args - 1 {
            env.pop_n(args);
            return Ok(0);
        } else {
            index as usize - 1
        }
    };

    let mut thread_mut = env.borrow_running_thread_mut();
    let len = thread_mut.data_stack.len();
    drop(thread_mut.data_stack.drain(len - args..=len - args + index));
    Ok(args - 1 - index)
}

pub fn setmetatable(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "table"));
    } else if args == 1 {
        env.pop();
        return Err(RuntimeError::new_empty_argument(2, "nil or table"));
    } else if args > 2 {
        env.pop_n(args - 2);
    }
    let (table, meta) = env.pop2();

    if let LuaValue::Table(table) = table {
        // check __metatable is defined
        if let Some(meta_old) = &table.borrow().meta {
            if meta_old
                .borrow()
                .map
                .contains_key(&LuaValue::from("__metatable"))
            {
                // try to modify protected metatable (__metatable defined)
                return Err(RuntimeError::Custom(
                    "cannot change a protected metatable".into(),
                ));
            }
        }
        match meta {
            LuaValue::Nil => {
                table.borrow_mut().meta = None;
                env.push(LuaValue::Table(table));
                Ok(1)
            }
            LuaValue::Table(meta) => {
                table.borrow_mut().meta = Some(meta);
                env.push(LuaValue::Table(table));
                Ok(1)
            }
            _ => Err(RuntimeError::BadArgument(
                2,
                Box::new(RuntimeError::Expected(
                    "nil or table",
                    meta.type_str().into(),
                )),
            )),
        }
    } else {
        Err(RuntimeError::BadArgument(
            1,
            Box::new(RuntimeError::Expected("table", table.type_str().into())),
        ))
    }
}
pub fn getmetatable(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "value"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let table = env.pop();
    match table {
        LuaValue::Table(table) => {
            if let Some(meta) = &table.borrow().meta {
                // check __metatable is defined
                if let Some(assoc) = meta.borrow().get(&"__metatable".into()) {
                    env.push(assoc.clone());
                } else {
                    env.push(LuaValue::Table(Rc::clone(meta)));
                }
            } else {
                env.push(LuaValue::Nil);
            }
            Ok(1)
        }
        _ => Err(RuntimeError::BadArgument(
            1,
            Box::new(RuntimeError::Expected("table", table.type_str().into())),
        )),
    }
}

pub fn tostring(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "value"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    env.tostring()?;
    Ok(1)
}

pub fn type_(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "value"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    env.push(arg.type_str().into());
    Ok(1)
}

pub fn assert(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "value"));
    }
    let thread = env.borrow_running_thread();
    if thread.data_stack[thread.data_stack.len() - args].to_bool() {
        Ok(args)
    } else {
        drop(thread);

        if args > 2 {
            env.pop_n(args - 2);
        }

        let error = if args == 1 {
            env.pop();
            "assertion failed!".into()
        } else {
            let (_, error) = env.pop2();
            error
        };
        Err(RuntimeError::Custom(error))
    }
}

pub fn error(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    let (error, level) = match args {
        0 => (LuaValue::Nil, 1),
        1 => {
            let error = env.pop();
            (error, 1)
        }
        _ => {
            env.pop_n(args - 2);
            let (error, level) = env.pop2();
            let level = level
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;
            (error, level)
        }
    };

    match level {
        // level 0, no additional info
        0 => Err(RuntimeError::Custom(error)),
        // otherwise, add additional info if the error is a string
        _level =>
        // @TODO
        {
            Err(RuntimeError::Custom(error))
        }
    }
}

/// iterator function for `ipairs`
fn ipair_next(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::Error);
    } else if args > 2 {
        env.pop_n(args - 2);
    }

    let (table, key) = env.pop2();
    // check key has integer representation
    match key {
        LuaValue::Number(LuaNumber::Int(mut n)) => {
            n += 1;
            env.push2(table, n.into());
            env.index()?;
            let ret = env.pop();
            if ret == LuaValue::Nil {
                env.push(LuaValue::Nil);
                Ok(1)
            } else {
                env.push2(n.into(), ret);
                Ok(2)
            }
        }
        // this should not happen; since this function is only called from `ipairs`, privately
        _ => Err(RuntimeError::Error),
    }
}
pub fn ipairs(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "value"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }

    // no check for table here
    let table = env.pop();
    env.push3(
        LuaFunction::from_func(ipair_next).into(),
        table,
        (0 as IntType).into(),
    );
    Ok(3)
}

pub fn next(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    let (table, index) = match args {
        0 => return Err(RuntimeError::Error),
        1 => (env.pop(), LuaValue::Nil),
        _ => {
            env.pop_n(args - 2);
            env.pop2()
        }
    };

    // iterate table through
    // integer keys (array part) first, then hash part

    match table {
        LuaValue::Table(table) => {
            match index {
                // index is nil, get first key for iteration
                LuaValue::Nil => {
                    if let Some((k, v)) = table.borrow().arr.first_key_value() {
                        env.push2((*k).into(), v.clone());
                        Ok(2)
                    } else {
                        // no array part
                        if let Some((k, v)) = table.borrow().map.first() {
                            env.push2(k.clone(), v.clone());
                            Ok(2)
                        } else {
                            // no hash part
                            env.push(LuaValue::Nil);
                            Ok(1)
                        }
                    }
                }

                // index is integer, get next element in array part
                LuaValue::Number(LuaNumber::Int(n)) => {
                    let table = table.borrow();
                    let mut range_it = table.arr.range(n..);
                    if range_it.next().map(|(k, _)| *k) == Some(n) {
                        if let Some((k, v)) = range_it.next() {
                            env.push2((*k).into(), v.clone());
                            Ok(2)
                        } else {
                            // n is the last element, check hash part
                            if let Some((k, v)) = table.map.first() {
                                env.push2(k.clone(), v.clone());
                                Ok(2)
                            } else {
                                // no hash part
                                env.push(LuaValue::Nil);
                                Ok(1)
                            }
                        }
                    } else {
                        Err(RuntimeError::Error)
                    }
                }

                index => {
                    // hash part
                    if let Some(cur_idx) = table.borrow().map.get_index_of(&index) {
                        if let Some((k, v)) = table.borrow().map.get_index(cur_idx + 1) {
                            env.push2(k.clone(), v.clone());
                            Ok(2)
                        } else {
                            // no more elements
                            env.push(LuaValue::Nil);
                            Ok(1)
                        }
                    } else {
                        Err(RuntimeError::Error)
                    }
                }
            }
        }
        // @TODO next() with non-table was possible...
        _ => Err(RuntimeError::BadArgument(
            1,
            Box::new(RuntimeError::Expected("table", table.type_str().into())),
        )),
    }
}

// @TODO __pair metamethod
pub fn pairs(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "value"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }

    let table = env.pop();
    let table = if let LuaValue::Table(table) = table {
        table
    } else {
        // @TODO pair() with non-table was possible...
        return Err(RuntimeError::BadArgument(
            1,
            Box::new(RuntimeError::Expected("table", table.type_str().into())),
        ));
    };

    env.push3(
        LuaFunction::from_func(next).into(),
        LuaValue::Table(table),
        LuaValue::Nil,
    );
    Ok(3)
}
