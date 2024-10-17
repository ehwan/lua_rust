use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use rand::SeedableRng;

use crate::builtin;
use crate::luaval::RefOrValue;
use crate::FunctionInfo;
use crate::IntType;
use crate::LuaFunction;
use crate::LuaFunctionLua;
use crate::LuaTable;
use crate::LuaValue;

use crate::Instruction;
use crate::RuntimeError;

pub struct LuaEnv {
    /// _env
    pub(crate) env: LuaValue,
    /// random number generator
    pub(crate) rng: rand::rngs::StdRng,
}

impl LuaEnv {
    pub fn new() -> LuaEnv {
        let env = Rc::new(RefCell::new(builtin::init_env().unwrap()));
        env.borrow_mut()
            .insert("_G".into(), LuaValue::Table(Rc::clone(&env)));
        LuaEnv {
            env: LuaValue::Table(env),
            rng: rand::rngs::StdRng::from_entropy(),
        }
    }

    /// Try to call binary metamethod f(lhs, rhs).
    /// It tries to search metamethod on lhs first, then rhs.
    fn try_call_metamethod(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
        lhs: LuaValue,
        rhs: LuaValue,
        meta_name: &str,
    ) -> Result<(), RuntimeError> {
        match lhs.get_metavalue(meta_name) {
            Some(meta) => {
                thread.borrow_mut().data_stack.push(lhs);
                thread.borrow_mut().data_stack.push(rhs);
                self.function_call(thread, chunk, 2, meta, Some(1))
            }
            None => match rhs.get_metavalue(meta_name) {
                Some(meta) => {
                    thread.borrow_mut().data_stack.push(lhs);
                    thread.borrow_mut().data_stack.push(rhs);
                    self.function_call(thread, chunk, 2, meta, Some(1))
                }
                None => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// add operation with __add metamethod
    pub fn add(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match (lhs, rhs) {
            // if both are numbers, add them
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                thread.borrow_mut().data_stack.push((lhs + rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(thread, chunk, lhs, rhs, "__add"),
        }
    }

    /// sub operation with __sub metamethod
    pub fn sub(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                thread.borrow_mut().data_stack.push((lhs - rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(thread, chunk, lhs, rhs, "__sub"),
        }
    }
    /// mul operation with __mul metamethod
    pub fn mul(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                thread.borrow_mut().data_stack.push((lhs * rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(thread, chunk, lhs, rhs, "__mul"),
        }
    }
    /// div operation with __div metamethod
    pub fn div(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                thread.borrow_mut().data_stack.push((lhs / rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(thread, chunk, lhs, rhs, "__div"),
        }
    }
    /// mod operation with __mod metamethod
    pub fn mod_(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                thread.borrow_mut().data_stack.push((lhs % rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(thread, chunk, lhs, rhs, "__mod"),
        }
    }
    /// pow operation with __pow metamethod
    pub fn pow(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                thread.borrow_mut().data_stack.push((lhs.pow(rhs)).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(thread, chunk, lhs, rhs, "__pow"),
        }
    }
    /// unary minus operation with __unm metamethod
    pub fn unm(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match lhs {
            LuaValue::Number(num) => {
                thread.borrow_mut().data_stack.push((-num).into());
                Ok(())
            }
            lhs => match lhs.get_metavalue("__unm") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    thread.borrow_mut().data_stack.push(lhs.clone());
                    thread.borrow_mut().data_stack.push(lhs);
                    self.function_call(thread, chunk, 2, meta, Some(1))
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// floor division operation with __idiv metamethod
    pub fn idiv(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                thread
                    .borrow_mut()
                    .data_stack
                    .push((lhs.floor_div(rhs)).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(thread, chunk, lhs, rhs, "__idiv"),
        }
    }
    /// bitwise and operation with __band metamethod
    pub fn band(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        thread.borrow_mut().data_stack.push((lhs & rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(thread, chunk, lhs, rhs, "__band"),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(thread, chunk, lhs, rhs, "__band"),
        }
    }
    /// bitwise or operation with __bor metamethod
    pub fn bor(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        thread.borrow_mut().data_stack.push((lhs | rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(thread, chunk, lhs, rhs, "__bor"),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(thread, chunk, lhs, rhs, "__bor"),
        }
    }
    /// bitwise xor operation with __bxor metamethod
    pub fn bxor(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        thread.borrow_mut().data_stack.push((lhs ^ rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(thread, chunk, lhs, rhs, "__bxor"),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(thread, chunk, lhs, rhs, "__bxor"),
        }
    }
    /// bitwise shift left operation with __shl metamethod
    pub fn shl(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        thread.borrow_mut().data_stack.push((lhs << rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(thread, chunk, lhs, rhs, "__shl"),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(thread, chunk, lhs, rhs, "__shl"),
        }
    }
    /// bitwise shift right operation with __shr metamethod
    pub fn shr(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        thread.borrow_mut().data_stack.push((lhs >> rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(thread, chunk, lhs, rhs, "__shr"),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(thread, chunk, lhs, rhs, "__shr"),
        }
    }
    /// bitwise not operation with __bnot metamethod
    pub fn bnot(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match &lhs {
            LuaValue::Number(lhs_num) => match lhs_num.try_to_int() {
                Some(i) => {
                    thread.borrow_mut().data_stack.push((!i).into());
                    Ok(())
                }
                _ => match lhs.get_metavalue("__bnot") {
                    Some(meta) => {
                        // For the unary operators (negation, length, and bitwise NOT),
                        // the metamethod is computed and called with a dummy second operand
                        // equal to the first one.
                        thread.borrow_mut().data_stack.push(lhs.clone());
                        thread.borrow_mut().data_stack.push(lhs);
                        self.function_call(thread, chunk, 2, meta, Some(1))
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
            _ => match lhs.get_metavalue("__bnot") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    thread.borrow_mut().data_stack.push(lhs.clone());
                    thread.borrow_mut().data_stack.push(lhs);
                    self.function_call(thread, chunk, 2, meta, Some(1))
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// concat operation with __concat metamethod
    pub fn concat(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match lhs {
            LuaValue::Number(lhs_num) => match rhs {
                LuaValue::Number(rhs_num) => {
                    let mut concated = lhs_num.to_string().into_bytes();
                    concated.append(&mut rhs_num.to_string().into_bytes());
                    thread
                        .borrow_mut()
                        .data_stack
                        .push(LuaValue::String(concated));
                    Ok(())
                }
                LuaValue::String(mut rhs) => {
                    let mut lhs = lhs_num.to_string().into_bytes();
                    lhs.append(&mut rhs);
                    thread.borrow_mut().data_stack.push(LuaValue::String(lhs));
                    Ok(())
                }
                _ => self.try_call_metamethod(thread, chunk, lhs, rhs, "__concat"),
            },

            LuaValue::String(lhs_str) => match rhs {
                LuaValue::Number(rhs_num) => {
                    let mut concated = lhs_str;
                    concated.append(&mut rhs_num.to_string().into_bytes());
                    thread
                        .borrow_mut()
                        .data_stack
                        .push(LuaValue::String(concated));
                    Ok(())
                }
                LuaValue::String(mut rhs) => {
                    let mut lhs = lhs_str;
                    lhs.append(&mut rhs);
                    thread.borrow_mut().data_stack.push(LuaValue::String(lhs));
                    Ok(())
                }
                _ => self.try_call_metamethod(
                    thread,
                    chunk,
                    LuaValue::String(lhs_str),
                    rhs,
                    "__concat",
                ),
            },

            _ => self.try_call_metamethod(thread, chunk, lhs, rhs, "__concat"),
        }
    }
    /// `#` length operation with __len metamethod
    pub fn len(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();
        match lhs {
            LuaValue::String(s) => {
                thread
                    .borrow_mut()
                    .data_stack
                    .push((s.len() as IntType).into());
                Ok(())
            }
            LuaValue::Table(table) => {
                let meta = table.borrow().get_metavalue("__len");
                match meta {
                    Some(meta) => {
                        // For the unary operators (negation, length, and bitwise NOT),
                        // the metamethod is computed and called with a dummy second operand
                        // equal to the first one.
                        thread
                            .borrow_mut()
                            .data_stack
                            .push(LuaValue::Table(Rc::clone(&table)));
                        thread.borrow_mut().data_stack.push(LuaValue::Table(table));
                        self.function_call(thread, chunk, 2, meta, Some(1))
                    }
                    _ => {
                        thread
                            .borrow_mut()
                            .data_stack
                            .push((table.borrow().len() as IntType).into());
                        Ok(())
                    }
                }
            }
            lhs => match lhs.get_metavalue("__len") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    thread.borrow_mut().data_stack.push(lhs.clone());
                    thread.borrow_mut().data_stack.push(lhs);
                    self.function_call(thread, chunk, 2, meta, Some(1))
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// table index get operation with __index metamethod
    pub fn index(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let key = thread.borrow_mut().data_stack.pop().unwrap();
        let table = thread.borrow_mut().data_stack.pop().unwrap();
        match table {
            LuaValue::Table(table) => {
                let get = table.borrow().get(&key).cloned();
                if let Some(get) = get {
                    thread.borrow_mut().data_stack.push(get);
                    Ok(())
                } else {
                    let meta = table.borrow().get_metavalue("__index");
                    match meta {
                        Some(LuaValue::Function(meta_func)) => {
                            thread.borrow_mut().data_stack.push(LuaValue::Table(table));
                            thread.borrow_mut().data_stack.push(key);
                            self.function_call(
                                thread,
                                chunk,
                                2,
                                LuaValue::Function(meta_func),
                                Some(1),
                            )
                        }
                        Some(LuaValue::Table(meta_table)) => {
                            thread
                                .borrow_mut()
                                .data_stack
                                .push(LuaValue::Table(meta_table));
                            thread.borrow_mut().data_stack.push(key);
                            self.index(thread, chunk)
                        }
                        _ => {
                            thread.borrow_mut().data_stack.push(LuaValue::Nil);
                            Ok(())
                        }
                    }
                }
            }
            table => {
                let meta = table.get_metavalue("__index");
                match meta {
                    Some(LuaValue::Function(meta_func)) => {
                        thread.borrow_mut().data_stack.push(table);
                        thread.borrow_mut().data_stack.push(key);
                        self.function_call(thread, chunk, 2, LuaValue::Function(meta_func), Some(1))
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        thread
                            .borrow_mut()
                            .data_stack
                            .push(LuaValue::Table(meta_table));
                        thread.borrow_mut().data_stack.push(key);
                        self.index(thread, chunk)
                    }
                    _ => Err(RuntimeError::NotTable),
                }
            }
        }
    }
    /// table index set operation with __newindex metamethod
    pub fn newindex(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let key = thread.borrow_mut().data_stack.pop().unwrap();
        let table = thread.borrow_mut().data_stack.pop().unwrap();
        let value = thread.borrow_mut().data_stack.pop().unwrap();

        match table {
            LuaValue::Table(table) => {
                {
                    let mut table = table.borrow_mut();
                    if let Some(val) = table.get_mut(&key) {
                        // if rhs is nil, remove the key
                        if value.is_nil() {
                            table.remove(&key);
                        } else {
                            *val = value;
                        }
                        return Ok(());
                    }
                }
                let meta = table.borrow().get_metavalue("__newindex");
                match meta {
                    Some(LuaValue::Function(meta_func)) => {
                        thread.borrow_mut().data_stack.push(LuaValue::Table(table));
                        thread.borrow_mut().data_stack.push(key);
                        thread.borrow_mut().data_stack.push(value);
                        self.function_call(thread, chunk, 3, LuaValue::Function(meta_func), Some(0))
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        thread.borrow_mut().data_stack.push(value);
                        thread
                            .borrow_mut()
                            .data_stack
                            .push(LuaValue::Table(meta_table));
                        thread.borrow_mut().data_stack.push(key);
                        self.newindex(thread, chunk)
                    }
                    _ => {
                        table.borrow_mut().insert(key, value);
                        Ok(())
                    }
                }
            }
            table => {
                let meta = table.get_metavalue("__newindex");
                match meta {
                    Some(LuaValue::Function(meta_func)) => {
                        thread.borrow_mut().data_stack.push(table);
                        thread.borrow_mut().data_stack.push(key);
                        thread.borrow_mut().data_stack.push(value);
                        self.function_call(thread, chunk, 3, LuaValue::Function(meta_func), Some(0))
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        thread.borrow_mut().data_stack.push(value);
                        thread
                            .borrow_mut()
                            .data_stack
                            .push(LuaValue::Table(meta_table));
                        thread.borrow_mut().data_stack.push(key);
                        self.newindex(thread, chunk)
                    }
                    _ => Err(RuntimeError::NotTable),
                }
            }
        }
    }
    /// equality operation with __eq metamethod
    pub fn eq(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();

        match (lhs, rhs) {
            (LuaValue::Table(lhs), LuaValue::Table(rhs)) => {
                if Rc::ptr_eq(&lhs, &rhs) {
                    thread.borrow_mut().data_stack.push(LuaValue::Boolean(true));
                    return Ok(());
                } else {
                    self.try_call_metamethod(
                        thread,
                        chunk,
                        LuaValue::Table(lhs),
                        LuaValue::Table(rhs),
                        "__eq",
                    )
                }
            }
            (lhs, rhs) => {
                thread
                    .borrow_mut()
                    .data_stack
                    .push(LuaValue::Boolean(lhs == rhs));
                Ok(())
            }
        }
    }
    /// less than operation with __lt metamethod
    pub fn lt(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();

        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                thread
                    .borrow_mut()
                    .data_stack
                    .push(LuaValue::Boolean(lhs < rhs));
                Ok(())
            }
            (LuaValue::String(lhs), LuaValue::String(rhs)) => {
                thread
                    .borrow_mut()
                    .data_stack
                    .push(LuaValue::Boolean(lhs < rhs));
                Ok(())
            }
            (lhs, rhs) => self.try_call_metamethod(thread, chunk, lhs, rhs, "__lt"),
        }
    }

    /// less than or equal operation with __le metamethod
    pub fn le(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let rhs = thread.borrow_mut().data_stack.pop().unwrap();
        let lhs = thread.borrow_mut().data_stack.pop().unwrap();

        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                thread
                    .borrow_mut()
                    .data_stack
                    .push(LuaValue::Boolean(lhs <= rhs));
                Ok(())
            }
            (LuaValue::String(lhs), LuaValue::String(rhs)) => {
                thread
                    .borrow_mut()
                    .data_stack
                    .push(LuaValue::Boolean(lhs <= rhs));
                Ok(())
            }
            (lhs, rhs) => self.try_call_metamethod(thread, chunk, lhs, rhs, "__le"),
        }
    }

    /// function call with __call metamethod.
    /// this does not return until the function call is finished.
    pub fn function_call(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
        // number of arguments actually passed
        args_num: usize,
        // function object
        func: LuaValue,
        // number of return values expected; returned values will be adjusted to this number
        expected_ret: Option<usize>,
    ) -> Result<(), RuntimeError> {
        match func {
            LuaValue::Function(func) => {
                let func_borrow = func.borrow();
                match &*func_borrow {
                    LuaFunction::LuaFunc(lua_internal) => {
                        let usize_len0 = thread.borrow().usize_stack.len();
                        {
                            let func_id = lua_internal.function_id;
                            let func_info = &chunk.functions[func_id];

                            let thread_mut = &mut *thread.borrow_mut();

                            thread_mut.usize_stack.push(thread_mut.counter);
                            thread_mut.usize_stack.push(thread_mut.bp);
                            thread_mut
                                .usize_stack
                                .push(thread_mut.data_stack.len() - args_num);

                            let variadic = if func_info.is_variadic {
                                if args_num <= func_info.args {
                                    thread_mut.data_stack.resize_with(
                                        thread_mut.data_stack.len() - args_num + func_info.args,
                                        Default::default,
                                    );
                                    Vec::new()
                                } else {
                                    thread_mut
                                        .data_stack
                                        .drain(
                                            thread_mut.data_stack.len() - args_num
                                                + func_info.args..,
                                        )
                                        .collect()
                                }
                            } else {
                                thread_mut.data_stack.resize_with(
                                    thread_mut.data_stack.len() - args_num + func_info.args,
                                    Default::default,
                                );
                                Vec::new()
                            };

                            thread_mut.bp = thread_mut.local_variables.len();
                            thread_mut.local_variables.reserve(func_info.stack_size);

                            {
                                let mut args: Vec<_> = thread_mut
                                    .data_stack
                                    .drain(thread_mut.data_stack.len() - func_info.args..)
                                    .map(|arg| RefOrValue::Value(arg))
                                    .collect();
                                thread_mut.local_variables.append(&mut args);
                            }

                            let func_stack = FunctionStackElem {
                                function_object: lua_internal.clone(),
                                return_expected: expected_ret,
                                variadic,
                            };

                            thread_mut.function_stack.push(func_stack);
                            thread_mut.counter = func_info.address;
                        }
                        drop(func_borrow);
                        loop {
                            if thread.borrow().usize_stack.len() == usize_len0 {
                                break;
                            }
                            let instruction =
                                chunk.instructions.get(thread.borrow().counter).unwrap();
                            self.cycle(thread, chunk, instruction)?;
                        }
                        Ok(())
                    }
                    LuaFunction::RustFunc(rust_internal) => {
                        let ret_num = rust_internal(self, thread, chunk, args_num)?;
                        if let Some(expected) = expected_ret {
                            let adjusted = thread.borrow().data_stack.len() - ret_num + expected;
                            thread
                                .borrow_mut()
                                .data_stack
                                .resize_with(adjusted, Default::default);
                        }
                        Ok(())
                    }
                }
            }
            other => {
                let func = other.get_metavalue("__call");
                if let Some(meta) = func {
                    {
                        let front_arg_pos = thread.borrow().data_stack.len() - args_num;
                        thread.borrow_mut().data_stack.insert(front_arg_pos, other);
                    }
                    self.function_call(thread, chunk, args_num + 1, meta, expected_ret)
                } else {
                    Err(RuntimeError::NotFunction)
                }
            }
        }
    }
    /// execute single instruction
    pub fn cycle(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
        instruction: &Instruction,
    ) -> Result<(), RuntimeError> {
        match instruction {
            Instruction::Clone => {
                let top = thread.borrow().data_stack.last().unwrap().clone();
                thread.borrow_mut().data_stack.push(top);
            }
            Instruction::Sp => {
                let len = thread.borrow().data_stack.len();
                thread.borrow_mut().usize_stack.push(len);
            }
            Instruction::Pop => {
                thread.borrow_mut().data_stack.pop();
            }
            Instruction::Deref => {
                let sp = *thread.borrow().usize_stack.last().unwrap();
                let top = thread.borrow().data_stack[sp].clone();
                thread.borrow_mut().data_stack.push(top);
            }
            Instruction::Jump(label) => {
                let pc = *chunk.label_map.get(*label).unwrap();
                thread.borrow_mut().counter = pc;
                return Ok(());
            }
            Instruction::JumpTrue(label) => {
                if thread.borrow_mut().data_stack.pop().unwrap().to_bool() {
                    let pc = *chunk.label_map.get(*label).unwrap();
                    thread.borrow_mut().counter = pc;
                    return Ok(());
                }
            }
            Instruction::JumpFalse(label) => {
                if !thread.borrow_mut().data_stack.pop().unwrap().to_bool() {
                    let pc = *chunk.label_map.get(*label).unwrap();
                    thread.borrow_mut().counter = pc;
                    return Ok(());
                }
            }
            Instruction::GetLocalVariable(local_id) => {
                let val = {
                    let thread = thread.borrow();
                    let local_idx = *local_id + thread.bp;
                    match thread.local_variables.get(local_idx).unwrap() {
                        RefOrValue::Ref(val) => val.borrow().clone(),
                        RefOrValue::Value(val) => val.clone(),
                    }
                };
                thread.borrow_mut().data_stack.push(val);
            }
            Instruction::SetLocalVariable(local_id) => {
                let top = thread.borrow_mut().data_stack.pop().unwrap();
                let local_idx = *local_id + thread.borrow().bp;
                match thread
                    .borrow_mut()
                    .local_variables
                    .get_mut(local_idx)
                    .unwrap()
                {
                    RefOrValue::Ref(val) => {
                        val.replace(top);
                    }
                    RefOrValue::Value(val) => {
                        *val = top;
                    }
                }
            }
            Instruction::InitLocalVariable(local_id) => {
                let top = thread.borrow_mut().data_stack.pop().unwrap();
                let idx = *local_id + thread.borrow().bp;
                {
                    let len = thread.borrow().local_variables.len();
                    if len <= idx {
                        thread
                            .borrow_mut()
                            .local_variables
                            .resize_with(idx + 1, Default::default);
                    }
                }
                *thread.borrow_mut().local_variables.get_mut(idx).unwrap() = RefOrValue::Value(top);
            }
            Instruction::IsNil => {
                let top = thread.borrow_mut().data_stack.pop().unwrap();
                thread
                    .borrow_mut()
                    .data_stack
                    .push(LuaValue::Boolean(top.is_nil()));
            }

            Instruction::Nil => {
                thread.borrow_mut().data_stack.push(LuaValue::Nil);
            }
            Instruction::Boolean(b) => {
                thread.borrow_mut().data_stack.push(LuaValue::Boolean(*b));
            }
            Instruction::Numeric(n) => thread.borrow_mut().data_stack.push(LuaValue::Number(*n)),
            Instruction::String(s) => {
                thread
                    .borrow_mut()
                    .data_stack
                    .push(LuaValue::String(s.clone()));
            }
            Instruction::GetEnv => {
                thread.borrow_mut().data_stack.push(self.env.clone());
            }
            Instruction::TableInit(cap) => {
                let table = LuaTable::with_capacity(*cap);
                thread
                    .borrow_mut()
                    .data_stack
                    .push(LuaValue::Table(Rc::new(RefCell::new(table))));
            }
            Instruction::TableIndexInit => {
                let value = thread.borrow_mut().data_stack.pop().unwrap();
                let index = thread.borrow_mut().data_stack.pop().unwrap();
                if let LuaValue::Table(table) = thread.borrow().data_stack.last().unwrap() {
                    table.borrow_mut().insert(index, value);
                } else {
                    unreachable!("table must be on top of stack");
                }
            }
            Instruction::TableInitLast(i) => {
                let sp = thread.borrow_mut().usize_stack.pop().unwrap();
                let mut values: BTreeMap<_, _> = thread
                    .borrow_mut()
                    .data_stack
                    .drain(sp..)
                    .enumerate()
                    .map(|(idx, value)| {
                        let index = idx as IntType + *i;
                        (index.into(), value)
                    })
                    .collect();

                if let LuaValue::Table(table) = thread.borrow().data_stack.last().unwrap() {
                    table.borrow_mut().arr.append(&mut values);
                } else {
                    unreachable!("table must be on top of stack");
                }
            }

            Instruction::TableIndex => {
                self.index(thread, chunk)?;
            }
            Instruction::TableIndexSet => {
                self.newindex(thread, chunk)?;
            }

            Instruction::FunctionInit(func_id, num_upvalues) => {
                let func = LuaFunctionLua {
                    function_id: *func_id,
                    upvalues: Vec::with_capacity(*num_upvalues),
                };
                let func = LuaFunction::LuaFunc(func);
                thread.borrow_mut().data_stack.push(func.into());
            }
            Instruction::FunctionInitUpvalueFromLocalVar(src_local_id) => {
                let local_var = {
                    let local_idx = *src_local_id + thread.borrow().bp;
                    let mut thread = thread.borrow_mut();
                    let local_var = thread.local_variables.get_mut(local_idx).unwrap();
                    // upvalue must be reference.
                    match local_var {
                        RefOrValue::Ref(r) => Rc::clone(r),
                        RefOrValue::Value(v) => {
                            let reffed_var = Rc::new(RefCell::new(v.clone()));
                            *local_var = RefOrValue::Ref(Rc::clone(&reffed_var));
                            reffed_var
                        }
                    }
                };

                match thread.borrow().data_stack.last().unwrap() {
                    LuaValue::Function(func) => match &mut *func.borrow_mut() {
                        LuaFunction::LuaFunc(f) => {
                            f.upvalues.push(local_var);
                        }
                        _ => unreachable!("stack top must be function"),
                    },
                    _ => unreachable!("stack top must be function"),
                }
            }
            Instruction::FunctionInitUpvalueFromUpvalue(src_upvalue_id) => {
                let value = {
                    Rc::clone(
                        &thread
                            .borrow()
                            .function_stack
                            .last()
                            .unwrap()
                            .function_object
                            .upvalues[*src_upvalue_id],
                    )
                };

                match thread.borrow().data_stack.last().unwrap() {
                    LuaValue::Function(func) => match &mut *func.borrow_mut() {
                        LuaFunction::LuaFunc(f) => {
                            f.upvalues.push(value);
                        }
                        _ => unreachable!("stack top must be function"),
                    },
                    _ => unreachable!("stack top must be function"),
                }
            }

            Instruction::FunctionUpvalue(upvalue_id) => {
                let value = {
                    RefCell::borrow(
                        &thread
                            .borrow()
                            .function_stack
                            .last()
                            .unwrap()
                            .function_object
                            .upvalues[*upvalue_id],
                    )
                    .clone()
                };
                thread.borrow_mut().data_stack.push(value);
            }
            Instruction::FunctionUpvalueSet(upvalue_id) => {
                let rhs = thread.borrow_mut().data_stack.pop().unwrap();
                *thread
                    .borrow()
                    .function_stack
                    .last()
                    .unwrap()
                    .function_object
                    .upvalues[*upvalue_id]
                    .borrow_mut() = rhs;
            }

            Instruction::BinaryAdd => {
                self.add(thread, chunk)?;
            }
            Instruction::BinarySub => {
                self.sub(thread, chunk)?;
            }
            Instruction::BinaryMul => {
                self.mul(thread, chunk)?;
            }
            Instruction::BinaryDiv => {
                self.div(thread, chunk)?;
            }
            Instruction::BinaryFloorDiv => {
                self.idiv(thread, chunk)?;
            }
            Instruction::BinaryMod => {
                self.mod_(thread, chunk)?;
            }
            Instruction::BinaryPow => {
                self.pow(thread, chunk)?;
            }
            Instruction::BinaryConcat => {
                self.concat(thread, chunk)?;
            }
            Instruction::BinaryBitwiseAnd => {
                self.band(thread, chunk)?;
            }
            Instruction::BinaryBitwiseOr => {
                self.bor(thread, chunk)?;
            }
            Instruction::BinaryBitwiseXor => {
                self.bxor(thread, chunk)?;
            }
            Instruction::BinaryShiftLeft => {
                self.shl(thread, chunk)?;
            }
            Instruction::BinaryShiftRight => {
                self.shr(thread, chunk)?;
            }
            Instruction::BinaryEqual => {
                self.eq(thread, chunk)?;
            }
            Instruction::BinaryLessThan => {
                self.lt(thread, chunk)?;
            }
            Instruction::BinaryLessEqual => {
                self.le(thread, chunk)?;
            }

            Instruction::UnaryMinus => {
                self.unm(thread, chunk)?;
            }
            Instruction::UnaryBitwiseNot => {
                self.bnot(thread, chunk)?;
            }
            Instruction::UnaryLength => {
                self.len(thread, chunk)?;
            }
            Instruction::UnaryLogicalNot => {
                let top = thread.borrow_mut().data_stack.pop().unwrap().to_bool();
                thread.borrow_mut().data_stack.push((!top).into());
            }

            Instruction::FunctionCall(expected_ret) => {
                let (func, num_args) = {
                    let func = thread.borrow_mut().data_stack.pop().unwrap();
                    let sp = thread.borrow_mut().usize_stack.pop().unwrap();
                    let num_args = thread.borrow().data_stack.len() - sp;
                    (func, num_args)
                };
                self.function_call(thread, chunk, num_args, func, *expected_ret)?;
            }

            // sp -> top
            Instruction::Return => {
                let mut thread = thread.borrow_mut();
                if let Some(func) = thread.function_stack.pop() {
                    // return from function call
                    let old_stacklen = thread.usize_stack.pop().unwrap();
                    let old_bp = thread.usize_stack.pop().unwrap();
                    let old_pc = thread.usize_stack.pop().unwrap();
                    let old_local_len = thread.bp;
                    thread.local_variables.truncate(old_local_len);
                    thread.bp = old_bp;
                    thread.counter = old_pc;

                    if let Some(expected) = func.return_expected {
                        let adjusted = old_stacklen + expected;
                        thread.data_stack.resize_with(adjusted, Default::default);
                    }
                    return Ok(());
                } else {
                    // main chunk
                    thread.counter = chunk.instructions.len();
                    return Ok(());
                }
            }

            Instruction::GetVariadic(expected) => {
                if let Some(expected) = expected {
                    let mut variadic = thread
                        .borrow()
                        .function_stack
                        .last()
                        .unwrap()
                        .variadic
                        .clone();
                    variadic.resize_with(*expected, Default::default);
                    thread.borrow_mut().data_stack.append(&mut variadic);
                } else {
                    let mut variadic = thread
                        .borrow()
                        .function_stack
                        .last()
                        .unwrap()
                        .variadic
                        .clone();
                    thread.borrow_mut().data_stack.append(&mut variadic);
                }
            }
        }
        thread.borrow_mut().counter += 1;
        Ok(())
    }
    /// run the whole chunk
    pub fn run(
        &mut self,
        thread: &Rc<RefCell<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        loop {
            let counter = thread.borrow().counter;
            if let Some(instruction) = chunk.instructions.get(counter) {
                self.cycle(thread, chunk, instruction)?;
            } else {
                break;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FunctionStackElem {
    /// function object
    pub function_object: LuaFunctionLua,
    /// number of return values expected.
    pub return_expected: Option<usize>,
    /// variadic arguments
    pub variadic: Vec<LuaValue>,
}

#[derive(Debug, Clone)]
pub struct LuaThread {
    /// local variable stack
    pub local_variables: Vec<RefOrValue>,
    /// offset of local variables for current scope
    pub bp: usize,

    /// normal stack, for temporary values
    pub data_stack: Vec<LuaValue>,

    /// stack for storing usize values
    pub usize_stack: Vec<usize>,

    // function object, variadic, return values multire expected count
    pub function_stack: Vec<FunctionStackElem>,

    /// current instruction counter
    pub counter: usize,
}
impl LuaThread {
    pub fn new() -> LuaThread {
        LuaThread {
            local_variables: Vec::new(),
            data_stack: Vec::new(),
            usize_stack: Vec::new(),
            function_stack: Vec::new(),
            counter: 0,
            bp: 0,
        }
    }

    pub fn top(&self) -> &LuaValue {
        self.data_stack.last().unwrap()
    }
    pub fn top_mut(&mut self) -> &mut LuaValue {
        self.data_stack.last_mut().unwrap()
    }
    pub fn pop(&mut self) -> Option<LuaValue> {
        self.data_stack.pop()
    }
    pub fn pop_n(&mut self, n: usize) -> impl Iterator<Item = LuaValue> + '_ {
        self.data_stack.drain(self.data_stack.len() - n..)
    }
    pub fn pop1(&mut self, n: usize) -> LuaValue {
        let mut it = self.pop_n(n);
        let v0 = it.next().unwrap();
        v0
    }
    pub fn pop2(&mut self, n: usize) -> (LuaValue, LuaValue) {
        let mut it = self.pop_n(n);
        let v0 = it.next().unwrap();
        let v1 = it.next().unwrap();
        (v0, v1)
    }
    pub fn pop3(&mut self, n: usize) -> (LuaValue, LuaValue, LuaValue) {
        let mut it = self.pop_n(n);
        let v0 = it.next().unwrap();
        let v1 = it.next().unwrap();
        let v2 = it.next().unwrap();
        (v0, v1, v2)
    }
    pub fn pop4(&mut self, n: usize) -> (LuaValue, LuaValue, LuaValue, LuaValue) {
        let mut it = self.pop_n(n);
        let v0 = it.next().unwrap();
        let v1 = it.next().unwrap();
        let v2 = it.next().unwrap();
        let v3 = it.next().unwrap();
        (v0, v1, v2, v3)
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub label_map: Vec<usize>,
    pub functions: Vec<FunctionInfo>,
    pub stack_size: usize,
}
impl Chunk {}
