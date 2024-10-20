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

    pub(crate) chunk: Chunk,

    /// main thread
    pub(crate) main_thread: Rc<RefCell<LuaThread>>,
    /// coroutine stack
    pub(crate) coroutines: Vec<Rc<RefCell<LuaThread>>>,
}

impl LuaEnv {
    pub fn new(chunk: Chunk) -> LuaEnv {
        let env = Rc::new(RefCell::new(builtin::init_env().unwrap()));
        env.borrow_mut()
            .insert("_G".into(), LuaValue::Table(Rc::clone(&env)));
        let main_thread = Rc::new(RefCell::new(LuaThread::new_main()));
        LuaEnv {
            env: LuaValue::Table(env),
            rng: rand::rngs::StdRng::from_entropy(),

            chunk,

            main_thread: Rc::clone(&main_thread),
            coroutines: vec![main_thread],
        }
    }

    pub fn main_thread(&self) -> &Rc<RefCell<LuaThread>> {
        &self.main_thread
    }
    pub fn running_thread(&self) -> &Rc<RefCell<LuaThread>> {
        self.coroutines.last().unwrap()
    }

    // pub fn error(&self, error_obj: LuaValue) -> RuntimeError {}

    pub fn push(&self, value: LuaValue) {
        self.running_thread().borrow_mut().data_stack.push(value);
    }
    pub fn push2(&self, value1: LuaValue, value2: LuaValue) {
        let mut thread = self.running_thread().borrow_mut();
        thread.data_stack.push(value1);
        thread.data_stack.push(value2);
    }
    pub fn push3(&self, value1: LuaValue, value2: LuaValue, value3: LuaValue) {
        let mut thread = self.running_thread().borrow_mut();
        thread.data_stack.push(value1);
        thread.data_stack.push(value2);
        thread.data_stack.push(value3);
    }
    pub fn pop(&self) -> LuaValue {
        self.running_thread().borrow_mut().data_stack.pop().unwrap()
    }
    pub fn pop2(&self) -> (LuaValue, LuaValue) {
        let mut thread = self.running_thread().borrow_mut();
        let value2 = thread.data_stack.pop().unwrap();
        let value1 = thread.data_stack.pop().unwrap();
        (value1, value2)
    }
    pub fn pop3(&self) -> (LuaValue, LuaValue, LuaValue) {
        let mut thread = self.running_thread().borrow_mut();
        let value3 = thread.data_stack.pop().unwrap();
        let value2 = thread.data_stack.pop().unwrap();
        let value1 = thread.data_stack.pop().unwrap();
        (value1, value2, value3)
    }
    pub fn pop4(&self) -> (LuaValue, LuaValue, LuaValue, LuaValue) {
        let mut thread = self.running_thread().borrow_mut();
        let value4 = thread.data_stack.pop().unwrap();
        let value3 = thread.data_stack.pop().unwrap();
        let value2 = thread.data_stack.pop().unwrap();
        let value1 = thread.data_stack.pop().unwrap();
        (value1, value2, value3, value4)
    }
    pub fn pop_n(&self, n: usize) {
        let mut thread_mut = self.running_thread().borrow_mut();
        let len = thread_mut.data_stack.len();
        thread_mut.data_stack.truncate(len - n);
    }
    pub fn top(&self) -> LuaValue {
        self.running_thread()
            .borrow()
            .data_stack
            .last()
            .unwrap()
            .clone()
    }
    /// get i'th value from top of the stack
    pub fn top_i(&self, i: usize) -> LuaValue {
        let thread = self.running_thread().borrow();
        let idx = thread.data_stack.len() - i - 1;
        thread.data_stack.get(idx).unwrap().clone()
    }
    pub fn borrow_running_thread(&self) -> std::cell::Ref<LuaThread> {
        self.running_thread().borrow()
    }
    pub fn borrow_running_thread_mut(&self) -> std::cell::RefMut<LuaThread> {
        self.running_thread().borrow_mut()
    }
    // pub fn fill_nil(&self, n: usize) {
    //     let mut thread = self.running_thread().borrow_mut();
    //     thread
    //         .data_stack
    //         .extend(std::iter::repeat(LuaValue::Nil).take(n));
    // }

    pub fn get_metavalue(&self, value: &LuaValue, key: &str) -> Option<LuaValue> {
        match value {
            // @TODO: link `string` module here
            LuaValue::String(s) => {
                let s = String::from_utf8_lossy(s);
                match key {
                    "__name" => Some(LuaValue::String(s.as_bytes().to_vec())),
                    _ => None,
                }
            }
            LuaValue::Table(table) => table.borrow().get_metavalue(key),
            _ => None,
        }
    }

    /// Try to call binary metamethod f(lhs, rhs).
    /// It tries to search metamethod on lhs first, then rhs.
    fn try_call_metamethod(
        &mut self,
        lhs: LuaValue,
        rhs: LuaValue,
        meta_name: &str,
        force_wait: bool,
    ) -> Result<(), RuntimeError> {
        match self.get_metavalue(&lhs, meta_name) {
            Some(meta) => {
                self.push2(lhs, rhs);
                self.function_call(2, meta, Some(1), force_wait)
            }
            None => match self.get_metavalue(&rhs, meta_name) {
                Some(meta) => {
                    self.push2(lhs, rhs);
                    self.function_call(2, meta, Some(1), force_wait)
                }
                None => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// string-fy a value
    pub fn tostring(&mut self) -> Result<(), RuntimeError> {
        let top = self.pop();
        let meta = self.get_metavalue(&top, "__tostring");
        match meta {
            Some(meta) => {
                self.push(top);
                self.function_call(1, meta, Some(1), true)?;
                self.tostring()
            }
            _ => {
                let name = self.get_metavalue(&top, "__name");
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
    pub fn add(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            // if both are numbers, add them
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs + rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(lhs, rhs, "__add", false),
        }
    }

    /// sub operation with __sub metamethod
    pub fn sub(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs - rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(lhs, rhs, "__sub", false),
        }
    }
    /// mul operation with __mul metamethod
    pub fn mul(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs * rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(lhs, rhs, "__mul", false),
        }
    }
    /// div operation with __div metamethod
    pub fn div(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs / rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(lhs, rhs, "__div", false),
        }
    }
    /// mod operation with __mod metamethod
    pub fn mod_(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs % rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(lhs, rhs, "__mod", false),
        }
    }
    /// pow operation with __pow metamethod
    pub fn pow(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs.pow(rhs)).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(lhs, rhs, "__pow", false),
        }
    }
    /// unary minus operation with __unm metamethod
    pub fn unm(&mut self) -> Result<(), RuntimeError> {
        let lhs = self.pop();
        match lhs {
            LuaValue::Number(num) => {
                self.push((-num).into());
                Ok(())
            }
            lhs => match self.get_metavalue(&lhs, "__unm") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    self.push2(lhs.clone(), lhs);
                    self.function_call(2, meta, Some(1), false)
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// floor division operation with __idiv metamethod
    pub fn idiv(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs.floor_div(rhs)).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(lhs, rhs, "__idiv", false),
        }
    }
    /// bitwise and operation with __band metamethod
    pub fn band(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        self.push((lhs & rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(lhs, rhs, "__band", false),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(lhs, rhs, "__band", false),
        }
    }
    /// bitwise or operation with __bor metamethod
    pub fn bor(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        self.push((lhs | rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(lhs, rhs, "__bor", false),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(lhs, rhs, "__bor", false),
        }
    }
    /// bitwise xor operation with __bxor metamethod
    pub fn bxor(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        self.push((lhs ^ rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(lhs, rhs, "__bxor", false),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(lhs, rhs, "__bxor", false),
        }
    }
    /// bitwise shift left operation with __shl metamethod
    pub fn shl(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        self.push((lhs << rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(lhs, rhs, "__shl", false),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(lhs, rhs, "__shl", false),
        }
    }
    /// bitwise shift right operation with __shr metamethod
    pub fn shr(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        self.push((lhs >> rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(lhs, rhs, "__shr", false),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(lhs, rhs, "__shr", false),
        }
    }
    /// bitwise not operation with __bnot metamethod
    pub fn bnot(&mut self) -> Result<(), RuntimeError> {
        let lhs = self.pop();
        match &lhs {
            LuaValue::Number(lhs_num) => match lhs_num.try_to_int() {
                Some(i) => {
                    self.push((!i).into());
                    Ok(())
                }
                _ => match self.get_metavalue(&lhs, "__bnot") {
                    Some(meta) => {
                        // For the unary operators (negation, length, and bitwise NOT),
                        // the metamethod is computed and called with a dummy second operand
                        // equal to the first one.
                        self.push2(lhs.clone(), lhs);
                        self.function_call(2, meta, Some(1), false)
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
            _ => match self.get_metavalue(&lhs, "__bnot") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    self.push2(lhs.clone(), lhs);
                    self.function_call(2, meta, Some(1), false)
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// concat operation with __concat metamethod
    pub fn concat(&mut self) -> Result<(), RuntimeError> {
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
                _ => self.try_call_metamethod(lhs, rhs, "__concat", false),
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
                _ => self.try_call_metamethod(LuaValue::String(lhs_str), rhs, "__concat", false),
            },

            _ => self.try_call_metamethod(lhs, rhs, "__concat", false),
        }
    }
    /// `#` length operation with __len metamethod
    pub fn len(&mut self) -> Result<(), RuntimeError> {
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
                        self.function_call(2, meta, Some(1), false)
                    }
                    _ => {
                        self.push((table.borrow().len() as IntType).into());
                        Ok(())
                    }
                }
            }
            lhs => match self.get_metavalue(&lhs, "__len") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    self.push2(lhs.clone(), lhs);
                    self.function_call(2, meta, Some(1), false)
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// table index get operation with __index metamethod
    pub fn index(&mut self) -> Result<(), RuntimeError> {
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
                            self.function_call(2, LuaValue::Function(meta_func), Some(1), false)
                        }
                        Some(LuaValue::Table(meta_table)) => {
                            self.push2(LuaValue::Table(meta_table), key);
                            self.index()
                        }
                        _ => {
                            self.push(LuaValue::Nil);
                            Ok(())
                        }
                    }
                }
            }
            table => {
                let meta = self.get_metavalue(&table, "__index");
                match meta {
                    Some(LuaValue::Function(meta_func)) => {
                        self.push2(table, key);
                        self.function_call(2, LuaValue::Function(meta_func), Some(1), false)
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        self.push2(LuaValue::Table(meta_table), key);
                        self.index()
                    }
                    _ => Err(RuntimeError::NotTable),
                }
            }
        }
    }
    /// table index set operation with __newindex metamethod
    pub fn newindex(&mut self) -> Result<(), RuntimeError> {
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
                        self.function_call(3, LuaValue::Function(meta_func), Some(0), false)
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        self.push3(value, LuaValue::Table(meta_table), key);
                        self.newindex()
                    }
                    _ => {
                        table.borrow_mut().insert(key, value);
                        Ok(())
                    }
                }
            }
            table => {
                let meta = self.get_metavalue(&table, "__newindex");
                match meta {
                    Some(LuaValue::Function(meta_func)) => {
                        self.push3(table, key, value);
                        self.function_call(3, LuaValue::Function(meta_func), Some(0), false)
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        self.push3(value, LuaValue::Table(meta_table), key);
                        self.newindex()
                    }
                    _ => Err(RuntimeError::NotTable),
                }
            }
        }
    }
    /// equality operation with __eq metamethod
    pub fn eq(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();

        match (lhs, rhs) {
            (LuaValue::Table(lhs), LuaValue::Table(rhs)) => {
                if Rc::ptr_eq(&lhs, &rhs) {
                    self.push(LuaValue::Boolean(true));
                    return Ok(());
                } else {
                    self.try_call_metamethod(
                        LuaValue::Table(lhs),
                        LuaValue::Table(rhs),
                        "__eq",
                        false,
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
    pub fn lt(&mut self) -> Result<(), RuntimeError> {
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
            (lhs, rhs) => self.try_call_metamethod(lhs, rhs, "__lt", false),
        }
    }

    /// less than or equal operation with __le metamethod
    pub fn le(&mut self) -> Result<(), RuntimeError> {
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
            (lhs, rhs) => self.try_call_metamethod(lhs, rhs, "__le", false),
        }
    }

    /// function call with __call metamethod.
    /// if `force_wait` is true, this does not return until the function call is finished.
    /// that is, if `force_wait` is false, this function just pushes data to the function-call-stack, and returns.
    /// user must call `run_instruction` to actually run the function.
    pub fn function_call(
        &mut self,
        // number of arguments actually passed
        args_num: usize,
        // function object
        func: LuaValue,
        // number of return values expected; returned values will be adjusted to this number
        expected_ret: Option<usize>,
        force_wait: bool,
    ) -> Result<(), RuntimeError> {
        match func {
            LuaValue::Function(func) => {
                let func_borrow = func.borrow();
                match &*func_borrow {
                    LuaFunction::LuaFunc(lua_internal) => {
                        let usize_len0 = self.running_thread().borrow().usize_stack.len();
                        let upvalues = lua_internal.upvalues.clone();
                        let function_id = lua_internal.function_id;
                        drop(func_borrow);
                        self.function_call_lua(args_num, upvalues, function_id, expected_ret)?;
                        if force_wait {
                            loop {
                                if self.running_thread().borrow().usize_stack.len() == usize_len0 {
                                    break;
                                }
                                let instruction = self
                                    .chunk
                                    .instructions
                                    .get(self.running_thread().borrow().counter)
                                    .unwrap()
                                    .clone();
                                self.running_thread().borrow_mut().counter += 1;
                                self.run_instruction(instruction)?;
                            }
                        }
                        Ok(())
                    }
                    LuaFunction::RustFunc(rust_internal) => {
                        rust_internal(self, args_num, expected_ret)
                    }
                }
            }
            other => {
                let func = self.get_metavalue(&other, "__call");
                if let Some(meta) = func {
                    {
                        // push `self` as first argument
                        let front_arg_pos =
                            self.running_thread().borrow().data_stack.len() - args_num;
                        self.running_thread()
                            .borrow_mut()
                            .data_stack
                            .insert(front_arg_pos, other);
                    }
                    self.function_call(args_num + 1, meta, expected_ret, force_wait)
                } else {
                    Err(RuntimeError::NotFunction)
                }
            }
        }
    }
    pub(crate) fn function_call_lua(
        &mut self,
        // number of arguments actually passed
        args_num: usize,
        upvalues: Vec<Rc<RefCell<LuaValue>>>,
        func_id: usize,
        // number of return values expected; returned values will be adjusted to this number
        expected_ret: Option<usize>,
    ) -> Result<(), RuntimeError> {
        let mut thread_mut_ = self.running_thread().borrow_mut();
        let thread_mut = &mut *thread_mut_;

        let func_info = &self.chunk.functions[func_id];

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
                    .drain(thread_mut.data_stack.len() - args_num + func_info.args..)
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
            upvalues,
            return_expected: expected_ret,
            variadic,
        });

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
        thread_mut
            .local_variables
            .reserve(func_info.local_variables);

        // copy arguments to local variables
        thread_mut.local_variables.extend(
            thread_mut
                .data_stack
                .drain(thread_mut.data_stack.len() - func_info.args..)
                .map(|arg| RefOrValue::Value(arg)),
        );

        // move program counter to function start
        thread_mut.counter = func_info.address;

        Ok(())
    }

    /// execute single instruction
    pub fn run_instruction(&mut self, instruction: Instruction) -> Result<(), RuntimeError> {
        match instruction {
            Instruction::Clone => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let top = thread_mut.data_stack.last().unwrap().clone();
                thread_mut.data_stack.push(top);
            }
            Instruction::Sp => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let len = thread_mut.data_stack.len();
                thread_mut.usize_stack.push(len);
            }
            Instruction::Pop => {
                self.pop();
            }
            Instruction::Deref => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let sp = thread_mut.usize_stack.pop().unwrap();
                let top = thread_mut.data_stack[sp].clone();
                thread_mut.data_stack.push(top);
            }
            Instruction::Jump(label) => {
                let pc = *self.chunk.label_map.get(label).unwrap();
                self.running_thread().borrow_mut().counter = pc;
            }
            Instruction::JumpTrue(label) => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let top = thread_mut.data_stack.pop().unwrap().to_bool();
                if top {
                    let pc = *self.chunk.label_map.get(label).unwrap();
                    thread_mut.counter = pc;
                }
            }
            Instruction::JumpFalse(label) => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let top = thread_mut.data_stack.pop().unwrap().to_bool();
                if !top {
                    let pc = *self.chunk.label_map.get(label).unwrap();
                    thread_mut.counter = pc;
                }
            }
            Instruction::GetLocalVariable(local_id) => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let local_idx = local_id + thread_mut.bp;
                let val = match thread_mut.local_variables.get(local_idx).unwrap() {
                    RefOrValue::Ref(val) => val.borrow().clone(),
                    RefOrValue::Value(val) => val.clone(),
                };
                thread_mut.data_stack.push(val);
            }
            Instruction::SetLocalVariable(local_id) => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let top = thread_mut.data_stack.pop().unwrap();
                let local_idx = local_id + thread_mut.bp;
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
                let mut thread_mut = self.running_thread().borrow_mut();
                let top = thread_mut.data_stack.pop().unwrap();
                let local_idx = local_id + thread_mut.bp;
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
                let mut thread_mut = self.running_thread().borrow_mut();
                let top = thread_mut.data_stack.pop().unwrap();
                thread_mut.data_stack.push(LuaValue::Boolean(top.is_nil()));
            }

            Instruction::Nil => {
                self.push(LuaValue::Nil);
            }
            Instruction::Boolean(b) => {
                self.push(LuaValue::Boolean(b));
            }
            Instruction::Numeric(n) => {
                self.push(LuaValue::Number(n));
            }
            Instruction::String(s) => {
                self.push(LuaValue::String(s));
            }
            Instruction::GetEnv => {
                let env = self.env.clone();
                self.push(env);
            }
            Instruction::TableInit(cap) => {
                let table = LuaTable::with_capacity(cap);
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
                let mut thread_mut = self.running_thread().borrow_mut();
                let sp = thread_mut.usize_stack.pop().unwrap();
                let mut values: BTreeMap<_, _> = thread_mut
                    .data_stack
                    .drain(sp..)
                    .enumerate()
                    .map(|(idx, value)| {
                        let index = idx as IntType + start_key;
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
                self.index()?;
            }
            Instruction::TableIndexSet => {
                self.newindex()?;
            }

            Instruction::FunctionInit(func_id, num_upvalues) => {
                let func = LuaFunctionLua {
                    function_id: func_id,
                    upvalues: Vec::with_capacity(num_upvalues),
                };
                let func = LuaFunction::LuaFunc(func);
                self.push(func.into());
            }
            Instruction::FunctionInitUpvalueFromLocalVar(src_local_id) => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let local_idx = src_local_id + thread_mut.bp;
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
                        .running_thread()
                        .borrow()
                        .function_stack
                        .last()
                        .unwrap()
                        .upvalues[src_upvalue_id],
                );

                match self.running_thread().borrow().data_stack.last().unwrap() {
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
                        .running_thread()
                        .borrow()
                        .function_stack
                        .last()
                        .unwrap()
                        .upvalues[upvalue_id],
                )
                .clone();
                self.push(value);
            }
            Instruction::FunctionUpvalueSet(upvalue_id) => {
                let top = self.pop();
                *self
                    .running_thread()
                    .borrow()
                    .function_stack
                    .last()
                    .unwrap()
                    .upvalues[upvalue_id]
                    .borrow_mut() = top;
            }

            Instruction::BinaryAdd => {
                self.add()?;
            }
            Instruction::BinarySub => {
                self.sub()?;
            }
            Instruction::BinaryMul => {
                self.mul()?;
            }
            Instruction::BinaryDiv => {
                self.div()?;
            }
            Instruction::BinaryFloorDiv => {
                self.idiv()?;
            }
            Instruction::BinaryMod => {
                self.mod_()?;
            }
            Instruction::BinaryPow => {
                self.pow()?;
            }
            Instruction::BinaryConcat => {
                self.concat()?;
            }
            Instruction::BinaryBitwiseAnd => {
                self.band()?;
            }
            Instruction::BinaryBitwiseOr => {
                self.bor()?;
            }
            Instruction::BinaryBitwiseXor => {
                self.bxor()?;
            }
            Instruction::BinaryShiftLeft => {
                self.shl()?;
            }
            Instruction::BinaryShiftRight => {
                self.shr()?;
            }
            Instruction::BinaryEqual => {
                self.eq()?;
            }
            Instruction::BinaryLessThan => {
                self.lt()?;
            }
            Instruction::BinaryLessEqual => {
                self.le()?;
            }

            Instruction::UnaryMinus => {
                self.unm()?;
            }
            Instruction::UnaryBitwiseNot => {
                self.bnot()?;
            }
            Instruction::UnaryLength => {
                self.len()?;
            }
            Instruction::UnaryLogicalNot => {
                let top = self.pop().to_bool();
                self.push((!top).into());
            }

            Instruction::FunctionCall(expected_ret) => {
                let (func, num_args) = {
                    let mut thread_mut = self.running_thread().borrow_mut();
                    let func = thread_mut.data_stack.pop().unwrap();
                    let sp = thread_mut.usize_stack.pop().unwrap();
                    let num_args = thread_mut.data_stack.len() - sp;
                    (func, num_args)
                };
                self.function_call(num_args, func, expected_ret, false)?;
            }

            Instruction::Return => {
                if self.coroutines.len() > 1 {
                    let mut thread_mut = self.running_thread().borrow_mut();
                    if thread_mut.function_stack.len() == 1 {
                        thread_mut.set_dead();
                        drop(thread_mut);
                        let co = self.coroutines.pop().unwrap();
                        let ret_args_len = co.borrow().data_stack.len();
                        let resume_expected = match self.running_thread().borrow().status {
                            ThreadStatus::ResumePending(expected) => expected,
                            _ => unreachable!("coroutine must be in resume pending state"),
                        };
                        self.running_thread().borrow_mut().status = ThreadStatus::Running;
                        self.running_thread()
                            .borrow_mut()
                            .data_stack
                            .push(true.into());
                        self.running_thread()
                            .borrow_mut()
                            .data_stack
                            .append(&mut co.borrow_mut().data_stack);
                        if let Some(resume_expected) = resume_expected {
                            let adjusted =
                                self.running_thread().borrow().data_stack.len() - ret_args_len - 1
                                    + resume_expected;
                            self.running_thread()
                                .borrow_mut()
                                .data_stack
                                .resize_with(adjusted, Default::default);
                        }
                    } else {
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
                        } else {
                            unreachable!("function stack must not be empty");
                        }
                    }
                } else {
                    let mut thread_mut = self.running_thread().borrow_mut();
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
                    } else {
                        // main chunk
                        thread_mut.set_dead();
                        drop(thread_mut);
                        self.coroutines.pop();
                    }
                }
            }

            Instruction::GetVariadic(expected) => {
                if let Some(expected) = expected {
                    let mut variadic = self
                        .running_thread()
                        .borrow()
                        .function_stack
                        .last()
                        .unwrap()
                        .variadic
                        .clone();
                    variadic.resize_with(expected, Default::default);
                    self.running_thread()
                        .borrow_mut()
                        .data_stack
                        .append(&mut variadic);
                } else {
                    let mut variadic = self
                        .running_thread()
                        .borrow()
                        .function_stack
                        .last()
                        .unwrap()
                        .variadic
                        .clone();
                    self.running_thread()
                        .borrow_mut()
                        .data_stack
                        .append(&mut variadic);
                }
            }
        }
        Ok(())
    }
    /// run the whole chunk
    pub fn run(&mut self) -> Result<(), RuntimeError> {
        loop {
            if self.coroutines.len() == 0 {
                break;
            }
            let counter = self.running_thread().borrow().counter;
            if counter >= self.chunk.instructions.len() {
                break;
            }
            self.running_thread().borrow_mut().counter += 1;
            let instruction = self.chunk.instructions.get(counter).unwrap().clone();
            self.run_instruction(instruction)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FunctionStackElem {
    /// upvalues
    pub upvalues: Vec<Rc<RefCell<LuaValue>>>,
    /// number of return values expected.
    pub return_expected: Option<usize>,
    /// variadic arguments
    pub variadic: Vec<LuaValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreadStatus {
    /// thread is not started yet
    NotStarted,
    /// thread is running.
    /// it is not pending on any operation.
    Running,
    /// thread is running, and it is pending on `resume`.
    /// value is the expected number of return values from `resume`.
    ResumePending(Option<usize>),
    /// thread is not running, and it is pending on `yield`.
    /// that is, the thread is waiting for any parent coroutine to `resume`.
    /// value is the expected number of return values from `yield`.
    YieldPending(Option<usize>),
    /// thread is dead, and it is not running anymore.
    Dead,
}
impl ThreadStatus {
    pub fn is_started(&self) -> bool {
        !matches!(self, ThreadStatus::NotStarted)
    }
    pub fn is_dead(&self) -> bool {
        matches!(self, ThreadStatus::Dead)
    }
    pub fn is_yield_pending(&self) -> bool {
        matches!(self, ThreadStatus::YieldPending(_))
    }
    pub fn is_resume_pending(&self) -> bool {
        matches!(self, ThreadStatus::ResumePending(_))
    }
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

    /// if this thread is a coroutine, this is the function object of the coroutine
    pub func: Option<Rc<RefCell<LuaFunction>>>,

    pub status: ThreadStatus,
}
impl LuaThread {
    pub fn new_main() -> LuaThread {
        LuaThread {
            local_variables: Vec::new(),
            data_stack: Vec::new(),
            usize_stack: Vec::new(),
            function_stack: Vec::new(),
            counter: 0,
            bp: 0,
            func: None,
            status: ThreadStatus::Running,
        }
    }
    pub fn new_coroutine(env: &LuaEnv, func: Rc<RefCell<LuaFunction>>) -> LuaThread {
        let func_borrow = func.borrow();
        match &*func_borrow {
            LuaFunction::LuaFunc(lua_func) => {
                let func_info = &env.chunk.functions[lua_func.function_id];
                drop(func_borrow);
                LuaThread {
                    local_variables: Vec::with_capacity(func_info.local_variables),
                    data_stack: Vec::new(),
                    usize_stack: Vec::new(),
                    function_stack: Vec::new(),
                    counter: 0,
                    bp: 0,
                    func: Some(func),
                    status: ThreadStatus::NotStarted,
                }
            }
            LuaFunction::RustFunc(_) => {
                drop(func_borrow);
                LuaThread {
                    local_variables: Vec::new(),
                    data_stack: Vec::new(),
                    usize_stack: Vec::new(),
                    function_stack: Vec::new(),
                    counter: usize::MAX,
                    bp: 0,
                    func: Some(func),
                    status: ThreadStatus::NotStarted,
                }
            }
        }
    }

    pub fn drain_last(&mut self, n: usize) -> impl Iterator<Item = LuaValue> + '_ {
        self.data_stack.drain(self.data_stack.len() - n..)
    }

    pub fn set_dead(&mut self) {
        self.counter = usize::MAX;
        self.status = ThreadStatus::Dead;
    }
    pub fn is_dead(&self) -> bool {
        self.status == ThreadStatus::Dead
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
