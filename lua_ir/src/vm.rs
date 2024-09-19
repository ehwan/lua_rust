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
    /// _G
    pub global: LuaValue,
    pub local_variables: Vec<RefOrValue>,
    /// offset of local variables
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
    /// get reference of i'th local variable
    pub fn local_variable(&self, i: usize) -> &RefOrValue {
        &self.local_variables[self.bp + i]
    }
    /// get mutable reference of i'th local variable
    pub fn local_variable_mut(&mut self, i: usize) -> &mut RefOrValue {
        &mut self.local_variables[self.bp + i]
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub label_map: Vec<usize>,
    pub functions: Vec<FunctionInfo>,
    pub stack_size: usize,
}
impl Program {
    pub fn new_stack(&self) -> Stack {
        let mut local_variables = Vec::new();
        local_variables.resize_with(self.stack_size, || RefOrValue::Value(LuaValue::Nil));
        let env = builtin::init_env().unwrap();
        Stack {
            global: LuaValue::Table(Rc::new(RefCell::new(env))),
            local_variables,
            data_stack: Vec::new(),
            usize_stack: Vec::new(),
            function_stack: Vec::new(),
            counter: 0,
            bp: 0,
        }
    }
    pub fn cycle(&self, stack: &mut Stack) -> Result<bool, RuntimeError> {
        if stack.counter >= self.instructions.len() {
            return Ok(true);
        }
        let instruction = &self.instructions[stack.counter];
        match instruction {
            Instruction::Clear(local_id) => {
                *stack.local_variable_mut(*local_id) = RefOrValue::Value(LuaValue::Nil);
            }
            Instruction::Clone => {
                let top = stack.data_stack.last().unwrap().clone();
                stack.data_stack.push(top);
            }
            Instruction::Swap => {
                let mut top = stack.data_stack.pop().unwrap();
                std::mem::swap(stack.data_stack.last_mut().unwrap(), &mut top);
                stack.data_stack.push(top);
            }
            Instruction::Sp => {
                stack.usize_stack.push(stack.data_stack.len());
            }
            Instruction::Pop => {
                stack.data_stack.pop();
            }
            Instruction::Jump(label) => {
                let pc = *self.label_map.get(*label).unwrap();
                stack.counter = pc;
                return Ok(false);
            }
            Instruction::JumpTrue(label) => {
                if stack.data_stack.pop().unwrap().to_bool() {
                    let pc = *self.label_map.get(*label).unwrap();
                    stack.counter = pc;
                    return Ok(false);
                }
            }
            Instruction::JumpFalse(label) => {
                if !stack.data_stack.pop().unwrap().to_bool() {
                    let pc = *self.label_map.get(*label).unwrap();
                    stack.counter = pc;
                    return Ok(false);
                }
            }
            Instruction::GetLocalVariable(local_id) => {
                let value = stack.local_variable(*local_id).clone().into();
                stack.data_stack.push(value);
            }
            Instruction::SetLocalVariable(local_id) => {
                let top = stack.data_stack.pop().unwrap();
                stack.local_variable_mut(*local_id).set(top);
            }
            Instruction::Nil => {
                stack.data_stack.push(LuaValue::Nil);
            }
            Instruction::Boolean(b) => {
                stack.data_stack.push(LuaValue::Boolean(*b));
            }
            Instruction::Numeric(n) => match n {
                IntOrFloat::Int(i) => {
                    stack.data_stack.push(LuaValue::Int(*i));
                }
                IntOrFloat::Float(f) => {
                    stack.data_stack.push(LuaValue::Float(*f));
                }
            },
            Instruction::String(s) => {
                stack.data_stack.push(LuaValue::String(s.clone()));
            }
            Instruction::GetGlobal => {
                stack.data_stack.push(stack.global.clone());
            }
            Instruction::GetEnv => {
                stack.data_stack.push(stack.global.clone());
            }
            Instruction::TableInit(cap) => {
                let table = LuaTable::with_capacity(*cap);
                stack
                    .data_stack
                    .push(LuaValue::Table(Rc::new(RefCell::new(table))));
            }
            Instruction::TableIndexInit => {
                let value = stack.data_stack.pop().unwrap();
                let index = stack.data_stack.pop().unwrap();
                let table = stack.data_stack.last_mut().unwrap();
                table.table_index_init(index, value)?;
            }
            Instruction::TableInitLast(i) => {
                let sp = stack.usize_stack.pop().unwrap();
                let values: Vec<_> = stack.data_stack.drain(sp..).collect();
                for (idx, value) in values.into_iter().enumerate() {
                    let index = idx as IntType + *i;
                    let table = stack.data_stack.last_mut().unwrap();
                    table.table_index_init(LuaValue::Int(index), value)?;
                }
            }

            Instruction::TableIndex => {
                let index = stack.data_stack.pop().unwrap();
                let table = stack.data_stack.pop().unwrap();
                let val = table.table_index_get(index)?;
                stack.data_stack.push(val);
            }
            Instruction::TableIndexSet => {
                let index = stack.data_stack.pop().unwrap();
                let mut table = stack.data_stack.pop().unwrap();
                let value = stack.data_stack.pop().unwrap();
                if !index.is_nil() {
                    table.table_index_set(index, value)?;
                }
            }

            Instruction::FunctionInit(func_id, num_upvalues) => {
                let func = LuaFunctionLua {
                    function_id: *func_id,
                    upvalues: Vec::with_capacity(*num_upvalues),
                };
                let func = LuaFunction::LuaFunc(func);
                stack.data_stack.push(LuaValue::Function(func));
            }
            Instruction::FunctionUpvaluePushFromLocalVar(src_local_id) => {
                let local_var = {
                    let local_var = stack.local_variable_mut(*src_local_id);
                    local_var.to_ref();
                    if let RefOrValue::Ref(r) = local_var {
                        r.clone()
                    } else {
                        unreachable!("local variable must be reference")
                    }
                };

                let dst_func = match stack.data_stack.last_mut().unwrap() {
                    LuaValue::Function(LuaFunction::LuaFunc(f)) => f,
                    _ => unreachable!("stack top must be function"),
                };
                dst_func.upvalues.push(local_var);
            }
            Instruction::FunctionUpvaluePushFromUpvalue(src_upvalue_id) => {
                let func = stack.function_stack.last().unwrap();
                let value = func.function_object.upvalues[*src_upvalue_id].clone();

                let dst_func = match stack.data_stack.last_mut().unwrap() {
                    LuaValue::Function(LuaFunction::LuaFunc(f)) => f,
                    _ => unreachable!("stack top must be function"),
                };
                dst_func.upvalues.push(value);
            }

            Instruction::FunctionUpvalue(upvalue_id) => {
                let func = stack.function_stack.last().unwrap();
                let value = RefCell::borrow(&func.function_object.upvalues[*upvalue_id]).clone();
                stack.data_stack.push(value);
            }
            Instruction::FunctionUpvalueSet(upvalue_id) => {
                let rhs = stack.data_stack.pop().unwrap();
                let func = stack.function_stack.last().unwrap();
                let upvalue = &func.function_object.upvalues[*upvalue_id];
                *RefCell::borrow_mut(upvalue) = rhs;
            }

            Instruction::BinaryAdd => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = lhs.add(&rhs)?;
                stack.data_stack.push(ret);
            }
            Instruction::BinarySub => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = lhs.sub(&rhs)?;
                stack.data_stack.push(ret);
            }
            Instruction::BinaryMul => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = lhs.mul(&rhs)?;
                stack.data_stack.push(ret);
            }
            Instruction::BinaryDiv => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = lhs.div(&rhs)?;
                stack.data_stack.push(ret);
            }
            Instruction::BinaryFloorDiv => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = lhs.fdiv(&rhs)?;
                stack.data_stack.push(ret);
            }
            Instruction::BinaryMod => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = lhs.modu(&rhs)?;
                stack.data_stack.push(ret);
            }
            Instruction::BinaryPow => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = lhs.pow(&rhs)?;
                stack.data_stack.push(ret);
            }
            Instruction::BinaryConcat => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = lhs.concat(&rhs)?;
                stack.data_stack.push(ret);
            }
            Instruction::BinaryBitwiseAnd => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = lhs.bit_and(&rhs)?;
                stack.data_stack.push(ret);
            }
            Instruction::BinaryBitwiseOr => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = lhs.bit_or(&rhs)?;
                stack.data_stack.push(ret);
            }
            Instruction::BinaryBitwiseXor => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = lhs.bit_xor(&rhs)?;
                stack.data_stack.push(ret);
            }
            Instruction::BinaryShiftLeft => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = lhs.bit_lshift(&rhs)?;
                stack.data_stack.push(ret);
            }
            Instruction::BinaryShiftRight => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = lhs.bit_rshift(&rhs)?;
                stack.data_stack.push(ret);
            }
            Instruction::BinaryEqual => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = lhs.eq(&rhs);
                stack.data_stack.push(ret.into());
            }
            Instruction::BinaryNotEqual => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = !lhs.eq(&rhs);
                stack.data_stack.push(ret.into());
            }
            Instruction::BinaryLessThan => {
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = lhs.lt(&rhs)?;
                stack.data_stack.push(ret.into());
            }
            Instruction::BinaryLessEqual => {
                // a <= b <=> !(a > b)
                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = !rhs.lt(&lhs)?;
                stack.data_stack.push(ret.into());
            }
            Instruction::BinaryGreaterThan => {
                // a > b <=> b < a

                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = rhs.lt(&lhs)?;
                stack.data_stack.push(ret.into());
            }
            Instruction::BinaryGreaterEqual => {
                // a >= b <=> !(a < b)

                let rhs = stack.data_stack.pop().unwrap();
                let lhs = stack.data_stack.pop().unwrap();
                let ret = !lhs.lt(&rhs)?;
                stack.data_stack.push(ret.into());
            }

            Instruction::UnaryMinus => {
                let top = stack.data_stack.pop().unwrap();
                let ret = top.neg()?;
                stack.data_stack.push(ret.into());
            }
            Instruction::UnaryBitwiseNot => {
                let top = stack.data_stack.pop().unwrap();
                let ret = top.bit_not()?;
                stack.data_stack.push(ret.into());
            }
            Instruction::UnaryLength => {
                let top = stack.data_stack.pop().unwrap();
                let ret = top.len()?;
                stack.data_stack.push(ret.into());
            }
            Instruction::UnaryLogicalNot => {
                let top = stack.data_stack.pop().unwrap();
                let ret = top.not();
                stack.data_stack.push(ret.into());
            }

            // expected can be 0
            Instruction::FunctionCall(expected_ret) => {
                let sp = stack.usize_stack.pop().unwrap();
                let mut args: Vec<_> = stack.data_stack.drain(sp + 1..).collect();
                let top = stack.data_stack.pop().unwrap();
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
                        let func_info = &self.functions[func_id];
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

                        stack.usize_stack.push(stack.counter + 1);
                        stack.usize_stack.push(stack.bp);
                        stack.usize_stack.push(stack.data_stack.len());
                        stack.bp = stack.local_variables.len();
                        stack.local_variables.resize_with(
                            stack.local_variables.len() + func_info.stack_size,
                            Default::default,
                        );
                        for (idx, arg) in args.into_iter().enumerate() {
                            stack.local_variable_mut(idx).set(arg);
                        }

                        stack.function_stack.push(func_stack);
                        stack.counter = func_info.address;
                        return Ok(false);
                    }
                    LuaFunction::RustFunc(rust_internal) => {
                        let func = rust_internal.func;
                        let mut res = func(args)?;
                        if let Some(expected) = *expected_ret {
                            res.resize_with(expected, Default::default);
                        }
                        stack.data_stack.append(&mut res);
                    }
                }
            }

            Instruction::Return => {
                let sp = stack.usize_stack.pop().unwrap();
                let mut returned_values = stack.data_stack.drain(sp..).collect::<Vec<_>>();
                if let Some(func) = stack.function_stack.pop() {
                    // function call

                    let old_stacklen = stack.usize_stack.pop().unwrap();
                    let old_bp = stack.usize_stack.pop().unwrap();
                    let old_pc = stack.usize_stack.pop().unwrap();
                    stack.local_variables.truncate(stack.bp);
                    stack.bp = old_bp;
                    stack.counter = old_pc;
                    stack.data_stack.truncate(old_stacklen);

                    if let Some(expected) = func.return_expected {
                        returned_values.resize_with(expected, || LuaValue::Nil);
                    }
                    stack.data_stack.append(&mut returned_values);
                    return Ok(false);
                } else {
                    // main chunk
                    stack.counter = self.instructions.len();
                    return Ok(false);
                }
            }

            Instruction::GetVariadic(expected) => {
                let func = stack.function_stack.last().unwrap();
                if let Some(expected) = expected {
                    if func.variadic.len() < *expected {
                        for v in func.variadic.iter() {
                            stack.data_stack.push(v.clone());
                        }
                        for _ in func.variadic.len()..*expected {
                            stack.data_stack.push(LuaValue::Nil);
                        }
                    } else {
                        for v in func.variadic.iter().take(*expected) {
                            stack.data_stack.push(v.clone());
                        }
                    }
                } else {
                    for v in func.variadic.iter() {
                        stack.data_stack.push(v.clone());
                    }
                }
            }

            Instruction::AdjustMultire(size) => {
                let sp = stack.usize_stack.pop().unwrap();
                let count = stack.data_stack.len() - sp;
                if count < *size {
                    for _ in 0..(*size - count) {
                        stack.data_stack.push(LuaValue::Nil);
                    }
                } else if count > *size {
                    for _ in 0..(count - *size) {
                        stack.data_stack.pop();
                    }
                }
            }
        }
        stack.counter += 1;
        Ok(stack.counter >= self.instructions.len())
    }
}
