use std::cell::RefCell;
use std::rc::Rc;

use lua_tokenizer::IntOrFloat;
use lua_tokenizer::IntType;
use rand::SeedableRng;

use crate::builtin;
use crate::luaval::RefOrValue;
use crate::FunctionInfo;
use crate::LuaFunction;
use crate::LuaFunctionLua;
use crate::LuaTable;
use crate::LuaValue;

use crate::Instruction;
use crate::RuntimeError;

pub struct FunctionStackElem {
    /// function object
    pub function_object: LuaFunctionLua,
    /// number of return values expected.
    pub return_expected: Option<usize>,
    /// variadic arguments
    pub variadic: Vec<LuaValue>,
}

pub struct Stack {
    /// _env
    pub env: LuaValue,

    pub(crate) rng: rand::rngs::StdRng,

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
impl Stack {
    pub fn new(stack_size: usize) -> Stack {
        let mut local_variables = Vec::new();
        local_variables.resize_with(stack_size, || RefOrValue::Value(LuaValue::Nil));
        let env = builtin::init_env().unwrap();
        Stack {
            env: LuaValue::Table(Rc::new(RefCell::new(env))),
            rng: rand::rngs::StdRng::from_entropy(),
            local_variables,
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
                self.data_stack.push(lhs);
                self.data_stack.push(rhs);
                self.function_call(chunk, 2, meta, Some(1))
            }
            None => match rhs.get_metavalue(meta_name) {
                Some(meta) => {
                    self.data_stack.push(lhs);
                    self.data_stack.push(rhs);
                    self.function_call(chunk, 2, meta, Some(1))
                }
                None => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// add operation with __add metamethod
    pub fn add(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs, rhs) {
            // if both are numbers, add them
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push((lhs + rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__add"),
        }
    }

    /// sub operation with __sub metamethod
    pub fn sub(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push((lhs - rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__sub"),
        }
    }
    /// mul operation with __mul metamethod
    pub fn mul(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push((lhs * rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__mul"),
        }
    }
    /// div operation with __div metamethod
    pub fn div(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push((lhs / rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__div"),
        }
    }
    /// mod operation with __mod metamethod
    pub fn mod_(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push((lhs % rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__mod"),
        }
    }
    /// pow operation with __pow metamethod
    pub fn pow(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push((lhs.pow(rhs)).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__pow"),
        }
    }
    /// unary minus operation with __unm metamethod
    pub fn unm(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let lhs = self.data_stack.pop().unwrap();
        match lhs {
            LuaValue::Number(num) => {
                self.data_stack.push((-num).into());
                Ok(())
            }
            lhs => match lhs.get_metavalue("__unm") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    self.data_stack.push(lhs.clone());
                    self.data_stack.push(lhs);
                    self.function_call(chunk, 2, meta, Some(1))
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// floor division operation with __idiv metamethod
    pub fn idiv(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push((lhs.floor_div(rhs)).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__idiv"),
        }
    }
    /// bitwise and operation with __band metamethod
    pub fn band(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        self.data_stack.push((lhs & rhs).into());
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
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        self.data_stack.push((lhs | rhs).into());
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
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        self.data_stack.push((lhs ^ rhs).into());
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
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        self.data_stack.push((lhs << rhs).into());
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
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        self.data_stack.push((lhs >> rhs).into());
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
        let lhs = self.data_stack.pop().unwrap();
        match &lhs {
            LuaValue::Number(lhs_num) => match lhs_num.try_to_int() {
                Some(i) => {
                    self.data_stack.push((!i).into());
                    Ok(())
                }
                _ => match lhs.get_metavalue("__bnot") {
                    Some(meta) => {
                        // For the unary operators (negation, length, and bitwise NOT),
                        // the metamethod is computed and called with a dummy second operand
                        // equal to the first one.
                        self.data_stack.push(lhs.clone());
                        self.data_stack.push(lhs);
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
                    self.data_stack.push(lhs.clone());
                    self.data_stack.push(lhs);
                    self.function_call(chunk, 2, meta, Some(1))
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// concat operation with __concat metamethod
    pub fn concat(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match lhs {
            LuaValue::Number(lhs_num) => match rhs {
                LuaValue::Number(rhs_num) => {
                    let mut concated = lhs_num.to_string().into_bytes();
                    concated.append(&mut rhs_num.to_string().into_bytes());
                    self.data_stack.push(LuaValue::String(concated));
                    Ok(())
                }
                LuaValue::String(mut rhs) => {
                    let mut lhs = lhs_num.to_string().into_bytes();
                    lhs.append(&mut rhs);
                    self.data_stack.push(LuaValue::String(lhs));
                    Ok(())
                }
                _ => self.try_call_metamethod(chunk, lhs, rhs, "__concat"),
            },

            LuaValue::String(lhs_str) => match rhs {
                LuaValue::Number(rhs_num) => {
                    let mut concated = lhs_str;
                    concated.append(&mut rhs_num.to_string().into_bytes());
                    self.data_stack.push(LuaValue::String(concated));
                    Ok(())
                }
                LuaValue::String(mut rhs) => {
                    let mut lhs = lhs_str;
                    lhs.append(&mut rhs);
                    self.data_stack.push(LuaValue::String(lhs));
                    Ok(())
                }
                _ => self.try_call_metamethod(chunk, LuaValue::String(lhs_str), rhs, "__concat"),
            },

            _ => self.try_call_metamethod(chunk, lhs, rhs, "__concat"),
        }
    }
    /// `#` length operation with __len metamethod
    pub fn len(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let lhs = self.data_stack.pop().unwrap();
        match lhs {
            LuaValue::String(s) => {
                self.data_stack.push((s.len() as IntType).into());
                Ok(())
            }
            LuaValue::Table(table) => {
                let meta = table.borrow().get_metavalue("__len");
                match meta {
                    Some(meta) => {
                        // For the unary operators (negation, length, and bitwise NOT),
                        // the metamethod is computed and called with a dummy second operand
                        // equal to the first one.
                        self.data_stack.push(LuaValue::Table(Rc::clone(&table)));
                        self.data_stack.push(LuaValue::Table(table));
                        self.function_call(chunk, 2, meta, Some(1))
                    }
                    _ => {
                        self.data_stack
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
                    self.data_stack.push(lhs.clone());
                    self.data_stack.push(lhs);
                    self.function_call(chunk, 2, meta, Some(1))
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// table index get operation with __index metamethod
    pub fn index(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let key = self.data_stack.pop().unwrap();
        let table = self.data_stack.pop().unwrap();
        match table {
            LuaValue::Table(table) => {
                let get = table.borrow().get(&key).cloned();
                if let Some(get) = get {
                    self.data_stack.push(get);
                    Ok(())
                } else {
                    let meta = table.borrow().get_metavalue("__index");
                    match meta {
                        Some(LuaValue::Function(meta_func)) => {
                            self.data_stack.push(LuaValue::Table(table));
                            self.data_stack.push(key);
                            self.function_call(chunk, 2, LuaValue::Function(meta_func), Some(1))
                        }
                        Some(LuaValue::Table(meta_table)) => {
                            self.data_stack.push(LuaValue::Table(meta_table));
                            self.data_stack.push(key);
                            self.index(chunk)
                        }
                        _ => {
                            self.data_stack.push(LuaValue::Nil);
                            Ok(())
                        }
                    }
                }
            }
            table => {
                let meta = table.get_metavalue("__index");
                match meta {
                    Some(LuaValue::Function(meta_func)) => {
                        self.data_stack.push(table);
                        self.data_stack.push(key);
                        self.function_call(chunk, 2, LuaValue::Function(meta_func), Some(1))
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        self.data_stack.push(LuaValue::Table(meta_table));
                        self.data_stack.push(key);
                        self.index(chunk)
                    }
                    _ => Err(RuntimeError::NotTable),
                }
            }
        }
    }
    /// table index set operation with __newindex metamethod
    pub fn newindex(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let key = self.data_stack.pop().unwrap();
        let table = self.data_stack.pop().unwrap();
        let value = self.data_stack.pop().unwrap();

        match table {
            LuaValue::Table(table) => {
                if let Some(val) = table.borrow_mut().get_mut(&key) {
                    *val = value;
                    return Ok(());
                }
                let meta = table.borrow().get_metavalue("__newindex");
                match meta {
                    Some(LuaValue::Function(meta_func)) => {
                        self.data_stack.push(LuaValue::Table(table));
                        self.data_stack.push(key);
                        self.data_stack.push(value);
                        self.function_call(chunk, 3, LuaValue::Function(meta_func), Some(0))
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        self.data_stack.push(value);
                        self.data_stack.push(LuaValue::Table(meta_table));
                        self.data_stack.push(key);
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
                        self.data_stack.push(table);
                        self.data_stack.push(key);
                        self.data_stack.push(value);
                        self.function_call(chunk, 3, LuaValue::Function(meta_func), Some(0))
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        self.data_stack.push(value);
                        self.data_stack.push(LuaValue::Table(meta_table));
                        self.data_stack.push(key);
                        self.newindex(chunk)
                    }
                    _ => Err(RuntimeError::NotTable),
                }
            }
        }
    }
    /// equality operation with __eq metamethod
    pub fn eq(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();

        match (lhs, rhs) {
            (LuaValue::Table(lhs), LuaValue::Table(rhs)) => {
                if Rc::ptr_eq(&lhs, &rhs) {
                    self.data_stack.push(LuaValue::Boolean(true));
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
                self.data_stack.push(LuaValue::Boolean(lhs == rhs));
                Ok(())
            }
        }
    }
    /// less than operation with __lt metamethod
    pub fn lt(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();

        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push(LuaValue::Boolean(lhs < rhs));
                Ok(())
            }
            (LuaValue::String(lhs), LuaValue::String(rhs)) => {
                self.data_stack.push(LuaValue::Boolean(lhs < rhs));
                Ok(())
            }
            (lhs, rhs) => self.try_call_metamethod(chunk, lhs, rhs, "__lt"),
        }
    }

    /// less than or equal operation with __le metamethod
    pub fn le(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();

        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push(LuaValue::Boolean(lhs <= rhs));
                Ok(())
            }
            (LuaValue::String(lhs), LuaValue::String(rhs)) => {
                self.data_stack.push(LuaValue::Boolean(lhs <= rhs));
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
            LuaValue::Function(LuaFunction::LuaFunc(lua_internal)) => {
                let func_id = lua_internal.function_id;
                let func_info = &chunk.functions[func_id];

                let usize_len0 = self.usize_stack.len();

                self.usize_stack.push(self.counter);
                self.usize_stack.push(self.bp);
                self.usize_stack.push(self.data_stack.len() - args_num);

                self.bp = self.local_variables.len();
                self.local_variables.resize_with(
                    self.local_variables.len() + func_info.stack_size,
                    Default::default,
                );
                let variadic = if args_num > func_info.args {
                    let variadic = if func_info.is_variadic {
                        let variadic_num = args_num - func_info.args;
                        self.data_stack
                            .drain(self.data_stack.len() - variadic_num..)
                            .collect()
                    } else {
                        self.data_stack.resize_with(
                            self.data_stack.len() - args_num + func_info.args,
                            Default::default,
                        );
                        Vec::new()
                    };
                    for (idx, arg) in self
                        .data_stack
                        .drain(self.data_stack.len() - func_info.args..)
                        .enumerate()
                    {
                        self.local_variables[self.bp + idx] = RefOrValue::Value(arg);
                    }

                    variadic
                } else {
                    for (idx, arg) in self
                        .data_stack
                        .drain(self.data_stack.len() - args_num..)
                        .enumerate()
                    {
                        self.local_variables[self.bp + idx] = RefOrValue::Value(arg);
                    }
                    Vec::new()
                };

                let func_stack = FunctionStackElem {
                    function_object: lua_internal,
                    return_expected: expected_ret,
                    variadic,
                };

                self.function_stack.push(func_stack);
                self.counter = func_info.address;
                while self.usize_stack.len() > usize_len0 {
                    let instruction = chunk.instructions.get(self.counter).unwrap();
                    self.cycle(chunk, instruction)?;
                }
                Ok(())
            }
            LuaValue::Function(LuaFunction::RustFunc(rust_internal)) => {
                let ret_num = rust_internal(self, chunk, args_num)?;
                if let Some(expected) = expected_ret {
                    let adjusted = self.data_stack.len() - ret_num + expected;
                    self.data_stack.resize_with(adjusted, Default::default);
                }
                Ok(())
            }
            other => {
                let func = other.get_metavalue("__call");
                if let Some(meta) = func {
                    self.data_stack
                        .insert(self.data_stack.len() - args_num, other);
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
                let top = self.data_stack.last().unwrap().clone();
                self.data_stack.push(top);
            }
            Instruction::Sp => {
                self.usize_stack.push(self.data_stack.len());
            }
            Instruction::Pop => {
                self.data_stack.pop();
            }
            Instruction::Deref => {
                let sp = *self.usize_stack.last().unwrap();
                let top = self.data_stack[sp].clone();
                self.data_stack.push(top);
            }
            Instruction::Jump(label) => {
                let pc = *chunk.label_map.get(*label).unwrap();
                self.counter = pc;
                return Ok(());
            }
            Instruction::JumpTrue(label) => {
                if self.data_stack.pop().unwrap().to_bool() {
                    let pc = *chunk.label_map.get(*label).unwrap();
                    self.counter = pc;
                    return Ok(());
                }
            }
            Instruction::JumpFalse(label) => {
                if !self.data_stack.pop().unwrap().to_bool() {
                    let pc = *chunk.label_map.get(*label).unwrap();
                    self.counter = pc;
                    return Ok(());
                }
            }
            Instruction::GetLocalVariable(local_id) => {
                let val = &self.local_variables[*local_id + self.bp];
                match val {
                    RefOrValue::Ref(val) => {
                        self.data_stack.push(val.borrow().clone());
                    }
                    RefOrValue::Value(val) => {
                        self.data_stack.push(val.clone());
                    }
                }
            }
            Instruction::SetLocalVariable(local_id) => {
                let top = self.data_stack.pop().unwrap();
                let val = &mut self.local_variables[*local_id + self.bp];
                match val {
                    RefOrValue::Ref(val) => {
                        val.replace(top);
                    }
                    RefOrValue::Value(val) => {
                        *val = top;
                    }
                }
            }
            Instruction::InitLocalVariable(local_id) => {
                let top = self.data_stack.pop().unwrap();
                self.local_variables[*local_id + self.bp] = RefOrValue::Value(top);
            }
            Instruction::IsNil => {
                let top = self.data_stack.pop().unwrap();
                self.data_stack.push(LuaValue::Boolean(top.is_nil()));
            }

            Instruction::Nil => {
                self.data_stack.push(LuaValue::Nil);
            }
            Instruction::Boolean(b) => {
                self.data_stack.push(LuaValue::Boolean(*b));
            }
            Instruction::Numeric(n) => match *n {
                IntOrFloat::Int(i) => {
                    self.data_stack.push(i.into());
                }
                IntOrFloat::Float(f) => {
                    self.data_stack.push(f.into());
                }
            },
            Instruction::String(s) => {
                self.data_stack.push(LuaValue::String(s.clone()));
            }
            Instruction::GetEnv => {
                self.data_stack.push(self.env.clone());
            }
            Instruction::TableInit(cap) => {
                let table = LuaTable::with_capacity(*cap);
                self.data_stack
                    .push(LuaValue::Table(Rc::new(RefCell::new(table))));
            }
            Instruction::TableIndexInit => {
                let value = self.data_stack.pop().unwrap();
                let index = self.data_stack.pop().unwrap();
                let table = self.data_stack.last_mut().unwrap();
                if let LuaValue::Table(table) = table {
                    table.borrow_mut().insert(index, value);
                } else {
                    unreachable!("table must be on top of stack");
                }
            }
            Instruction::TableInitLast(i) => {
                let sp = self.usize_stack.pop().unwrap();
                let values: Vec<_> = self.data_stack.drain(sp..).collect();
                let table = self.data_stack.last().unwrap();
                for (idx, value) in values.into_iter().enumerate() {
                    let index = idx as IntType + *i;
                    if let LuaValue::Table(table) = table {
                        table.borrow_mut().arr.insert(index, value);
                        // @TODO: use iterator and insert all at once
                    } else {
                        unreachable!("table must be on top of stack");
                    }
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
                self.data_stack.push(LuaValue::Function(func));
            }
            Instruction::FunctionInitUpvalueFromLocalVar(src_local_id) => {
                let local_var = {
                    let local_var = &mut self.local_variables[*src_local_id + self.bp];
                    // upvalue must be reference.
                    match local_var {
                        RefOrValue::Ref(r) => r.clone(),
                        RefOrValue::Value(v) => {
                            let reffed_var = Rc::new(RefCell::new(v.clone()));
                            *local_var = RefOrValue::Ref(Rc::clone(&reffed_var));
                            reffed_var
                        }
                    }
                };

                let dst_func = match self.data_stack.last_mut().unwrap() {
                    LuaValue::Function(LuaFunction::LuaFunc(f)) => f,
                    _ => unreachable!("stack top must be function"),
                };
                dst_func.upvalues.push(local_var);
            }
            Instruction::FunctionInitUpvalueFromUpvalue(src_upvalue_id) => {
                let func = self.function_stack.last().unwrap();
                let value = Rc::clone(&func.function_object.upvalues[*src_upvalue_id]);

                let dst_func = match self.data_stack.last_mut().unwrap() {
                    LuaValue::Function(LuaFunction::LuaFunc(f)) => f,
                    _ => unreachable!("stack top must be function"),
                };
                dst_func.upvalues.push(value);
            }

            Instruction::FunctionUpvalue(upvalue_id) => {
                let func = self.function_stack.last().unwrap();
                let value = RefCell::borrow(&func.function_object.upvalues[*upvalue_id]).clone();
                self.data_stack.push(value);
            }
            Instruction::FunctionUpvalueSet(upvalue_id) => {
                let rhs = self.data_stack.pop().unwrap();
                let func = self.function_stack.last().unwrap();
                let upvalue = &func.function_object.upvalues[*upvalue_id];
                *RefCell::borrow_mut(upvalue) = rhs;
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
                let top = self.data_stack.pop().unwrap().to_bool();
                self.data_stack.push((!top).into());
            }

            Instruction::FunctionCall(expected_ret) => {
                let func = self.data_stack.pop().unwrap();
                let sp = self.usize_stack.pop().unwrap();
                let num_args = self.data_stack.len() - sp;
                self.function_call(chunk, num_args, func, *expected_ret)?;
            }

            // sp -> top
            Instruction::Return => {
                if let Some(func) = self.function_stack.pop() {
                    // return from function call
                    let old_stacklen = self.usize_stack.pop().unwrap();
                    let old_bp = self.usize_stack.pop().unwrap();
                    let old_pc = self.usize_stack.pop().unwrap();
                    self.local_variables.truncate(self.bp);
                    self.bp = old_bp;
                    self.counter = old_pc;

                    if let Some(expected) = func.return_expected {
                        let adjusted = old_stacklen + expected;
                        self.data_stack.resize_with(adjusted, Default::default);
                    }
                    return Ok(());
                } else {
                    // main chunk
                    self.counter = chunk.instructions.len();
                    return Ok(());
                }
            }

            Instruction::GetVariadic(expected) => {
                let func = self.function_stack.last().unwrap();
                if let Some(expected) = expected {
                    let expected_len_after_push = self.data_stack.len() + *expected;
                    if func.variadic.len() < *expected {
                        self.data_stack.extend(func.variadic.iter().cloned());
                        self.data_stack.extend(
                            std::iter::repeat(LuaValue::Nil).take(*expected - func.variadic.len()),
                        );
                    } else {
                        self.data_stack
                            .extend(func.variadic.iter().take(*expected).cloned());
                    }
                    debug_assert!(self.data_stack.len() == expected_len_after_push);
                } else {
                    self.data_stack.extend(func.variadic.iter().cloned());
                }
            }
        }
        self.counter += 1;
        Ok(())
    }
    /// run the whole chunk
    pub fn run(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        while let Some(instruction) = chunk.instructions.get(self.counter) {
            self.cycle(chunk, instruction)?;
        }
        Ok(())
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
