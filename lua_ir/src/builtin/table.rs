use std::cell::RefCell;
use std::rc::Rc;

use crate::Chunk;
use crate::IntType;
use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;
use crate::Stack;

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
        _ => list.borrow().len() as IntType,
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

pub fn insert(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    match args {
        0 | 1 => {
            drop(stack.pop_n(args));
            Err(RuntimeError::ValueExpected)
        }
        2 => {
            let (table, value) = stack.pop2(args);
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
            let (table, pos, value) = stack.pop3(args);
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
                let split_right = table.borrow_mut().arr.split_off(&pos);
                for (idx, val) in split_right.into_iter() {
                    table.borrow_mut().arr.insert(idx + 1, val);
                }
                table.borrow_mut().arr.insert(pos, value);
            }
            Ok(0)
        }
    }
}
pub fn move_(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args < 4 {
        return Err(RuntimeError::ValueExpected);
    }

    let mut it = stack.pop_n(args);

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

    if f <= e {
        for i in f..=e {
            let value = a1.borrow_mut().arr.remove(&i).unwrap_or_default();
            a2.borrow_mut().insert_arr(i - f + t, value);
        }
    }
    stack.data_stack.push(LuaValue::Table(a2));
    Ok(1)
}
pub fn pack(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    let new_table = LuaTable::arr_from_iter(stack.pop_n(args));
    stack
        .data_stack
        .push(LuaValue::Table(Rc::new(RefCell::new(new_table))));
    Ok(1)
}
pub fn remove(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    match args {
        0 => {
            return Err(RuntimeError::ValueExpected);
        }
        1 => {
            let list = stack.pop1(args);
            match list {
                LuaValue::Table(table) => {
                    let removed = table
                        .borrow_mut()
                        .arr
                        .pop_last()
                        .map(|(_, v)| v)
                        .unwrap_or_default();
                    stack.data_stack.push(removed);
                    Ok(1)
                }
                _ => Err(RuntimeError::NotTable),
            }
        }

        _ => {
            let (list, pos) = stack.pop2(args);
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
                            stack.data_stack.push(LuaValue::Nil);
                            return Ok(1);
                        }
                    } else if pos == len + 1 {
                        stack.data_stack.push(LuaValue::Nil);
                        return Ok(1);
                    }

                    let split_right = table.borrow_mut().arr.split_off(&pos);
                    let first_pos = split_right.first_key_value().map(|(k, _)| *k);
                    if first_pos == Some(pos) {
                        let mut it = split_right.into_iter();
                        let removed = it.next().unwrap().1;
                        stack.data_stack.push(removed);
                        for (idx, val) in it {
                            table.borrow_mut().arr.insert(idx - 1, val);
                        }
                    } else {
                        for (idx, val) in split_right {
                            table.borrow_mut().arr.insert(idx - 1, val);
                        }
                    }
                    Ok(1)
                }
                _ => Err(RuntimeError::NotTable),
            }
        }
    }
}
pub fn sort(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    let (list, _cmp) = stack.pop2(args);

    let list = match list {
        LuaValue::Table(t) => t,
        _ => return Err(RuntimeError::NotTable),
    };

    let mut list_to_vec = Vec::new();
    unpack_impl(
        Rc::clone(&list),
        1,
        list.borrow().len() as IntType,
        &mut list_to_vec,
    )?;

    unimplemented!("table.sort")
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
pub fn unpack(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    let mut it = stack.pop_n(args);
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

    unpack_impl(table, i, j, &mut stack.data_stack)
}
