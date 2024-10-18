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

    pub(crate) main_thread: Rc<RefCell<LuaThread>>,
    pub(crate) running_thread: Rc<RefCell<LuaThread>>,
}

impl LuaEnv {
    pub fn new() -> LuaEnv {
        let env = Rc::new(RefCell::new(builtin::init_env().unwrap()));
        env.borrow_mut()
            .insert("_G".into(), LuaValue::Table(Rc::clone(&env)));
        let main_thread = Rc::new(RefCell::new(LuaThread::new()));
        LuaEnv {
            env: LuaValue::Table(env),
            rng: rand::rngs::StdRng::from_entropy(),

            main_thread: Rc::clone(&main_thread),
            running_thread: main_thread,
        }
    }

    /// clone i'th value from top of the stack and push it.
    pub fn clone_stack_relative(&self, i_from_top: usize) {
        let mut thread = self.running_thread.borrow_mut();
        let idx = thread.data_stack.len() - i_from_top - 1;
        let top = thread.data_stack[idx].clone();
        thread.data_stack.push(top);
    }

    pub fn main_thread(&self) -> &Rc<RefCell<LuaThread>> {
        &self.main_thread
    }

    pub fn push(&self, value: LuaValue) {
        self.running_thread.borrow_mut().data_stack.push(value);
    }
    pub fn push2(&self, value1: LuaValue, value2: LuaValue) {
        let mut thread = self.running_thread.borrow_mut();
        thread.data_stack.push(value1);
        thread.data_stack.push(value2);
    }
    pub fn push3(&self, value1: LuaValue, value2: LuaValue, value3: LuaValue) {
        let mut thread = self.running_thread.borrow_mut();
        thread.data_stack.push(value1);
        thread.data_stack.push(value2);
        thread.data_stack.push(value3);
    }
    pub fn push4(&self, value1: LuaValue, value2: LuaValue, value3: LuaValue, value4: LuaValue) {
        let mut thread = self.running_thread.borrow_mut();
        thread.data_stack.push(value1);
        thread.data_stack.push(value2);
        thread.data_stack.push(value3);
        thread.data_stack.push(value4);
    }
    pub fn pop(&self) -> LuaValue {
        self.running_thread.borrow_mut().data_stack.pop().unwrap()
    }
    pub fn pop2(&self) -> (LuaValue, LuaValue) {
        let mut thread = self.running_thread.borrow_mut();
        let value2 = thread.data_stack.pop().unwrap();
        let value1 = thread.data_stack.pop().unwrap();
        (value1, value2)
    }
    pub fn pop3(&self) -> (LuaValue, LuaValue, LuaValue) {
        let mut thread = self.running_thread.borrow_mut();
        let value3 = thread.data_stack.pop().unwrap();
        let value2 = thread.data_stack.pop().unwrap();
        let value1 = thread.data_stack.pop().unwrap();
        (value1, value2, value3)
    }
    pub fn pop_n(&self, n: usize) {
        let len = self.running_thread.borrow().data_stack.len();
        self.running_thread
            .borrow_mut()
            .data_stack
            .truncate(len - n);
    }
    pub fn borrow_running_thread(&self) -> std::cell::Ref<LuaThread> {
        self.running_thread.borrow()
    }
    pub fn borrow_running_thread_mut(&self) -> std::cell::RefMut<LuaThread> {
        self.running_thread.borrow_mut()
    }
    pub fn fill_nil(&self, n: usize) {
        let mut thread = self.running_thread.borrow_mut();
        thread
            .data_stack
            .extend(std::iter::repeat(LuaValue::Nil).take(n));
    }

    /// Try to call binary metamethod f(lhs, rhs).
    /// It tries to search metamethod on lhs first, then rhs.
    fn try_call_metamethod(
        &mut self,
        chunk: &Chunk,
        lhs: LuaValue,
        rhs: LuaValue,
        meta_name: &str,
    ) -> Result<(), RuntimeError> {
        match lhs.get_metavalue(meta_name) {
            Some(meta) => {
                self.push2(lhs, rhs);
                self.function_call(chunk, 2, meta, Some(1))
            }
            None => match rhs.get_metavalue(meta_name) {
                Some(meta) => {
                    self.push2(lhs, rhs);
                    self.function_call(chunk, 2, meta, Some(1))
                }
                None => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// string-fy a value
    pub fn tostring(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let top = self.pop();
        let meta = top.get_metavalue("__tostring");
        match meta {
            Some(meta) => {
                self.push(top);
                self.function_call(chunk, 1, meta, Some(1))?;
                self.tostring(chunk)
            }
            _ => {
                let name = top.get_metavalue("__name");
                let s = match name {
                    Some(name) => match name {
                        LuaValue::String(name) => LuaValue::String(name),
                        _ => LuaValue::String(name.to_string().into_bytes()),
                    },
                    None => match top {
                        LuaValue::String(s) => LuaValue::String(s),
                        top => LuaValue::String(top.to_string().into_bytes()),
                    },
                };
                self.push(s);
                Ok(())
            }
        }
    }

    /// add operation with __add metamethod
    pub fn add(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            // if both are numbers, add them
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs + rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__add"),
        }
    }

    /// sub operation with __sub metamethod
    pub fn sub(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs - rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__sub"),
        }
    }
    /// mul operation with __mul metamethod
    pub fn mul(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs * rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__mul"),
        }
    }
    /// div operation with __div metamethod
    pub fn div(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs / rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__div"),
        }
    }
    /// mod operation with __mod metamethod
    pub fn mod_(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs % rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__mod"),
        }
    }
    /// pow operation with __pow metamethod
    pub fn pow(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs.pow(rhs)).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__pow"),
        }
    }
    /// unary minus operation with __unm metamethod
    pub fn unm(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let lhs = self.pop();
        match lhs {
            LuaValue::Number(num) => {
                self.push((-num).into());
                Ok(())
            }
            lhs => match lhs.get_metavalue("__unm") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    self.push2(lhs.clone(), lhs);
                    self.function_call(chunk, 2, meta, Some(1))
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// floor division operation with __idiv metamethod
    pub fn idiv(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs.floor_div(rhs)).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__idiv"),
        }
    }
    /// bitwise and operation with __band metamethod
    pub fn band(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        self.push((lhs & rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(chunk, lhs, rhs, "__band"),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(chunk, lhs, rhs, "__band"),
        }
    }
    /// bitwise or operation with __bor metamethod
    pub fn bor(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        self.push((lhs | rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(chunk, lhs, rhs, "__bor"),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(chunk, lhs, rhs, "__bor"),
        }
    }
    /// bitwise xor operation with __bxor metamethod
    pub fn bxor(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        self.push((lhs ^ rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(chunk, lhs, rhs, "__bxor"),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(chunk, lhs, rhs, "__bxor"),
        }
    }
    /// bitwise shift left operation with __shl metamethod
    pub fn shl(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        self.push((lhs << rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(chunk, lhs, rhs, "__shl"),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(chunk, lhs, rhs, "__shl"),
        }
    }
    /// bitwise shift right operation with __shr metamethod
    pub fn shr(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        self.push((lhs >> rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(chunk, lhs, rhs, "__shr"),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(chunk, lhs, rhs, "__shr"),
        }
    }
    /// bitwise not operation with __bnot metamethod
    pub fn bnot(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let lhs = self.pop();
        match &lhs {
            LuaValue::Number(lhs_num) => match lhs_num.try_to_int() {
                Some(i) => {
                    self.push((!i).into());
                    Ok(())
                }
                _ => match lhs.get_metavalue("__bnot") {
                    Some(meta) => {
                        // For the unary operators (negation, length, and bitwise NOT),
                        // the metamethod is computed and called with a dummy second operand
                        // equal to the first one.
                        self.push2(lhs.clone(), lhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
            _ => match lhs.get_metavalue("__bnot") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    self.push2(lhs.clone(), lhs);
                    self.function_call(chunk, 2, meta, Some(1))
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// concat operation with __concat metamethod
    pub fn concat(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match lhs {
            LuaValue::Number(lhs_num) => match rhs {
                LuaValue::Number(rhs_num) => {
                    let mut concated = lhs_num.to_string().into_bytes();
                    concated.append(&mut rhs_num.to_string().into_bytes());
                    self.push(LuaValue::String(concated));
                    Ok(())
                }
                LuaValue::String(mut rhs) => {
                    let mut lhs = lhs_num.to_string().into_bytes();
                    lhs.append(&mut rhs);
                    self.push(LuaValue::String(lhs));
                    Ok(())
                }
                _ => self.try_call_metamethod(chunk, lhs, rhs, "__concat"),
            },

            LuaValue::String(lhs_str) => match rhs {
                LuaValue::Number(rhs_num) => {
                    let mut concated = lhs_str;
                    concated.append(&mut rhs_num.to_string().into_bytes());
                    self.push(LuaValue::String(concated));
                    Ok(())
                }
                LuaValue::String(mut rhs) => {
                    let mut lhs = lhs_str;
                    lhs.append(&mut rhs);
                    self.push(LuaValue::String(lhs));
                    Ok(())
                }
                _ => self.try_call_metamethod(chunk, LuaValue::String(lhs_str), rhs, "__concat"),
            },

            _ => self.try_call_metamethod(chunk, lhs, rhs, "__concat"),
        }
    }
    /// `#` length operation with __len metamethod
    pub fn len(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let lhs = self.pop();
        match lhs {
            LuaValue::String(s) => {
                self.push((s.len() as IntType).into());
                Ok(())
            }
            LuaValue::Table(table) => {
                let meta = table.borrow().get_metavalue("__len");
                match meta {
                    Some(meta) => {
                        // For the unary operators (negation, length, and bitwise NOT),
                        // the metamethod is computed and called with a dummy second operand
                        // equal to the first one.
                        self.push2(LuaValue::Table(Rc::clone(&table)), LuaValue::Table(table));
                        self.function_call(chunk, 2, meta, Some(1))
                    }
                    _ => {
                        self.push((table.borrow().len() as IntType).into());
                        Ok(())
                    }
                }
            }
            lhs => match lhs.get_metavalue("__len") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    self.push2(lhs.clone(), lhs);
                    self.function_call(chunk, 2, meta, Some(1))
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// table index get operation with __index metamethod
    pub fn index(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (table, key) = self.pop2();
        match table {
            LuaValue::Table(table) => {
                let get = table.borrow().get(&key).cloned();
                if let Some(get) = get {
                    self.push(get);
                    Ok(())
                } else {
                    let meta = table.borrow().get_metavalue("__index");
                    match meta {
                        Some(LuaValue::Function(meta_func)) => {
                            self.push2(LuaValue::Table(table), key);
                            self.function_call(chunk, 2, LuaValue::Function(meta_func), Some(1))
                        }
                        Some(LuaValue::Table(meta_table)) => {
                            self.push2(LuaValue::Table(meta_table), key);
                            self.index(chunk)
                        }
                        _ => {
                            self.push(LuaValue::Nil);
                            Ok(())
                        }
                    }
                }
            }
            table => {
                let meta = table.get_metavalue("__index");
                match meta {
                    Some(LuaValue::Function(meta_func)) => {
                        self.push2(table, key);
                        self.function_call(chunk, 2, LuaValue::Function(meta_func), Some(1))
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        self.push2(LuaValue::Table(meta_table), key);
                        self.index(chunk)
                    }
                    _ => Err(RuntimeError::NotTable),
                }
            }
        }
    }
    /// table index set operation with __newindex metamethod
    pub fn newindex(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (value, table, key) = self.pop3();

        match table {
            LuaValue::Table(table) => {
                {
                    let mut table_mut = table.borrow_mut();
                    if let Some(val) = table_mut.get_mut(&key) {
                        // if rhs is nil, remove the key
                        if value.is_nil() {
                            table_mut.remove(&key);
                        } else {
                            *val = value;
                        }
                        return Ok(());
                    }
                }
                let meta = table.borrow().get_metavalue("__newindex");
                match meta {
                    Some(LuaValue::Function(meta_func)) => {
                        self.push3(LuaValue::Table(table), key, value);
                        self.function_call(chunk, 3, LuaValue::Function(meta_func), Some(0))
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        self.push3(value, LuaValue::Table(meta_table), key);
                        self.newindex(chunk)
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
                        self.push3(table, key, value);
                        self.function_call(chunk, 3, LuaValue::Function(meta_func), Some(0))
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        self.push3(value, LuaValue::Table(meta_table), key);
                        self.newindex(chunk)
                    }
                    _ => Err(RuntimeError::NotTable),
                }
            }
        }
    }
    /// equality operation with __eq metamethod
    pub fn eq(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();

        match (lhs, rhs) {
            (LuaValue::Table(lhs), LuaValue::Table(rhs)) => {
                if Rc::ptr_eq(&lhs, &rhs) {
                    self.push(LuaValue::Boolean(true));
                    return Ok(());
                } else {
                    self.try_call_metamethod(
                        chunk,
                        LuaValue::Table(lhs),
                        LuaValue::Table(rhs),
                        "__eq",
                    )
                }
            }
            (lhs, rhs) => {
                self.push(LuaValue::Boolean(lhs == rhs));
                Ok(())
            }
        }
    }
    /// less than operation with __lt metamethod
    pub fn lt(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();

        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push(LuaValue::Boolean(lhs < rhs));
                Ok(())
            }
            (LuaValue::String(lhs), LuaValue::String(rhs)) => {
                self.push(LuaValue::Boolean(lhs < rhs));
                Ok(())
            }
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__lt"),
        }
    }

    /// less than or equal operation with __le metamethod
    pub fn le(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();

        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push(LuaValue::Boolean(lhs <= rhs));
                Ok(())
            }
            (LuaValue::String(lhs), LuaValue::String(rhs)) => {
                self.push(LuaValue::Boolean(lhs <= rhs));
                Ok(())
            }
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__le"),
        }
    }

    /// function call with __call metamethod.
    /// this does not return until the function call is finished.
    pub fn function_call(
        &mut self,
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
                        let usize_len0 = self.running_thread.borrow().usize_stack.len();

                        let mut thread_mut_ = self.running_thread.borrow_mut();
                        let thread_mut = &mut *thread_mut_;

                        let func_info = &chunk.functions[lua_internal.function_id];

                        // adjust function arguments
                        // extract variadic arguments if needed
                        // push function call stack
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
                                        thread_mut.data_stack.len() - args_num + func_info.args..,
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
                        thread_mut.function_stack.push(FunctionStackElem {
                            function_object: lua_internal.clone(),
                            return_expected: expected_ret,
                            variadic,
                        });
                        drop(func_borrow);

                        // push stack frame
                        //  - current instruction
                        thread_mut.usize_stack.push(thread_mut.counter);
                        //  - current base pointer
                        thread_mut.usize_stack.push(thread_mut.bp);
                        //  - current stack size
                        thread_mut
                            .usize_stack
                            .push(thread_mut.data_stack.len() - args_num);

                        // set base pointer to new stack frame
                        thread_mut.bp = thread_mut.local_variables.len();
                        // reserve stack space for local variables
                        thread_mut.local_variables.reserve(func_info.stack_size);

                        // copy arguments to local variables
                        thread_mut.local_variables.extend(
                            thread_mut
                                .data_stack
                                .drain(thread_mut.data_stack.len() - func_info.args..)
                                .map(|arg| RefOrValue::Value(arg)),
                        );

                        // move program counter to function start
                        thread_mut.counter = func_info.address;
                        drop(thread_mut_);

                        loop {
                            if self.running_thread.borrow().usize_stack.len() == usize_len0 {
                                break;
                            }
                            let instruction = chunk
                                .instructions
                                .get(self.running_thread.borrow().counter)
                                .unwrap();
                            self.cycle(chunk, instruction)?;
                        }
                        Ok(())
                    }
                    LuaFunction::RustFunc(rust_internal) => {
                        rust_internal(self, chunk, args_num, expected_ret)
                    }
                }
            }
            other => {
                let func = other.get_metavalue("__call");
                if let Some(meta) = func {
                    {
                        // push `self` as first argument
                        let front_arg_pos =
                            self.running_thread.borrow().data_stack.len() - args_num;
                        self.running_thread
                            .borrow_mut()
                            .data_stack
                            .insert(front_arg_pos, other);
                    }
                    self.function_call(chunk, args_num + 1, meta, expected_ret)
                } else {
                    Err(RuntimeError::NotFunction)
                }
            }
        }
    }
    /// execute single instruction
    pub fn cycle(&mut self, chunk: &Chunk, instruction: &Instruction) -> Result<(), RuntimeError> {
        match instruction {
            Instruction::Clone => {
                let mut thread_mut = self.running_thread.borrow_mut();
                let top = thread_mut.data_stack.last().unwrap().clone();
                thread_mut.data_stack.push(top);
            }
            Instruction::Sp => {
                let mut thread_mut = self.running_thread.borrow_mut();
                let len = thread_mut.data_stack.len();
                thread_mut.usize_stack.push(len);
            }
            Instruction::Pop => {
                self.pop();
            }
            Instruction::Deref => {
                let mut thread_mut = self.running_thread.borrow_mut();
                let sp = thread_mut.usize_stack.pop().unwrap();
                let top = thread_mut.data_stack[sp].clone();
                thread_mut.data_stack.push(top);
            }
            Instruction::Jump(label) => {
                let pc = *chunk.label_map.get(*label).unwrap();
                self.running_thread.borrow_mut().counter = pc;
                return Ok(());
            }
            Instruction::JumpTrue(label) => {
                let mut thread_mut = self.running_thread.borrow_mut();
                let top = thread_mut.data_stack.pop().unwrap().to_bool();
                if top {
                    let pc = *chunk.label_map.get(*label).unwrap();
                    thread_mut.counter = pc;
                    return Ok(());
                }
            }
            Instruction::JumpFalse(label) => {
                let mut thread_mut = self.running_thread.borrow_mut();
                let top = thread_mut.data_stack.pop().unwrap().to_bool();
                if !top {
                    let pc = *chunk.label_map.get(*label).unwrap();
                    thread_mut.counter = pc;
                    return Ok(());
                }
            }
            Instruction::GetLocalVariable(local_id) => {
                let mut thread_mut = self.running_thread.borrow_mut();
                let local_idx = *local_id + thread_mut.bp;
                let val = match thread_mut.local_variables.get(local_idx).unwrap() {
                    RefOrValue::Ref(val) => val.borrow().clone(),
                    RefOrValue::Value(val) => val.clone(),
                };
                thread_mut.data_stack.push(val);
            }
            Instruction::SetLocalVariable(local_id) => {
                let mut thread_mut = self.running_thread.borrow_mut();
                let top = thread_mut.data_stack.pop().unwrap();
                let local_idx = *local_id + thread_mut.bp;
                match thread_mut.local_variables.get_mut(local_idx).unwrap() {
                    RefOrValue::Ref(val) => {
                        val.replace(top);
                    }
                    RefOrValue::Value(val) => {
                        *val = top;
                    }
                }
            }
            Instruction::InitLocalVariable(local_id) => {
                let mut thread_mut = self.running_thread.borrow_mut();
                let top = thread_mut.data_stack.pop().unwrap();
                let local_idx = *local_id + thread_mut.bp;
                {
                    let len = thread_mut.local_variables.len();
                    if len <= local_idx {
                        thread_mut
                            .local_variables
                            .resize_with(local_idx + 1, Default::default);
                    }
                }
                *thread_mut.local_variables.get_mut(local_idx).unwrap() = RefOrValue::Value(top);
            }
            Instruction::IsNil => {
                let mut thread_mut = self.running_thread.borrow_mut();
                let top = thread_mut.data_stack.pop().unwrap();
                thread_mut.data_stack.push(LuaValue::Boolean(top.is_nil()));
            }

            Instruction::Nil => {
                self.push(LuaValue::Nil);
            }
            Instruction::Boolean(b) => {
                self.push(LuaValue::Boolean(*b));
            }
            Instruction::Numeric(n) => {
                self.push(LuaValue::Number(*n));
            }
            Instruction::String(s) => {
                self.push(LuaValue::String(s.clone()));
            }
            Instruction::GetEnv => {
                let env = self.env.clone();
                self.push(env);
            }
            Instruction::TableInit(cap) => {
                let table = LuaTable::with_capacity(*cap);
                self.push(table.into());
            }
            Instruction::TableIndexInit => {
                let (table, index, value) = self.pop3();
                if let LuaValue::Table(table) = table {
                    table.borrow_mut().insert(index, value);
                    self.push(LuaValue::Table(table));
                } else {
                    unreachable!("table must be on top of stack");
                }
            }
            Instruction::TableInitLast(start_key) => {
                let mut thread_mut = self.running_thread.borrow_mut();
                let sp = thread_mut.usize_stack.pop().unwrap();
                let mut values: BTreeMap<_, _> = thread_mut
                    .data_stack
                    .drain(sp..)
                    .enumerate()
                    .map(|(idx, value)| {
                        let index = idx as IntType + *start_key;
                        (index.into(), value)
                    })
                    .collect();
                if let LuaValue::Table(table) = thread_mut.data_stack.last().unwrap() {
                    table.borrow_mut().arr.append(&mut values);
                } else {
                    unreachable!("table must be on top of stack");
                }
            }

            Instruction::TableIndex => {
                self.index(chunk)?;
            }
            Instruction::TableIndexSet => {
                self.newindex(chunk)?;
            }

            Instruction::FunctionInit(func_id, num_upvalues) => {
                let func = LuaFunctionLua {
                    function_id: *func_id,
                    upvalues: Vec::with_capacity(*num_upvalues),
                };
                let func = LuaFunction::LuaFunc(func);
                self.push(func.into());
            }
            Instruction::FunctionInitUpvalueFromLocalVar(src_local_id) => {
                let mut thread_mut = self.running_thread.borrow_mut();
                let local_idx = *src_local_id + thread_mut.bp;
                let local_var = thread_mut.local_variables.get_mut(local_idx).unwrap();
                // upvalue must be reference.
                let local_var = match local_var {
                    RefOrValue::Ref(r) => Rc::clone(r),
                    RefOrValue::Value(v) => {
                        let reffed_var = Rc::new(RefCell::new(v.clone()));
                        *local_var = RefOrValue::Ref(Rc::clone(&reffed_var));
                        reffed_var
                    }
                };
                match thread_mut.data_stack.last().unwrap() {
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
                let value = Rc::clone(
                    &self
                        .running_thread
                        .borrow()
                        .function_stack
                        .last()
                        .unwrap()
                        .function_object
                        .upvalues[*src_upvalue_id],
                );

                match self.running_thread.borrow().data_stack.last().unwrap() {
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
                let value = RefCell::borrow(
                    &self
                        .running_thread
                        .borrow()
                        .function_stack
                        .last()
                        .unwrap()
                        .function_object
                        .upvalues[*upvalue_id],
                )
                .clone();
                self.push(value);
            }
            Instruction::FunctionUpvalueSet(upvalue_id) => {
                let top = self.pop();
                *self
                    .running_thread
                    .borrow()
                    .function_stack
                    .last()
                    .unwrap()
                    .function_object
                    .upvalues[*upvalue_id]
                    .borrow_mut() = top;
            }

            Instruction::BinaryAdd => {
                self.add(chunk)?;
            }
            Instruction::BinarySub => {
                self.sub(chunk)?;
            }
            Instruction::BinaryMul => {
                self.mul(chunk)?;
            }
            Instruction::BinaryDiv => {
                self.div(chunk)?;
            }
            Instruction::BinaryFloorDiv => {
                self.idiv(chunk)?;
            }
            Instruction::BinaryMod => {
                self.mod_(chunk)?;
            }
            Instruction::BinaryPow => {
                self.pow(chunk)?;
            }
            Instruction::BinaryConcat => {
                self.concat(chunk)?;
            }
            Instruction::BinaryBitwiseAnd => {
                self.band(chunk)?;
            }
            Instruction::BinaryBitwiseOr => {
                self.bor(chunk)?;
            }
            Instruction::BinaryBitwiseXor => {
                self.bxor(chunk)?;
            }
            Instruction::BinaryShiftLeft => {
                self.shl(chunk)?;
            }
            Instruction::BinaryShiftRight => {
                self.shr(chunk)?;
            }
            Instruction::BinaryEqual => {
                self.eq(chunk)?;
            }
            Instruction::BinaryLessThan => {
                self.lt(chunk)?;
            }
            Instruction::BinaryLessEqual => {
                self.le(chunk)?;
            }

            Instruction::UnaryMinus => {
                self.unm(chunk)?;
            }
            Instruction::UnaryBitwiseNot => {
                self.bnot(chunk)?;
            }
            Instruction::UnaryLength => {
                self.len(chunk)?;
            }
            Instruction::UnaryLogicalNot => {
                let top = self.pop().to_bool();
                self.push((!top).into());
            }

            Instruction::FunctionCall(expected_ret) => {
                let (func, num_args) = {
                    let mut thread_mut = self.running_thread.borrow_mut();
                    let func = thread_mut.data_stack.pop().unwrap();
                    let sp = thread_mut.usize_stack.pop().unwrap();
                    let num_args = thread_mut.data_stack.len() - sp;
                    (func, num_args)
                };
                self.function_call(chunk, num_args, func, *expected_ret)?;
            }

            // sp -> top
            Instruction::Return => {
                let mut thread_mut = self.running_thread.borrow_mut();
                if let Some(func) = thread_mut.function_stack.pop() {
                    // return from function call
                    let old_stacklen = thread_mut.usize_stack.pop().unwrap();
                    let old_bp = thread_mut.usize_stack.pop().unwrap();
                    let old_pc = thread_mut.usize_stack.pop().unwrap();
                    let old_local_len = thread_mut.bp;
                    thread_mut.local_variables.truncate(old_local_len);
                    thread_mut.bp = old_bp;
                    thread_mut.counter = old_pc;

                    if let Some(expected) = func.return_expected {
                        let adjusted = old_stacklen + expected;
                        thread_mut
                            .data_stack
                            .resize_with(adjusted, Default::default);
                    }
                    return Ok(());
                } else {
                    // main chunk
                    thread_mut.counter = chunk.instructions.len();
                    return Ok(());
                }
            }

            Instruction::GetVariadic(expected) => {
                if let Some(expected) = expected {
                    let mut variadic = self
                        .running_thread
                        .borrow()
                        .function_stack
                        .last()
                        .unwrap()
                        .variadic
                        .clone();
                    variadic.resize_with(*expected, Default::default);
                    self.running_thread
                        .borrow_mut()
                        .data_stack
                        .append(&mut variadic);
                } else {
                    let mut variadic = self
                        .running_thread
                        .borrow()
                        .function_stack
                        .last()
                        .unwrap()
                        .variadic
                        .clone();
                    self.running_thread
                        .borrow_mut()
                        .data_stack
                        .append(&mut variadic);
                }
            }
        }
        self.running_thread.borrow_mut().counter += 1;
        Ok(())
    }
    /// run the whole chunk
    pub fn run(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        loop {
            let counter = self.running_thread.borrow().counter;
            if let Some(instruction) = chunk.instructions.get(counter) {
                self.cycle(chunk, instruction)?;
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

#[derive(Debug, Clone, Copy)]
pub enum ThreadStatus {
    Init,    // initialized, `resume` not called
    Running, // `resume` called, not finished
    Dead,    // reached end of chunk
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

    /// status of this thread.
    /// `None` if this thread is main thread.
    pub status: Option<ThreadStatus>,

    pub func: Option<Rc<RefCell<LuaFunction>>>,
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
            status: None,
            func: None,
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

    pub fn adjust(&mut self, inserted: usize, expected: Option<usize>) {
        if let Some(expected) = expected {
            let adjusted = self.data_stack.len() - inserted + expected;
            self.data_stack.resize_with(adjusted, Default::default);
        }
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
