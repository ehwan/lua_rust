use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::IntType;
use crate::LuaEnv;
use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;

/// init table module
pub fn init() -> Result<LuaValue, RuntimeError> {
    let mut table = LuaTable::new();

    table.insert("concat".into(), LuaFunction::from_func(concat).into());
    table.insert("insert".into(), LuaFunction::from_func(insert).into());
    table.insert("move".into(), LuaFunction::from_func(move_).into());
    table.insert("pack".into(), LuaFunction::from_func(pack).into());
    table.insert("remove".into(), LuaFunction::from_func(remove).into());
    table.insert("sort".into(), LuaFunction::from_func(sort).into());
    table.insert("unpack".into(), LuaFunction::from_func(unpack).into());

    Ok(LuaValue::Table(Rc::new(RefCell::new(table))))
}

pub fn concat(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    let (list, sep, i, j) = match args {
        0 => return Err(RuntimeError::new_empty_argument(1, "table")),
        1 => {
            let list = env.pop();
            let list = match list {
                LuaValue::Table(table) => table,
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("table", list.type_str().into())),
                    ))
                }
            };
            let i = 1 as IntType;
            let j = list.borrow().len();
            (list, Vec::new(), i, j)
        }
        2 => {
            let (list, sep) = env.pop2();
            let list = match list {
                LuaValue::Table(table) => table,
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("table", list.type_str().into())),
                    ))
                }
            };
            let sep = match sep {
                LuaValue::Nil => Vec::new(),
                LuaValue::String(s) => s,
                LuaValue::Number(n) => n.to_string().into_bytes(),
                _ => {
                    return Err(RuntimeError::BadArgument(
                        2,
                        Box::new(RuntimeError::Expected("string", sep.type_str().into())),
                    ))
                }
            };
            let i = 1 as IntType;
            let j = list.borrow().len();
            (list, sep, i, j)
        }
        3 => {
            let (list, sep, i) = env.pop3();

            let list = match list {
                LuaValue::Table(table) => table,
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("table", list.type_str().into())),
                    ))
                }
            };
            let sep = match sep {
                LuaValue::Nil => Vec::new(),
                LuaValue::String(s) => s,
                LuaValue::Number(n) => n.to_string().into_bytes(),
                _ => {
                    return Err(RuntimeError::BadArgument(
                        2,
                        Box::new(RuntimeError::Expected("string", sep.type_str().into())),
                    ))
                }
            };
            let i = i
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(3, Box::new(e)))?;
            let j = list.borrow().len();

            (list, sep, i, j)
        }
        _ => {
            env.pop_n(args - 4);
            let (list, sep, i, j) = env.pop4();
            let list = match list {
                LuaValue::Table(table) => table,
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("table", list.type_str().into())),
                    ))
                }
            };
            let sep = match sep {
                LuaValue::Nil => Vec::new(),
                LuaValue::String(s) => s,
                LuaValue::Number(n) => n.to_string().into_bytes(),
                _ => {
                    return Err(RuntimeError::BadArgument(
                        2,
                        Box::new(RuntimeError::Expected("string", sep.type_str().into())),
                    ))
                }
            };
            let i = i
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(3, Box::new(e)))?;
            let j = j
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(4, Box::new(e)))?;
            (list, sep, i, j)
        }
    };
    if i > j {
        env.push(LuaValue::String(Vec::new()));
        return Ok(1);
    }

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
            elem => {
                // @TODO  invalid value (nil) at index 2 in table for 'concat'
                return Err(RuntimeError::Custom(
                    format!(
                        "invalid value ({}) at index {} in table for 'concat'",
                        elem.map(|val| val.type_str()).unwrap_or("nil"),
                        k
                    )
                    .into(),
                ));
            }
        }
    }
    env.push(LuaValue::String(ret));
    Ok(1)
}

pub fn insert(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    match args {
        0 => return Err(RuntimeError::new_empty_argument(1, "table")),
        1 => {
            env.pop();
            Err(RuntimeError::Custom(
                "wrong number of arguments to 'insert'".into(),
            ))
        }
        2 => {
            let (table, value) = env.pop2();
            match table {
                LuaValue::Table(table) => {
                    let len = table.borrow().len();
                    table.borrow_mut().arr.insert(len + 1, value);
                }
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("table", table.type_str().into())),
                    ));
                }
            };
            Ok(0)
        }
        _ => {
            env.pop_n(args - 3);
            let (table, pos, value) = env.pop3();
            let table = match table {
                LuaValue::Table(table) => table,
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("table", table.type_str().into())),
                    ));
                }
            };
            let pos = pos
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;
            let len = table.borrow().len();
            if pos <= 0 || pos > len + 1 {
                return Err(RuntimeError::BadArgument(
                    2,
                    Box::new(RuntimeError::PositionOutOfBounds),
                ));
            }

            if pos == len + 1 {
                table.borrow_mut().arr.insert(len + 1, value);
            } else {
                let mut table_mut = table.borrow_mut();
                let split_right = table_mut.arr.split_off(&pos);
                table_mut.arr.insert(pos, value);
                table_mut
                    .arr
                    .extend(split_right.into_iter().map(|(idx, val)| (idx + 1, val)));
            }
            Ok(0)
        }
    }
}
pub fn move_(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    match args {
        0 => return Err(RuntimeError::new_empty_argument(1, "table")),
        1 => {
            env.pop();
            return Err(RuntimeError::new_empty_argument(2, "number"));
        }
        2 => {
            env.pop2();
            return Err(RuntimeError::new_empty_argument(3, "number"));
        }
        3 => {
            env.pop3();
            return Err(RuntimeError::new_empty_argument(4, "number"));
        }
        _ => {}
    }

    let (a1, f, e, t, a2) = match args {
        4 => {
            let (a1, f, e, t) = env.pop4();
            let a1 = match a1 {
                LuaValue::Table(table) => table,
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("table", a1.type_str().into())),
                    ))
                }
            };
            let a2 = Rc::clone(&a1);
            (a1, f, e, t, a2)
        }
        _ => {
            env.pop_n(args - 5);

            let a2 = env.pop();
            let (a1, f, e, t) = env.pop4();
            let a1 = match a1 {
                LuaValue::Table(table) => table,
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("table", a1.type_str().into())),
                    ))
                }
            };
            let a2 = match a2 {
                LuaValue::Table(table) => table,
                _ => {
                    return Err(RuntimeError::BadArgument(
                        5,
                        Box::new(RuntimeError::Expected("table", a2.type_str().into())),
                    ))
                }
            };

            (a1, f, e, t, a2)
        }
    };

    let f = f
        .try_to_int()
        .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;
    let e = e
        .try_to_int()
        .map_err(|e| RuntimeError::BadArgument(3, Box::new(e)))?;
    let t = t
        .try_to_int()
        .map_err(|e| RuntimeError::BadArgument(4, Box::new(e)))?;

    if f <= e {
        for i in f..=e {
            let value = a1.borrow_mut().arr.remove(&i).unwrap_or_default();
            a2.borrow_mut().insert_arr(i - f + t, value);
        }
    }
    env.push(LuaValue::Table(a2));
    Ok(1)
}
pub fn pack(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    let mut new_table = LuaTable::new();
    let mut thread_mut = env.borrow_running_thread_mut();
    let len = thread_mut.data_stack.len();
    new_table.arr = BTreeMap::from_iter(
        thread_mut
            .data_stack
            .drain(len - args..)
            .enumerate()
            .map(|(idx, value)| (idx as IntType + 1, value)),
    );
    new_table.map.insert("n".into(), (args as IntType).into());
    thread_mut.data_stack.push(new_table.into());
    Ok(1)
}
pub fn remove(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    match args {
        0 => {
            return Err(RuntimeError::new_empty_argument(1, "table"));
        }
        1 => {
            let list = env.pop();
            match list {
                LuaValue::Table(table) => {
                    let removed = table
                        .borrow_mut()
                        .arr
                        .pop_last()
                        .map(|(_, v)| v)
                        .unwrap_or_default();
                    env.push(removed);
                    Ok(1)
                }
                _ => Err(RuntimeError::BadArgument(
                    1,
                    Box::new(RuntimeError::Expected("table", list.type_str().into())),
                )),
            }
        }
        _ => {
            env.pop_n(args - 2);
            let (list, pos) = env.pop2();
            match list {
                LuaValue::Table(table) => {
                    let len = table.borrow().len();
                    let pos = pos
                        .try_to_int()
                        .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;
                    if pos < 0 || pos > len + 1 {
                        return Err(RuntimeError::BadArgument(
                            2,
                            Box::new(RuntimeError::PositionOutOfBounds),
                        ));
                    }

                    if pos == 0 {
                        if len != 0 {
                            return Err(RuntimeError::BadArgument(
                                2,
                                Box::new(RuntimeError::PositionOutOfBounds),
                            ));
                        } else {
                            env.push(LuaValue::Nil);
                            return Ok(1);
                        }
                    } else if pos == len + 1 {
                        env.push(LuaValue::Nil);
                        return Ok(1);
                    }

                    let mut split_right = table.borrow_mut().arr.split_off(&pos);
                    let first_pos = split_right.first_key_value().map(|(k, _)| *k);
                    if first_pos == Some(pos) {
                        let removed = split_right.pop_first().unwrap().1;
                        env.push(removed);
                    }
                    table
                        .borrow_mut()
                        .arr
                        .extend(split_right.into_iter().map(|(idx, val)| (idx - 1, val)));
                    Ok(1)
                }
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("table", list.type_str().into())),
                    ))
                }
            }
        }
    }
}
pub fn sort(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    match args {
        0 => return Err(RuntimeError::new_empty_argument(1, "table")),
        1 => {
            let list = env.pop();
            let list = match list {
                LuaValue::Table(table) => table,
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("table", list.type_str().into())),
                    ))
                }
            };
            if list.borrow().len() < 2 {
                return Ok(0);
            }
            let mut list_to_vec_elem = Vec::new();
            unpack_impl(
                Rc::clone(&list),
                1,
                list.borrow().len() as IntType,
                &mut list_to_vec_elem,
            )?;
            list_to_vec_elem.sort_unstable_by(|a, b| {
                env.push2(a.clone(), b.clone());
                if env.lt().is_err() {
                    std::cmp::Ordering::Equal
                } else {
                    let ret = env.pop().to_bool();
                    if ret {
                        std::cmp::Ordering::Less
                    } else {
                        env.push2(b.clone(), a.clone());
                        if env.lt().is_err() {
                            std::cmp::Ordering::Equal
                        } else {
                            let ret = env.pop().to_bool();
                            if ret {
                                std::cmp::Ordering::Greater
                            } else {
                                std::cmp::Ordering::Equal
                            }
                        }
                    }
                }
            });
            list.borrow_mut().arr.extend(
                list_to_vec_elem
                    .into_iter()
                    .enumerate()
                    .map(|(idx, val)| (idx as IntType + 1, val)),
            );
            Ok(0)
        }
        _ => {
            env.pop_n(args - 2);
            let (list, cmp) = env.pop2();

            let list = match list {
                LuaValue::Table(table) => table,
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("table", list.type_str().into())),
                    ))
                }
            };
            if list.borrow().len() < 2 {
                return Ok(0);
            }
            let mut list_to_vec_elem = Vec::new();
            unpack_impl(
                Rc::clone(&list),
                1,
                list.borrow().len() as IntType,
                &mut list_to_vec_elem,
            )?;
            list_to_vec_elem.sort_unstable_by(|a, b| {
                env.push2(a.clone(), b.clone());
                if env.function_call(2, cmp.clone(), Some(1)).is_err() {
                    std::cmp::Ordering::Equal
                } else {
                    let ret = env.pop().to_bool();
                    if ret {
                        std::cmp::Ordering::Less
                    } else {
                        env.push2(b.clone(), a.clone());
                        if env.function_call(2, cmp.clone(), Some(1)).is_err() {
                            std::cmp::Ordering::Equal
                        } else {
                            let ret = env.pop().to_bool();
                            if ret {
                                std::cmp::Ordering::Greater
                            } else {
                                std::cmp::Ordering::Equal
                            }
                        }
                    }
                }
            });
            list.borrow_mut().arr.extend(
                list_to_vec_elem
                    .into_iter()
                    .enumerate()
                    .map(|(idx, val)| (idx as IntType + 1, val)),
            );
            Ok(0)
        }
    }
}

fn unpack_impl(
    table: Rc<RefCell<LuaTable>>,
    mut i: IntType,
    j: IntType,
    out: &mut Vec<LuaValue>,
) -> Result<usize, RuntimeError> {
    if j < i {
        return Ok(0);
    }
    let len = j - i + 1;
    out.reserve(len as usize);

    for (idx, value) in table.borrow().arr.range(i..=j) {
        if i < *idx {
            let nil_count = *idx - i;

            out.resize_with(out.len() + nil_count as usize, Default::default);
        }
        // maybe unwrapped add?
        i = *idx + 1;
        out.push(value.clone());
    }
    if i <= j {
        let nil_count = j - i + 1;
        out.resize_with(out.len() + nil_count as usize, Default::default);
    }

    Ok(len as usize)
}
pub fn unpack(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    let (list, i, j) = match args {
        0 => {
            return Err(RuntimeError::AttemptToGetLengthOf("nil"));
        }
        1 => {
            let list = env.pop();
            let list = match list {
                LuaValue::Table(table) => table,
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("table", list.type_str().into())),
                    ))
                }
            };
            let i = 1;
            let j = list.borrow().len() as IntType;
            (list, i, j)
        }
        2 => {
            let (list, i) = env.pop2();
            let list = match list {
                LuaValue::Table(table) => table,
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("table", list.type_str().into())),
                    ))
                }
            };
            let i = i
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;
            let j = list.borrow().len() as IntType;
            (list, i, j)
        }
        _ => {
            env.pop_n(args - 3);
            let (list, i, j) = env.pop3();
            let list = match list {
                LuaValue::Table(table) => table,
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("table", list.type_str().into())),
                    ))
                }
            };
            let i = i
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;
            let j = j
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(3, Box::new(e)))?;
            (list, i, j)
        }
    };
    let mut thread_mut = env.borrow_running_thread_mut();
    unpack_impl(list, i, j, &mut thread_mut.data_stack)
}
