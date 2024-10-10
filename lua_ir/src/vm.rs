use std::cell::RefCell;
use std::rc::Rc;

use lua_tokenizer::IntOrFloat;
use lua_tokenizer::IntType;

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
            local_variables,
            data_stack: Vec::new(),
            usize_stack: Vec::new(),
            function_stack: Vec::new(),
            counter: 0,
            bp: 0,
        }
    }

    pub fn adjust_multire(&mut self, size: usize) {
        let sp = self.usize_stack.pop().unwrap();
        let resize_to = sp + size;
        self.data_stack.resize_with(resize_to, || LuaValue::Nil);
    }
    pub fn pop(&mut self) -> Option<LuaValue> {
        self.data_stack.pop()
    }
    pub fn push(&mut self, value: LuaValue) {
        self.data_stack.push(value);
    }
    pub fn pop_multire(&mut self) -> impl Iterator<Item = LuaValue> + '_ {
        let sp = self.usize_stack.pop().unwrap();
        self.data_stack.drain(sp..)
    }

    pub fn function_call(
        &mut self,
        chunk: &Chunk,
        mut args: Vec<LuaValue>,
        func: LuaFunction,
        expected_ret: Option<usize>,
    ) -> Result<(), RuntimeError> {
        match func {
            LuaFunction::LuaFunc(lua_internal) => {
                let func_id = lua_internal.function_id;
                let func_info = &chunk.functions[func_id];
                let variadic = if args.len() < func_info.args {
                    args.resize_with(func_info.args, Default::default);
                    Vec::new()
                } else {
                    if func_info.is_variadic {
                        args.split_off(func_info.args)
                    } else {
                        args.truncate(func_info.args);
                        Vec::new()
                    }
                };

                let func_stack = FunctionStackElem {
                    function_object: lua_internal.clone(),
                    return_expected: expected_ret,
                    variadic,
                };

                let usize_len0 = self.usize_stack.len();

                self.usize_stack.push(self.counter + 1);
                self.usize_stack.push(self.bp);
                self.usize_stack.push(self.data_stack.len());
                self.bp = self.local_variables.len();
                self.local_variables.resize_with(
                    self.local_variables.len() + func_info.stack_size,
                    Default::default,
                );
                for (idx, arg) in args.into_iter().enumerate() {
                    match &mut self.local_variables[self.bp + idx] {
                        RefOrValue::Ref(r) => {
                            *r.borrow_mut() = arg;
                        }
                        RefOrValue::Value(v) => {
                            *v = arg;
                        }
                    }
                }

                self.function_stack.push(func_stack);
                self.counter = func_info.address;
                while self.usize_stack.len() > usize_len0 {
                    self.cycle(chunk)?;
                }
            }
            LuaFunction::RustFunc(rust_internal) => {
                let mut ret = rust_internal(self, args)?;
                if let Some(expected) = expected_ret {
                    ret.resize_with(expected, Default::default);
                }
                for v in ret.into_iter() {
                    self.data_stack.push(v);
                }
            }
        }
        Ok(())
    }

    pub fn cycle(&mut self, chunk: &Chunk) -> Result<bool, RuntimeError> {
        match chunk.instructions.get(self.counter) {
            Some(instruction) => {
                match instruction {
                    Instruction::Clear(local_id) => {
                        self.local_variables[*local_id + self.bp] =
                            RefOrValue::Value(LuaValue::Nil);
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
                    Instruction::Jump(label) => {
                        let pc = *chunk.label_map.get(*label).unwrap();
                        self.counter = pc;
                        return Ok(false);
                    }
                    Instruction::JumpTrue(label) => {
                        if self.data_stack.pop().unwrap().to_bool() {
                            let pc = *chunk.label_map.get(*label).unwrap();
                            self.counter = pc;
                            return Ok(false);
                        }
                    }
                    Instruction::JumpFalse(label) => {
                        if !self.data_stack.pop().unwrap().to_bool() {
                            let pc = *chunk.label_map.get(*label).unwrap();
                            self.counter = pc;
                            return Ok(false);
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
                    Instruction::Numeric(n) => match n {
                        IntOrFloat::Int(i) => {
                            self.data_stack.push(LuaValue::Int(*i));
                        }
                        IntOrFloat::Float(f) => {
                            self.data_stack.push(LuaValue::Float(*f));
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
                                table.borrow_mut().map.insert(LuaValue::Int(index), value);
                            } else {
                                unreachable!("table must be on top of stack");
                            }
                        }
                    }

                    Instruction::TableIndex => {
                        let index = self.data_stack.pop().unwrap();
                        let table = self.data_stack.pop().unwrap();
                        if let LuaValue::Table(table) = table {
                            let get = table
                                .borrow()
                                .map
                                .get(&index)
                                .cloned()
                                .unwrap_or(LuaValue::Nil);
                            self.data_stack.push(get);
                        } else {
                            unreachable!("table_index: table must be on top of stack");
                        }
                    }
                    Instruction::TableIndexSet => {
                        let index = self.data_stack.pop().unwrap();
                        let table = self.data_stack.pop().unwrap();
                        let value = self.data_stack.pop().unwrap();
                        if let LuaValue::Table(table) = table {
                            if !index.is_nil() {
                                table.borrow_mut().map.insert(index, value);
                            }
                        } else {
                            unreachable!("table_index_set: table must be on top of stack");
                        }
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
                        let value =
                            RefCell::borrow(&func.function_object.upvalues[*upvalue_id]).clone();
                        self.data_stack.push(value);
                    }
                    Instruction::FunctionUpvalueSet(upvalue_id) => {
                        let rhs = self.data_stack.pop().unwrap();
                        let func = self.function_stack.last().unwrap();
                        let upvalue = &func.function_object.upvalues[*upvalue_id];
                        *RefCell::borrow_mut(upvalue) = rhs;
                    }

                    Instruction::BinaryAdd => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = lhs.add(&rhs)?;
                        self.data_stack.push(ret);
                    }
                    Instruction::BinarySub => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = lhs.sub(&rhs)?;
                        self.data_stack.push(ret);
                    }
                    Instruction::BinaryMul => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = lhs.mul(&rhs)?;
                        self.data_stack.push(ret);
                    }
                    Instruction::BinaryDiv => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = lhs.div(&rhs)?;
                        self.data_stack.push(ret);
                    }
                    Instruction::BinaryFloorDiv => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = lhs.fdiv(&rhs)?;
                        self.data_stack.push(ret);
                    }
                    Instruction::BinaryMod => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = lhs.modu(&rhs)?;
                        self.data_stack.push(ret);
                    }
                    Instruction::BinaryPow => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = lhs.pow(&rhs)?;
                        self.data_stack.push(ret);
                    }
                    Instruction::BinaryConcat => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = lhs.concat(&rhs)?;
                        self.data_stack.push(ret);
                    }
                    Instruction::BinaryBitwiseAnd => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = lhs.bit_and(&rhs)?;
                        self.data_stack.push(ret);
                    }
                    Instruction::BinaryBitwiseOr => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = lhs.bit_or(&rhs)?;
                        self.data_stack.push(ret);
                    }
                    Instruction::BinaryBitwiseXor => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = lhs.bit_xor(&rhs)?;
                        self.data_stack.push(ret);
                    }
                    Instruction::BinaryShiftLeft => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = lhs.bit_lshift(&rhs)?;
                        self.data_stack.push(ret);
                    }
                    Instruction::BinaryShiftRight => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = lhs.bit_rshift(&rhs)?;
                        self.data_stack.push(ret);
                    }
                    Instruction::BinaryEqual => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = lhs.eq(&rhs);
                        self.data_stack.push(ret.into());
                    }
                    Instruction::BinaryNotEqual => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = !lhs.eq(&rhs);
                        self.data_stack.push(ret.into());
                    }
                    Instruction::BinaryLessThan => {
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = lhs.lt(&rhs)?;
                        self.data_stack.push(ret.into());
                    }
                    Instruction::BinaryLessEqual => {
                        // a <= b <=> !(a > b)
                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = !rhs.lt(&lhs)?;
                        self.data_stack.push(ret.into());
                    }
                    Instruction::BinaryGreaterThan => {
                        // a > b <=> b < a

                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = rhs.lt(&lhs)?;
                        self.data_stack.push(ret.into());
                    }
                    Instruction::BinaryGreaterEqual => {
                        // a >= b <=> !(a < b)

                        let rhs = self.data_stack.pop().unwrap();
                        let lhs = self.data_stack.pop().unwrap();
                        let ret = !lhs.lt(&rhs)?;
                        self.data_stack.push(ret.into());
                    }

                    Instruction::UnaryMinus => {
                        let top = self.data_stack.pop().unwrap();
                        let ret = top.neg()?;
                        self.data_stack.push(ret.into());
                    }
                    Instruction::UnaryBitwiseNot => {
                        let top = self.data_stack.pop().unwrap();
                        let ret = top.bit_not()?;
                        self.data_stack.push(ret.into());
                    }
                    Instruction::UnaryLength => {
                        let top = self.data_stack.pop().unwrap();
                        let ret = top.len()?;
                        self.data_stack.push(ret.into());
                    }
                    Instruction::UnaryLogicalNot => {
                        let top = self.data_stack.pop().unwrap();
                        let ret = top.not();
                        self.data_stack.push(ret.into());
                    }

                    // expected can be 0
                    Instruction::FunctionCall(expected_ret) => {
                        let sp = self.usize_stack.pop().unwrap();
                        let mut args: Vec<_> = self.data_stack.drain(sp + 1..).collect();
                        let top = self.data_stack.pop().unwrap();
                        let func = match top {
                            LuaValue::Function(f) => f,
                            _ => {
                                // @TODO check metatable
                                return Err(RuntimeError::FunctionCallOnNonFunction);
                            }
                        };
                        match func {
                            LuaFunction::LuaFunc(lua_internal) => {
                                let func_id = lua_internal.function_id;
                                let func_info = &chunk.functions[func_id];
                                let variadic = if args.len() < func_info.args {
                                    args.resize_with(func_info.args, Default::default);
                                    Vec::new()
                                } else {
                                    if func_info.is_variadic {
                                        args.split_off(func_info.args)
                                    } else {
                                        args.truncate(func_info.args);
                                        Vec::new()
                                    }
                                };

                                let func_stack = FunctionStackElem {
                                    function_object: lua_internal.clone(),
                                    return_expected: *expected_ret,
                                    variadic,
                                };

                                self.usize_stack.push(self.counter + 1);
                                self.usize_stack.push(self.bp);
                                self.usize_stack.push(self.data_stack.len());
                                self.bp = self.local_variables.len();
                                self.local_variables.resize_with(
                                    self.local_variables.len() + func_info.stack_size,
                                    Default::default,
                                );
                                for (idx, arg) in args.into_iter().enumerate() {
                                    match &mut self.local_variables[self.bp + idx] {
                                        RefOrValue::Ref(r) => {
                                            *r.borrow_mut() = arg;
                                        }
                                        RefOrValue::Value(v) => {
                                            *v = arg;
                                        }
                                    }
                                }

                                self.function_stack.push(func_stack);
                                self.counter = func_info.address;
                                return Ok(false);
                            }
                            LuaFunction::RustFunc(rust_internal) => {
                                let mut ret = rust_internal(self, args)?;
                                if let Some(expected) = expected_ret {
                                    ret.resize_with(*expected, Default::default);
                                }
                                for v in ret.into_iter() {
                                    self.data_stack.push(v);
                                }
                            }
                        }
                    }

                    Instruction::Return => {
                        let sp = self.usize_stack.pop().unwrap();
                        let mut returned_values = self.data_stack.drain(sp..).collect::<Vec<_>>();
                        if let Some(func) = self.function_stack.pop() {
                            // function call

                            let old_stacklen = self.usize_stack.pop().unwrap();
                            let old_bp = self.usize_stack.pop().unwrap();
                            let old_pc = self.usize_stack.pop().unwrap();
                            self.local_variables.truncate(self.bp);
                            self.bp = old_bp;
                            self.counter = old_pc;
                            self.data_stack.truncate(old_stacklen);

                            if let Some(expected) = func.return_expected {
                                returned_values.resize_with(expected, || LuaValue::Nil);
                            }
                            self.data_stack.append(&mut returned_values);
                            return Ok(false);
                        } else {
                            // main chunk
                            self.counter = chunk.instructions.len();
                            return Ok(false);
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

                    Instruction::AdjustMultire(size) => {
                        self.adjust_multire(*size);
                    }
                }
                self.counter += 1;
                Ok(self.counter >= chunk.instructions.len())
            }
            None => {
                return Ok(true);
            }
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
