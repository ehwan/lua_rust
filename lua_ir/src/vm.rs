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

    /// coroutine stack
    pub(crate) coroutines: Vec<Rc<RefCell<LuaThread>>>,

    /// last operation (for error message)
    pub(crate) last_op: String,
}

impl LuaEnv {
    pub fn new(chunk: Chunk) -> LuaEnv {
        let env = Rc::new(RefCell::new(builtin::init_env().unwrap()));
        env.borrow_mut()
            .insert("_G".into(), LuaValue::Table(Rc::clone(&env)));
        let main_thread = Rc::new(RefCell::new(LuaThread::new_main(&chunk)));
        LuaEnv {
            env: LuaValue::Table(env),
            rng: rand::rngs::StdRng::from_entropy(),

            chunk,

            coroutines: vec![main_thread],
            last_op: "last_op".to_string(),
        }
    }

    pub fn main_thread(&self) -> &Rc<RefCell<LuaThread>> {
        self.coroutines.first().unwrap()
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
        error_wrapper: impl FnOnce(&'static str) -> RuntimeError,
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
                None => Err(error_wrapper(lhs.type_str())),
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
            (lhs, rhs) => self.try_call_metamethod(
                lhs,
                rhs,
                "__add",
                false,
                RuntimeError::AttemptToArithmeticOn,
            ),
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
            (lhs, rhs) => self.try_call_metamethod(
                lhs,
                rhs,
                "__sub",
                false,
                RuntimeError::AttemptToArithmeticOn,
            ),
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
            (lhs, rhs) => self.try_call_metamethod(
                lhs,
                rhs,
                "__mul",
                false,
                RuntimeError::AttemptToArithmeticOn,
            ),
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
            (lhs, rhs) => self.try_call_metamethod(
                lhs,
                rhs,
                "__div",
                false,
                RuntimeError::AttemptToArithmeticOn,
            ),
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
            (lhs, rhs) => self.try_call_metamethod(
                lhs,
                rhs,
                "__mod",
                false,
                RuntimeError::AttemptToArithmeticOn,
            ),
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
            (lhs, rhs) => self.try_call_metamethod(
                lhs,
                rhs,
                "__pow",
                false,
                RuntimeError::AttemptToArithmeticOn,
            ),
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
                _ => Err(RuntimeError::AttemptToArithmeticOn(lhs.type_str())),
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
            (lhs, rhs) => self.try_call_metamethod(
                lhs,
                rhs,
                "__idiv",
                false,
                RuntimeError::AttemptToArithmeticOn,
            ),
        }
    }
    /// bitwise and operation with __band metamethod
    pub fn band(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Ok(lhs), Ok(rhs)) => {
                        self.push((lhs & rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(
                        lhs,
                        rhs,
                        "__band",
                        false,
                        RuntimeError::AttemptToBitwiseOn,
                    ),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(
                lhs,
                rhs,
                "__band",
                false,
                RuntimeError::AttemptToBitwiseOn,
            ),
        }
    }
    /// bitwise or operation with __bor metamethod
    pub fn bor(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Ok(lhs), Ok(rhs)) => {
                        self.push((lhs | rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(
                        lhs,
                        rhs,
                        "__bor",
                        false,
                        RuntimeError::AttemptToBitwiseOn,
                    ),
                }
            }
            // else, try to call metamethod, search on left first
            _ => {
                self.try_call_metamethod(lhs, rhs, "__bor", false, RuntimeError::AttemptToBitwiseOn)
            }
        }
    }
    /// bitwise xor operation with __bxor metamethod
    pub fn bxor(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Ok(lhs), Ok(rhs)) => {
                        self.push((lhs ^ rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(
                        lhs,
                        rhs,
                        "__bxor",
                        false,
                        RuntimeError::AttemptToBitwiseOn,
                    ),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(
                lhs,
                rhs,
                "__bxor",
                false,
                RuntimeError::AttemptToBitwiseOn,
            ),
        }
    }
    /// bitwise shift left operation with __shl metamethod
    pub fn shl(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Ok(lhs), Ok(rhs)) => {
                        self.push((lhs << rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(
                        lhs,
                        rhs,
                        "__shl",
                        false,
                        RuntimeError::AttemptToBitwiseOn,
                    ),
                }
            }
            // else, try to call metamethod, search on left first
            _ => {
                self.try_call_metamethod(lhs, rhs, "__shl", false, RuntimeError::AttemptToBitwiseOn)
            }
        }
    }
    /// bitwise shift right operation with __shr metamethod
    pub fn shr(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Ok(lhs), Ok(rhs)) => {
                        self.push((lhs >> rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(
                        lhs,
                        rhs,
                        "__shr",
                        false,
                        RuntimeError::AttemptToBitwiseOn,
                    ),
                }
            }
            // else, try to call metamethod, search on left first
            _ => {
                self.try_call_metamethod(lhs, rhs, "__shr", false, RuntimeError::AttemptToBitwiseOn)
            }
        }
    }
    /// bitwise not operation with __bnot metamethod
    pub fn bnot(&mut self) -> Result<(), RuntimeError> {
        let lhs = self.pop();
        match &lhs {
            LuaValue::Number(lhs_num) => match lhs_num.try_to_int() {
                Ok(i) => {
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
                    _ => Err(RuntimeError::AttemptToBitwiseOn(lhs.type_str())),
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
                _ => Err(RuntimeError::AttemptToBitwiseOn(lhs.type_str())),
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
                _ => self.try_call_metamethod(
                    lhs,
                    rhs,
                    "__concat",
                    false,
                    RuntimeError::AttemptToConcatenate,
                ),
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
                _ => self.try_call_metamethod(
                    LuaValue::String(lhs_str),
                    rhs,
                    "__concat",
                    false,
                    RuntimeError::AttemptToConcatenate,
                ),
            },

            _ => self.try_call_metamethod(
                lhs,
                rhs,
                "__concat",
                false,
                RuntimeError::AttemptToConcatenate,
            ),
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
                _ => Err(RuntimeError::AttemptToGetLengthOf(lhs.type_str())),
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
                    _ =>
                    // @TODO : error message
                    {
                        Err(RuntimeError::Custom("__index metamethod not found".into()))
                    }
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
                    _ => Err(RuntimeError::Custom(
                        "__newindex metamethod not found".into(),
                    )),
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
                    // @TODO
                    self.try_call_metamethod(
                        LuaValue::Table(lhs),
                        LuaValue::Table(rhs),
                        "__eq",
                        false,
                        RuntimeError::AttemptToArithmeticOn,
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
            // @TODO error type
            (lhs, rhs) => self.try_call_metamethod(
                lhs,
                rhs,
                "__lt",
                false,
                RuntimeError::AttemptToArithmeticOn,
            ),
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
            // @TODO error type
            (lhs, rhs) => self.try_call_metamethod(
                lhs,
                rhs,
                "__le",
                false,
                RuntimeError::AttemptToArithmeticOn,
            ),
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
                        let upvalues = lua_internal.upvalues.clone();
                        let function_id = lua_internal.function_id;
                        drop(func_borrow);
                        self.function_call_lua(args_num, upvalues, function_id, expected_ret)?;
                        if force_wait {
                            let coroutine_len = self.coroutines.len();
                            let call_stack_len = self.running_thread().borrow().call_stack.len();
                            loop {
                                if coroutine_len == self.coroutines.len()
                                    && self.coroutines[coroutine_len - 1].borrow().call_stack.len()
                                        == call_stack_len - 1
                                {
                                    break;
                                }
                                if self.cycle()? == false {
                                    break;
                                }
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
                    // @TODO : error message
                    Err(RuntimeError::Custom("__call metamethod not found".into()))
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
        if func_info.is_variadic {
            let (variadic, rest_args_num) = if args_num <= func_info.args {
                (Vec::new(), args_num)
            } else {
                (
                    thread_mut
                        .data_stack
                        .drain(thread_mut.data_stack.len() - args_num + func_info.args..)
                        .collect(),
                    func_info.args,
                )
            };
            // push call stack frame
            thread_mut.call_stack.push(CallStackFrame {
                upvalues,
                return_expected: expected_ret,
                variadic: variadic,
                bp: thread_mut.bp,
                counter: thread_mut.counter,
                data_stack: thread_mut.data_stack.len() - args_num,
                local_variables: thread_mut.local_variables.len(),
            });

            // set base pointer to new stack frame
            thread_mut.bp = thread_mut.local_variables.len();
            // reserve stack space for local variables
            thread_mut.local_variables.resize_with(
                thread_mut.local_variables.len() + func_info.local_variables,
                Default::default,
            );

            // copy arguments to local variables
            for (idx, arg) in thread_mut
                .data_stack
                .drain(thread_mut.data_stack.len() - rest_args_num..)
                .enumerate()
            {
                thread_mut.local_variables[thread_mut.bp + idx] = RefOrValue::Value(arg);
            }
        } else {
            // push call stack frame
            thread_mut.call_stack.push(CallStackFrame {
                upvalues,
                return_expected: expected_ret,
                variadic: Vec::new(),
                bp: thread_mut.bp,
                counter: thread_mut.counter,
                data_stack: thread_mut.data_stack.len() - args_num,
                local_variables: thread_mut.local_variables.len(),
            });

            // set base pointer to new stack frame
            thread_mut.bp = thread_mut.local_variables.len();
            // reserve stack space for local variables
            thread_mut.local_variables.resize_with(
                thread_mut.local_variables.len() + func_info.local_variables,
                Default::default,
            );

            // copy arguments to local variables
            for (idx, arg) in thread_mut
                .data_stack
                .drain(thread_mut.data_stack.len() - args_num..)
                .take(func_info.args)
                .enumerate()
            {
                thread_mut.local_variables[thread_mut.bp + idx] = RefOrValue::Value(arg);
            }
        };

        // move program counter to function start
        thread_mut.counter = func_info.address;

        Ok(())
    }

    /// execute single instruction
    pub fn run_instruction(&mut self, instruction: Instruction) -> Result<(), RuntimeError> {
        debug_assert!(self.coroutines.is_empty() == false);
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
                        .call_stack
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
                        .call_stack
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
                    .call_stack
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
                    if thread_mut.call_stack.len() == 1 {
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
                        if let Some(func) = thread_mut.call_stack.pop() {
                            // return from function call
                            thread_mut.local_variables.truncate(func.local_variables);
                            thread_mut.bp = func.bp;
                            thread_mut.counter = func.counter;

                            if let Some(expected) = func.return_expected {
                                let adjusted = func.data_stack + expected;
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
                    if let Some(func) = thread_mut.call_stack.pop() {
                        // return from function call
                        thread_mut.local_variables.truncate(func.local_variables);
                        thread_mut.bp = func.bp;
                        thread_mut.counter = func.counter;

                        if let Some(expected) = func.return_expected {
                            let adjusted = func.data_stack + expected;
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
                        .call_stack
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
                        .call_stack
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
    pub fn cycle(&mut self) -> Result<bool, RuntimeError> {
        if self.coroutines.len() == 0 {
            return Ok(false);
        }
        let mut thread_mut = self.running_thread().borrow_mut();
        if let Some(instruction) = self.chunk.instructions.get(thread_mut.counter) {
            thread_mut.counter += 1;
            drop(thread_mut);
            match self.run_instruction(instruction.clone()) {
                Ok(_) => return Ok(true),
                Err(err) => {
                    // if this error was occured in main chunk, just return it
                    if self.coroutines.len() == 1 {
                        return Err(err);
                    } else {
                        // if this error was occured in coroutine, propagate it to parent coroutine
                        let error_object = err.into_lua_value(self);

                        // return 'false' and 'error_object' to parent's 'resume()'
                        self.coroutines.pop().unwrap().borrow_mut().set_dead();
                        let status = self.running_thread().borrow().status;
                        if let ThreadStatus::ResumePending(resume_expected) = status {
                            match resume_expected {
                                Some(0) => {}
                                Some(1) => {
                                    self.running_thread()
                                        .borrow_mut()
                                        .data_stack
                                        .push(false.into());
                                }
                                Some(resume_expected) => {
                                    self.push2(false.into(), error_object);
                                    self.running_thread().borrow_mut().data_stack.extend(
                                        std::iter::repeat(LuaValue::Nil).take(resume_expected - 2),
                                    );
                                }
                                None => {
                                    self.push2(false.into(), error_object);
                                }
                            }
                        } else {
                            unreachable!("coroutine must be in resume pending state");
                        }
                        self.running_thread().borrow_mut().status = ThreadStatus::Running;

                        Ok(true)
                    }
                }
            }
        } else {
            Ok(false)
        }
    }
    /// run the whole chunk
    pub fn run(&mut self) -> Result<(), RuntimeError> {
        while self.cycle()? {}
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CallStackFrame {
    /// upvalues
    pub upvalues: Vec<Rc<RefCell<LuaValue>>>,
    /// number of return values expected.
    pub return_expected: Option<usize>,
    /// variadic arguments
    pub variadic: Vec<LuaValue>,

    /// data_stack.len() to restore when return
    pub data_stack: usize,
    /// bp to restore when return
    pub bp: usize,
    /// program counter to restore when return
    pub counter: usize,
    /// local_variables.len() to restore when return
    pub local_variables: usize,
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

/// for error handling, recovering state
#[derive(Debug, Clone)]
pub struct ThreadState {
    pub local_variables: usize,
    pub data_stack: usize,
    pub usize_stack: usize,
    pub call_stack: usize,
    pub counter: usize,
    pub bp: usize,
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
    pub call_stack: Vec<CallStackFrame>,

    /// current instruction counter
    pub counter: usize,

    /// if this thread is a coroutine, this is the function object of the coroutine
    pub func: Option<Rc<RefCell<LuaFunction>>>,

    pub status: ThreadStatus,
}
impl LuaThread {
    pub fn new_main(chunk: &Chunk) -> LuaThread {
        let mut local_variables = Vec::new();
        local_variables.resize_with(chunk.stack_size, Default::default);
        LuaThread {
            local_variables,
            data_stack: Vec::new(),
            usize_stack: Vec::new(),
            call_stack: Vec::new(),
            counter: 0,
            bp: 0,
            func: None,
            status: ThreadStatus::Running,
        }
    }
    pub fn new_coroutine(env: &LuaEnv, func: Rc<RefCell<LuaFunction>>) -> LuaThread {
        let mut local_variables = Vec::new();
        let func_borrow = func.borrow();
        match &*func_borrow {
            LuaFunction::LuaFunc(func) => {
                local_variables.resize_with(
                    env.chunk.functions[func.function_id].local_variables,
                    Default::default,
                );
            }
            _ => {}
        }
        drop(func_borrow);
        LuaThread {
            local_variables,
            data_stack: Vec::new(),
            usize_stack: Vec::new(),
            call_stack: Vec::new(),
            counter: usize::MAX,
            bp: 0,
            func: Some(func),
            status: ThreadStatus::NotStarted,
        }
    }

    pub(crate) fn to_state(&self) -> ThreadState {
        ThreadState {
            local_variables: self.local_variables.len(),
            data_stack: self.data_stack.len(),
            usize_stack: self.usize_stack.len(),
            call_stack: self.call_stack.len(),
            counter: self.counter,
            bp: self.bp,
        }
    }
    pub(crate) fn from_state(&mut self, state: ThreadState) {
        debug_assert!(self.status == ThreadStatus::Running);
        self.local_variables.truncate(state.local_variables);
        self.data_stack.truncate(state.data_stack);
        self.usize_stack.truncate(state.usize_stack);
        self.call_stack.truncate(state.call_stack);
        self.counter = state.counter;
        self.bp = state.bp;
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
