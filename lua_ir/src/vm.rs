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
    pub fn push(&mut self, value: LuaValue) {
        self.data_stack.push(value);
    }
    pub fn pop_multire(&mut self) -> impl Iterator<Item = LuaValue> + '_ {
        let sp = self.usize_stack.pop().unwrap();
        self.data_stack.drain(sp..)
    }

    pub fn add(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push((lhs + rhs).into());
                Ok(())
            }
            (lhs, rhs) => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__add");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(rhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        match rhs {
                            LuaValue::Table(rhs) => {
                                let func = rhs.borrow().get_metamethod("__add");
                                if let Some(meta) = func {
                                    self.data_stack.push(LuaValue::Table(lhs));
                                    self.data_stack.push(LuaValue::Table(rhs));
                                    self.function_call(chunk, 2, meta, Some(1))
                                } else {
                                    Err(RuntimeError::NoMetaMethod)
                                }
                            }
                            _ => Err(RuntimeError::NoMetaMethod),
                        }
                    }
                }
                lhs => match rhs {
                    LuaValue::Table(rhs) => {
                        let func = rhs.borrow().get_metamethod("__add");
                        if let Some(meta) = func {
                            self.data_stack.push(lhs);
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
        }
    }

    pub fn sub(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push((lhs - rhs).into());
                Ok(())
            }
            (lhs, rhs) => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__sub");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(rhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        match rhs {
                            LuaValue::Table(rhs) => {
                                let func = rhs.borrow().get_metamethod("__sub");
                                if let Some(meta) = func {
                                    self.data_stack.push(LuaValue::Table(lhs));
                                    self.data_stack.push(LuaValue::Table(rhs));
                                    self.function_call(chunk, 2, meta, Some(1))
                                } else {
                                    Err(RuntimeError::NoMetaMethod)
                                }
                            }
                            _ => Err(RuntimeError::NoMetaMethod),
                        }
                    }
                }
                lhs => match rhs {
                    LuaValue::Table(rhs) => {
                        let func = rhs.borrow().get_metamethod("__sub");
                        if let Some(meta) = func {
                            self.data_stack.push(lhs);
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
        }
    }
    pub fn mul(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push((lhs * rhs).into());
                Ok(())
            }
            (lhs, rhs) => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__mul");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(rhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        match rhs {
                            LuaValue::Table(rhs) => {
                                let func = rhs.borrow().get_metamethod("__mul");
                                if let Some(meta) = func {
                                    self.data_stack.push(LuaValue::Table(lhs));
                                    self.data_stack.push(LuaValue::Table(rhs));
                                    self.function_call(chunk, 2, meta, Some(1))
                                } else {
                                    Err(RuntimeError::NoMetaMethod)
                                }
                            }
                            _ => Err(RuntimeError::NoMetaMethod),
                        }
                    }
                }
                lhs => match rhs {
                    LuaValue::Table(rhs) => {
                        let func = rhs.borrow().get_metamethod("__mul");
                        if let Some(meta) = func {
                            self.data_stack.push(lhs);
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
        }
    }
    pub fn div(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push((lhs / rhs).into());
                Ok(())
            }
            (lhs, rhs) => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__div");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(rhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        match rhs {
                            LuaValue::Table(rhs) => {
                                let func = rhs.borrow().get_metamethod("__div");
                                if let Some(meta) = func {
                                    self.data_stack.push(LuaValue::Table(lhs));
                                    self.data_stack.push(LuaValue::Table(rhs));
                                    self.function_call(chunk, 2, meta, Some(1))
                                } else {
                                    Err(RuntimeError::NoMetaMethod)
                                }
                            }
                            _ => Err(RuntimeError::NoMetaMethod),
                        }
                    }
                }
                lhs => match rhs {
                    LuaValue::Table(rhs) => {
                        let func = rhs.borrow().get_metamethod("__div");
                        if let Some(meta) = func {
                            self.data_stack.push(lhs);
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
        }
    }
    pub fn mod_(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push((lhs % rhs).into());
                Ok(())
            }
            (lhs, rhs) => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__mod");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(rhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        match rhs {
                            LuaValue::Table(rhs) => {
                                let func = rhs.borrow().get_metamethod("__mod");
                                if let Some(meta) = func {
                                    self.data_stack.push(LuaValue::Table(lhs));
                                    self.data_stack.push(LuaValue::Table(rhs));
                                    self.function_call(chunk, 2, meta, Some(1))
                                } else {
                                    Err(RuntimeError::NoMetaMethod)
                                }
                            }
                            _ => Err(RuntimeError::NoMetaMethod),
                        }
                    }
                }
                lhs => match rhs {
                    LuaValue::Table(rhs) => {
                        let func = rhs.borrow().get_metamethod("__mod");
                        if let Some(meta) = func {
                            self.data_stack.push(lhs);
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
        }
    }
    pub fn pow(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push(lhs.pow(rhs).into());
                Ok(())
            }
            (lhs, rhs) => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__pow");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(rhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        match rhs {
                            LuaValue::Table(rhs) => {
                                let func = rhs.borrow().get_metamethod("__pow");
                                if let Some(meta) = func {
                                    self.data_stack.push(LuaValue::Table(lhs));
                                    self.data_stack.push(LuaValue::Table(rhs));
                                    self.function_call(chunk, 2, meta, Some(1))
                                } else {
                                    Err(RuntimeError::NoMetaMethod)
                                }
                            }
                            _ => Err(RuntimeError::NoMetaMethod),
                        }
                    }
                }
                lhs => match rhs {
                    LuaValue::Table(rhs) => {
                        let func = rhs.borrow().get_metamethod("__pow");
                        if let Some(meta) = func {
                            self.data_stack.push(lhs);
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
        }
    }
    pub fn unm(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let lhs = self.data_stack.pop().unwrap();
        match lhs {
            LuaValue::Number(num) => {
                self.data_stack.push((-num).into());
                Ok(())
            }
            lhs => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__unm");
                    if let Some(meta) = func {
                        // For the unary operators (negation, length, and bitwise NOT),
                        // the metamethod is computed and called with a dummy second operand
                        // equal to the first one.
                        self.data_stack.push(LuaValue::Table(Rc::clone(&lhs)));
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        Err(RuntimeError::NoMetaMethod)
                    }
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    pub fn idiv(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.data_stack.push(lhs.floor_div(rhs).into());
                Ok(())
            }
            (lhs, rhs) => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__idiv");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(rhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        match rhs {
                            LuaValue::Table(rhs) => {
                                let func = rhs.borrow().get_metamethod("__idiv");
                                if let Some(meta) = func {
                                    self.data_stack.push(LuaValue::Table(lhs));
                                    self.data_stack.push(LuaValue::Table(rhs));
                                    self.function_call(chunk, 2, meta, Some(1))
                                } else {
                                    Err(RuntimeError::NoMetaMethod)
                                }
                            }
                            _ => Err(RuntimeError::NoMetaMethod),
                        }
                    }
                }
                lhs => match rhs {
                    LuaValue::Table(rhs) => {
                        let func = rhs.borrow().get_metamethod("__idiv");
                        if let Some(meta) = func {
                            self.data_stack.push(lhs);
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
        }
    }
    pub fn band(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs.try_to_int(), rhs.try_to_int()) {
            (Some(lhs), Some(rhs)) => {
                self.data_stack.push((lhs & rhs).into());
                Ok(())
            }
            _ => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__band");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(rhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        match rhs {
                            LuaValue::Table(rhs) => {
                                let func = rhs.borrow().get_metamethod("__band");
                                if let Some(meta) = func {
                                    self.data_stack.push(LuaValue::Table(lhs));
                                    self.data_stack.push(LuaValue::Table(rhs));
                                    self.function_call(chunk, 2, meta, Some(1))
                                } else {
                                    Err(RuntimeError::NoMetaMethod)
                                }
                            }
                            _ => Err(RuntimeError::NoMetaMethod),
                        }
                    }
                }
                lhs => match rhs {
                    LuaValue::Table(rhs) => {
                        let func = rhs.borrow().get_metamethod("__band");
                        if let Some(meta) = func {
                            self.data_stack.push(lhs);
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
        }
    }
    pub fn bor(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs.try_to_int(), rhs.try_to_int()) {
            (Some(lhs), Some(rhs)) => {
                self.data_stack.push((lhs | rhs).into());
                Ok(())
            }
            _ => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__bor");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(rhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        match rhs {
                            LuaValue::Table(rhs) => {
                                let func = rhs.borrow().get_metamethod("__bor");
                                if let Some(meta) = func {
                                    self.data_stack.push(LuaValue::Table(lhs));
                                    self.data_stack.push(LuaValue::Table(rhs));
                                    self.function_call(chunk, 2, meta, Some(1))
                                } else {
                                    Err(RuntimeError::NoMetaMethod)
                                }
                            }
                            _ => Err(RuntimeError::NoMetaMethod),
                        }
                    }
                }
                lhs => match rhs {
                    LuaValue::Table(rhs) => {
                        let func = rhs.borrow().get_metamethod("__bor");
                        if let Some(meta) = func {
                            self.data_stack.push(lhs);
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
        }
    }
    pub fn bxor(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs.try_to_int(), rhs.try_to_int()) {
            (Some(lhs), Some(rhs)) => {
                self.data_stack.push((lhs ^ rhs).into());
                Ok(())
            }
            _ => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__bxor");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(rhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        match rhs {
                            LuaValue::Table(rhs) => {
                                let func = rhs.borrow().get_metamethod("__bxor");
                                if let Some(meta) = func {
                                    self.data_stack.push(LuaValue::Table(lhs));
                                    self.data_stack.push(LuaValue::Table(rhs));
                                    self.function_call(chunk, 2, meta, Some(1))
                                } else {
                                    Err(RuntimeError::NoMetaMethod)
                                }
                            }
                            _ => Err(RuntimeError::NoMetaMethod),
                        }
                    }
                }
                lhs => match rhs {
                    LuaValue::Table(rhs) => {
                        let func = rhs.borrow().get_metamethod("__bxor");
                        if let Some(meta) = func {
                            self.data_stack.push(lhs);
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
        }
    }
    pub fn shl(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs.try_to_int(), rhs.try_to_int()) {
            (Some(lhs), Some(rhs)) => {
                self.data_stack.push((lhs << rhs).into());
                Ok(())
            }
            _ => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__shl");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(rhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        match rhs {
                            LuaValue::Table(rhs) => {
                                let func = rhs.borrow().get_metamethod("__shl");
                                if let Some(meta) = func {
                                    self.data_stack.push(LuaValue::Table(lhs));
                                    self.data_stack.push(LuaValue::Table(rhs));
                                    self.function_call(chunk, 2, meta, Some(1))
                                } else {
                                    Err(RuntimeError::NoMetaMethod)
                                }
                            }
                            _ => Err(RuntimeError::NoMetaMethod),
                        }
                    }
                }
                lhs => match rhs {
                    LuaValue::Table(rhs) => {
                        let func = rhs.borrow().get_metamethod("__shl");
                        if let Some(meta) = func {
                            self.data_stack.push(lhs);
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
        }
    }
    pub fn shr(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs.try_to_int(), rhs.try_to_int()) {
            (Some(lhs), Some(rhs)) => {
                self.data_stack.push((lhs >> rhs).into());
                Ok(())
            }
            _ => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__shr");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(rhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        match rhs {
                            LuaValue::Table(rhs) => {
                                let func = rhs.borrow().get_metamethod("__shr");
                                if let Some(meta) = func {
                                    self.data_stack.push(LuaValue::Table(lhs));
                                    self.data_stack.push(LuaValue::Table(rhs));
                                    self.function_call(chunk, 2, meta, Some(1))
                                } else {
                                    Err(RuntimeError::NoMetaMethod)
                                }
                            }
                            _ => Err(RuntimeError::NoMetaMethod),
                        }
                    }
                }
                lhs => match rhs {
                    LuaValue::Table(rhs) => {
                        let func = rhs.borrow().get_metamethod("__shr");
                        if let Some(meta) = func {
                            self.data_stack.push(lhs);
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
        }
    }
    pub fn bnot(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let lhs = self.data_stack.pop().unwrap();
        match lhs.try_to_int() {
            Some(i) => {
                self.data_stack.push((!i).into());
                Ok(())
            }
            _ => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__bnot");
                    if let Some(meta) = func {
                        // For the unary operators (negation, length, and bitwise NOT),
                        // the metamethod is computed and called with a dummy second operand
                        // equal to the first one.
                        self.data_stack.push(LuaValue::Table(Rc::clone(&lhs)));
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        Err(RuntimeError::NoMetaMethod)
                    }
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    pub fn concat(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();
        match (lhs.try_to_string(), rhs.try_to_string()) {
            (Some(mut lhs), Some(mut rhs)) => {
                lhs.append(&mut rhs);
                self.data_stack.push(LuaValue::String(lhs));
                Ok(())
            }
            _ => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__concat");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(rhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        match rhs {
                            LuaValue::Table(rhs) => {
                                let func = rhs.borrow().get_metamethod("__concat");
                                if let Some(meta) = func {
                                    self.data_stack.push(LuaValue::Table(lhs));
                                    self.data_stack.push(LuaValue::Table(rhs));
                                    self.function_call(chunk, 2, meta, Some(1))
                                } else {
                                    Err(RuntimeError::NoMetaMethod)
                                }
                            }
                            _ => Err(RuntimeError::NoMetaMethod),
                        }
                    }
                }
                lhs => match rhs {
                    LuaValue::Table(rhs) => {
                        let func = rhs.borrow().get_metamethod("__concat");
                        if let Some(meta) = func {
                            self.data_stack.push(lhs);
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
        }
    }
    pub fn len(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let lhs = self.data_stack.pop().unwrap();
        match lhs {
            LuaValue::String(s) => {
                self.data_stack.push((s.len() as IntType).into());
                Ok(())
            }
            LuaValue::Table(table) => {
                let func = table.borrow().get_metamethod("__len");
                if let Some(meta) = func {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    self.data_stack.push(LuaValue::Table(Rc::clone(&table)));
                    self.data_stack.push(LuaValue::Table(table));
                    self.function_call(chunk, 1, meta, Some(1))
                } else {
                    self.data_stack
                        .push((table.borrow().len()? as IntType).into());
                    Ok(())
                }
            }
            _ => Err(RuntimeError::NoMetaMethod),
        }
    }
    pub fn index(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let key = self.data_stack.pop().unwrap();
        let table = self.data_stack.pop().unwrap();
        match table {
            LuaValue::Table(table) => {
                let get = table.borrow().map.get(&key).cloned();
                if let Some(get) = get {
                    self.data_stack.push(get);
                    Ok(())
                } else {
                    let meta = table.borrow().get_metamethod("__index");
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
            _ => Err(RuntimeError::NotTable),
        }
    }
    pub fn newindex(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let key = self.data_stack.pop().unwrap();
        let table = self.data_stack.pop().unwrap();
        let value = self.data_stack.pop().unwrap();

        match table {
            LuaValue::Table(table) => {
                if let Some(val) = table.borrow_mut().map.get_mut(&key) {
                    *val = value;
                    return Ok(());
                }
                let meta = table.borrow().get_metamethod("__index");

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
                        table.borrow_mut().map.insert(key, value);
                        Ok(())
                    }
                }
            }
            _ => Err(RuntimeError::NotTable),
        }
    }
    pub fn eq(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();

        match (lhs, rhs) {
            (LuaValue::Table(lhs), LuaValue::Table(rhs)) => {
                if Rc::ptr_eq(&lhs, &rhs) {
                    self.data_stack.push(LuaValue::Boolean(true));
                    return Ok(());
                } else {
                    let func = lhs.borrow().get_metamethod("__eq");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(LuaValue::Table(rhs));
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        let func = rhs.borrow().get_metamethod("__eq");
                        if let Some(meta) = func {
                            self.data_stack.push(LuaValue::Table(lhs));
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                }
            }
            (lhs, rhs) => {
                self.data_stack.push(LuaValue::Boolean(lhs == rhs));
                Ok(())
            }
        }
    }
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
            (lhs, rhs) => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__lt");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(rhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        match rhs {
                            LuaValue::Table(rhs) => {
                                let func = rhs.borrow().get_metamethod("__lt");
                                if let Some(meta) = func {
                                    self.data_stack.push(LuaValue::Table(lhs));
                                    self.data_stack.push(LuaValue::Table(rhs));
                                    self.function_call(chunk, 2, meta, Some(1))
                                } else {
                                    Err(RuntimeError::NoMetaMethod)
                                }
                            }
                            _ => Err(RuntimeError::NoMetaMethod),
                        }
                    }
                }
                lhs => match rhs {
                    LuaValue::Table(rhs) => {
                        let func = rhs.borrow().get_metamethod("__lt");
                        if let Some(meta) = func {
                            self.data_stack.push(lhs);
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
        }
    }

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
            (lhs, rhs) => match lhs {
                LuaValue::Table(lhs) => {
                    let func = lhs.borrow().get_metamethod("__le");
                    if let Some(meta) = func {
                        self.data_stack.push(LuaValue::Table(lhs));
                        self.data_stack.push(rhs);
                        self.function_call(chunk, 2, meta, Some(1))
                    } else {
                        match rhs {
                            LuaValue::Table(rhs) => {
                                let func = rhs.borrow().get_metamethod("__le");
                                if let Some(meta) = func {
                                    self.data_stack.push(LuaValue::Table(lhs));
                                    self.data_stack.push(LuaValue::Table(rhs));
                                    self.function_call(chunk, 2, meta, Some(1))
                                } else {
                                    Err(RuntimeError::NoMetaMethod)
                                }
                            }
                            _ => Err(RuntimeError::NoMetaMethod),
                        }
                    }
                }
                lhs => match rhs {
                    LuaValue::Table(rhs) => {
                        let func = rhs.borrow().get_metamethod("__le");
                        if let Some(meta) = func {
                            self.data_stack.push(lhs);
                            self.data_stack.push(LuaValue::Table(rhs));
                            self.function_call(chunk, 2, meta, Some(1))
                        } else {
                            Err(RuntimeError::NoMetaMethod)
                        }
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
        }
    }

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
            }
            LuaValue::Function(LuaFunction::RustFunc(rust_internal)) => {
                let ret_num = rust_internal(self, chunk, args_num)?;
                if let Some(expected) = expected_ret {
                    let adjusted = self.data_stack.len() - ret_num + expected;
                    self.data_stack.resize_with(adjusted, Default::default);
                }
            }
            LuaValue::Table(table) => {
                let func = table.borrow().get_metamethod("__call");
                if let Some(meta) = func {
                    self.data_stack
                        .insert(self.data_stack.len() - args_num, LuaValue::Table(table));
                    self.function_call(chunk, args_num + 1, meta, expected_ret)?;
                } else {
                    return Err(RuntimeError::NotFunction);
                }
            }
            _ => {
                return Err(RuntimeError::NotFunction);
            }
        }
        Ok(())
    }
    pub fn cycle(&mut self, chunk: &Chunk, instruction: &Instruction) -> Result<(), RuntimeError> {
        match instruction {
            Instruction::Clear(local_id) => {
                self.local_variables[*local_id + self.bp] = RefOrValue::Value(LuaValue::Nil);
            }
            Instruction::Clone => {
                let top = self.data_stack.last().unwrap().clone();
                self.data_stack.push(top);
            }
            Instruction::Swap => {
                let mut top = self.data_stack.pop().unwrap();
                std::mem::swap(self.data_stack.last_mut().unwrap(), &mut top);
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
            Instruction::GetGlobal => {
                // @TODO _G? or _ENV?
                self.data_stack.push(self.env.clone());
            }
            Instruction::GetEnv => {
                // @TODO _G? or _ENV?
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
                    table.borrow_mut().map.insert(index, value);
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
                        table.borrow_mut().map.insert(index.into(), value);
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
            Instruction::FunctionUpvaluePushFromLocalVar(src_local_id) => {
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
            Instruction::FunctionUpvaluePushFromUpvalue(src_upvalue_id) => {
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
            Instruction::BinaryNotEqual => {
                self.eq(chunk)?;
                match &mut self.data_stack.last_mut().unwrap() {
                    LuaValue::Boolean(b) => {
                        *b = !*b;
                    }
                    _ => unreachable!("eq must return boolean"),
                }
            }
            Instruction::BinaryLessThan => {
                self.lt(chunk)?;
            }
            Instruction::BinaryLessEqual => {
                self.le(chunk)?;
            }
            Instruction::BinaryGreaterThan => {
                // a > b <=> b < a
                let rhs = self.data_stack.pop().unwrap();
                let lhs = self.data_stack.pop().unwrap();
                self.data_stack.push(rhs);
                self.data_stack.push(lhs);
                self.lt(chunk)?;
            }
            Instruction::BinaryGreaterEqual => {
                // a >= b <=> b <= a
                let rhs = self.data_stack.pop().unwrap();
                let lhs = self.data_stack.pop().unwrap();
                self.data_stack.push(rhs);
                self.data_stack.push(lhs);
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
                let top = self.data_stack.pop().unwrap();
                let ret = top.not();
                self.data_stack.push(ret.into());
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
                    if func.variadic.len() < *expected {
                        for v in func.variadic.iter() {
                            self.data_stack.push(v.clone());
                        }
                        for _ in func.variadic.len()..*expected {
                            self.data_stack.push(LuaValue::Nil);
                        }
                    } else {
                        for v in func.variadic.iter().take(*expected) {
                            self.data_stack.push(v.clone());
                        }
                    }
                } else {
                    for v in func.variadic.iter() {
                        self.data_stack.push(v.clone());
                    }
                }
            }
        }
        self.counter += 1;
        Ok(())
    }
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
