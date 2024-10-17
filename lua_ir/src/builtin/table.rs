use std::cell::RefCell;
use std::rc::Rc;

use crate::Chunk;
use crate::IntType;
use crate::LuaEnv;
use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaThread;
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

pub fn concat(
    env: &mut LuaEnv,
    thread: &Rc<RefCell<LuaThread>>,
    chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    let mut thread_mut = thread.borrow_mut();
    let mut it = thread_mut.pop_n(args);
    let list = match it.next() {
        Some(LuaValue::Table(table)) => table,
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
        _ => list.borrow().len() as IntType,
    };
    drop(it);

    if i > j {
        thread_mut
            .data_stack
            .push(LuaValue::String(Default::default()));
        return Ok(1);
    }
    drop(thread_mut);

    let sep = match sep {
        Some(sep) => super::tostring_impl(env, thread, chunk, sep)?,
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

    thread.borrow_mut().data_stack.push(LuaValue::String(ret));
    Ok(1)
}

pub fn insert(
    _env: &mut LuaEnv,
    thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    match args {
        0 | 1 => {
            drop(thread.borrow_mut().pop_n(args));
            Err(RuntimeError::ValueExpected)
        }
        2 => {
            let (table, value) = thread.borrow_mut().pop2(args);
            match table {
                LuaValue::Table(table) => {
                    let len = table.borrow().len();
                    table.borrow_mut().arr.insert(len + 1, value);
                }
                _ => {
                    return Err(RuntimeError::NotTable);
                }
            };
            Ok(0)
        }
        _ => {
            let (table, pos, value) = thread.borrow_mut().pop3(args);
            let table = match table {
                LuaValue::Table(table) => table,
                _ => {
                    return Err(RuntimeError::NotTable);
                }
            };
            let pos = match pos.try_to_int() {
                Some(pos) => pos,
                _ => {
                    return Err(RuntimeError::NotNumber);
                }
            };
            let len = table.borrow().len();
            if pos <= 0 || pos > len + 1 {
                return Err(RuntimeError::OutOfRange);
            }

            if pos == len + 1 {
                table.borrow_mut().arr.insert(len + 1, value);
            } else {
                let mut table_mut = table.borrow_mut();
                let split_right = table_mut.arr.split_off(&pos);
                table_mut
                    .arr
                    .extend(split_right.into_iter().map(|(idx, val)| (idx + 1, val)));
                table_mut.arr.insert(pos, value);
            }
            Ok(0)
        }
    }
}
pub fn move_(
    _env: &mut LuaEnv,
    thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    if args < 4 {
        return Err(RuntimeError::ValueExpected);
    }

    let mut thread_mut = thread.borrow_mut();
    let mut it = thread_mut.pop_n(args);

    let a1 = match it.next().unwrap() {
        LuaValue::Table(table) => table,
        _ => return Err(RuntimeError::NotTable),
    };
    let f = match it.next().unwrap().try_to_int() {
        Some(i) => i,
        None => return Err(RuntimeError::NotNumber),
    };
    let e = match it.next().unwrap().try_to_int() {
        Some(i) => i,
        None => return Err(RuntimeError::NotNumber),
    };
    let t = match it.next().unwrap().try_to_int() {
        Some(i) => i,
        None => return Err(RuntimeError::NotNumber),
    };
    let a2 = match it.next() {
        Some(LuaValue::Table(t)) => t,
        _ => Rc::clone(&a1),
    };
    drop(it);
    drop(thread_mut);

    if f <= e {
        for i in f..=e {
            let value = a1.borrow_mut().arr.remove(&i).unwrap_or_default();
            a2.borrow_mut().insert_arr(i - f + t, value);
        }
    }
    thread.borrow_mut().data_stack.push(LuaValue::Table(a2));
    Ok(1)
}
pub fn pack(
    _env: &mut LuaEnv,
    thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    let new_table = LuaTable::arr_from_iter(thread.borrow_mut().pop_n(args));
    thread
        .borrow_mut()
        .data_stack
        .push(LuaValue::Table(Rc::new(RefCell::new(new_table))));
    Ok(1)
}
pub fn remove(
    _env: &mut LuaEnv,
    thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    match args {
        0 => {
            return Err(RuntimeError::ValueExpected);
        }
        1 => {
            let list = thread.borrow_mut().pop1(args);
            match list {
                LuaValue::Table(table) => {
                    let removed = table
                        .borrow_mut()
                        .arr
                        .pop_last()
                        .map(|(_, v)| v)
                        .unwrap_or_default();
                    thread.borrow_mut().data_stack.push(removed);
                    Ok(1)
                }
                _ => Err(RuntimeError::NotTable),
            }
        }

        _ => {
            let (list, pos) = thread.borrow_mut().pop2(args);
            match list {
                LuaValue::Table(table) => {
                    let len = table.borrow().len();
                    let pos = match pos.try_to_int() {
                        Some(pos) => pos,
                        None => return Err(RuntimeError::NotNumber),
                    };

                    if pos < 0 || pos > len + 1 {
                        return Err(RuntimeError::OutOfRange);
                    }

                    if pos == 0 {
                        if len != 0 {
                            return Err(RuntimeError::OutOfRange);
                        } else {
                            thread.borrow_mut().data_stack.push(LuaValue::Nil);
                            return Ok(1);
                        }
                    } else if pos == len + 1 {
                        thread.borrow_mut().data_stack.push(LuaValue::Nil);
                        return Ok(1);
                    }

                    let mut split_right = table.borrow_mut().arr.split_off(&pos);
                    let first_pos = split_right.first_key_value().map(|(k, _)| *k);
                    if first_pos == Some(pos) {
                        let removed = split_right.pop_first().unwrap().1;
                        thread.borrow_mut().data_stack.push(removed);
                    }
                    table
                        .borrow_mut()
                        .arr
                        .extend(split_right.into_iter().map(|(idx, val)| (idx - 1, val)));
                    Ok(1)
                }
                _ => Err(RuntimeError::NotTable),
            }
        }
    }
}
pub fn sort(
    env: &mut LuaEnv,
    thread: &Rc<RefCell<LuaThread>>,
    chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    let mut thread_mut = thread.borrow_mut();
    let mut it = thread_mut.pop_n(args);
    let list = match it.next() {
        Some(LuaValue::Table(list)) => list,
        _ => return Err(RuntimeError::NotTable),
    };
    let cmp = it.next();
    drop(it);
    drop(thread_mut);

    let mut list_to_vec_elem = Vec::new();
    unpack_impl(
        Rc::clone(&list),
        1,
        list.borrow().len() as IntType,
        &mut list_to_vec_elem,
    )?;

    if list_to_vec_elem.len() < 2 {
        return Ok(0);
    }
    if let Some(cmp) = cmp {
        list_to_vec_elem.sort_unstable_by(|a, b| {
            thread.borrow_mut().data_stack.push(a.clone());
            thread.borrow_mut().data_stack.push(b.clone());
            if env
                .function_call(thread, chunk, 2, cmp.clone(), Some(1))
                .is_err()
            {
                std::cmp::Ordering::Equal
            } else {
                let ret = thread.borrow_mut().data_stack.pop().unwrap().to_bool();
                if ret {
                    std::cmp::Ordering::Less
                } else {
                    thread.borrow_mut().data_stack.push(b.clone());
                    thread.borrow_mut().data_stack.push(a.clone());
                    if env
                        .function_call(thread, chunk, 2, cmp.clone(), Some(1))
                        .is_err()
                    {
                        std::cmp::Ordering::Equal
                    } else {
                        let ret = thread.borrow_mut().data_stack.pop().unwrap().to_bool();
                        if ret {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    }
                }
            }
        });
    } else {
        list_to_vec_elem.sort_unstable_by(|a, b| {
            thread.borrow_mut().data_stack.push(a.clone());
            thread.borrow_mut().data_stack.push(b.clone());
            if env.lt(thread, chunk).is_err() {
                std::cmp::Ordering::Equal
            } else {
                let ret = thread.borrow_mut().data_stack.pop().unwrap().to_bool();
                if ret {
                    std::cmp::Ordering::Less
                } else {
                    thread.borrow_mut().data_stack.push(b.clone());
                    thread.borrow_mut().data_stack.push(a.clone());
                    if env.lt(thread, chunk).is_err() {
                        std::cmp::Ordering::Equal
                    } else {
                        let ret = thread.borrow_mut().data_stack.pop().unwrap().to_bool();
                        if ret {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    }
                }
            }
        });
    }
    list.borrow_mut().arr.extend(
        list_to_vec_elem
            .into_iter()
            .enumerate()
            .map(|(idx, val)| (idx as IntType + 1, val)),
    );
    Ok(0)
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
pub fn unpack(
    _env: &mut LuaEnv,
    thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    let mut thread_mut = thread.borrow_mut();
    let mut it = thread_mut.pop_n(args);
    let table = match it.next() {
        Some(LuaValue::Table(t)) => t,
        _ => return Err(RuntimeError::NotTable),
    };
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
        _ => table.borrow().len() as IntType,
    };
    drop(it);

    unpack_impl(table, i, j, &mut thread_mut.data_stack)
}
