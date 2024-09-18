use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use lua_semantics::FunctionDefinition;
use lua_tokenizer::IntOrFloat;
use lua_tokenizer::IntType;

use crate::luaval::LuaFunction;
use crate::luaval::LuaFunctionInternal;
use crate::luaval::LuaRef;
use crate::luaval::LuaTable;
use crate::luaval::RustFuncType;
use crate::LuaValue;

use crate::Instruction;
use crate::RuntimeError;

pub struct FunctionStack {
    pub function_object: Rc<RefCell<LuaFunctionInternal>>,
    pub return_expected: Option<usize>,
    pub variadic: Vec<LuaValue>,
}

pub struct Stack {
    pub global: LuaValue,
    pub data_stack: Vec<LuaValue>,
    pub usize_stack: Vec<usize>,
    // function object, variadic, return values multire expected count
    pub function_stack: Vec<FunctionStack>,
    pub counter: usize,
    pub bp: usize,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub label_map: HashMap<String, usize>,
    pub function_map: Vec<String>,
    pub functions: Vec<FunctionDefinition>,
    pub stack_size: usize,
}
impl Program {
    pub fn new_stack(&self) -> Stack {
        let mut data_stack = Vec::new();
        data_stack.resize_with(self.stack_size, || LuaValue::Nil);
        Stack {
            global: LuaValue::new_global(),
            data_stack,
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
            Instruction::Clone => {
                let top = stack.data_stack.last().unwrap().deref();
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
            Instruction::Jump(label) => {
                let pc = *self.label_map.get(label).unwrap();
                stack.counter = pc;
                return Ok(false);
            }
            Instruction::JumpTrue(label) => {
                if stack.data_stack.pop().unwrap().to_bool() {
                    let pc = *self.label_map.get(label).unwrap();
                    stack.counter = pc;
                    return Ok(false);
                }
            }
            Instruction::JumpFalse(label) => {
                if !stack.data_stack.pop().unwrap().to_bool() {
                    let pc = *self.label_map.get(label).unwrap();
                    stack.counter = pc;
                    return Ok(false);
                }
            }
            Instruction::GetStack(i) => {
                let top = stack.data_stack[stack.bp + *i].deref();
                stack.data_stack.push(top);
            }
            Instruction::SetStack(i) => {
                let top = stack.data_stack.pop().unwrap();
                let lhs = &mut stack.data_stack[stack.bp + *i];
                match lhs {
                    LuaValue::Ref(r) => {
                        *RefCell::borrow_mut(&r.value) = top;
                    }
                    _ => {
                        *lhs = top;
                    }
                }
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
                stack.data_stack.push(LuaValue::Table(table));
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
                let func = LuaFunctionInternal {
                    function_id: *func_id,
                    upvalues: Vec::with_capacity(*num_upvalues),
                };
                let func = LuaFunction::LuaFunc(Rc::new(RefCell::new(func)));
                stack.data_stack.push(LuaValue::Function(func));
            }
            Instruction::FunctionUpvaluePushWithStack(src_stack_id) => {
                let value = stack.data_stack[stack.bp + *src_stack_id].clone();
                let value = match value {
                    LuaValue::Ref(r) => r,
                    _ => unreachable!("source of upvalue must be reference"),
                };

                let dst_func = match stack.data_stack.last().unwrap() {
                    LuaValue::Function(LuaFunction::LuaFunc(f)) => f,
                    _ => unreachable!("stack top must be function"),
                };
                dst_func.borrow_mut().upvalues.push(value);
            }
            Instruction::FunctionUpvaluePushWithUpvalue(src_upvalue_id) => {
                let func = stack.function_stack.last().unwrap();
                let value =
                    RefCell::borrow(&func.function_object).upvalues[*src_upvalue_id].clone();

                let dst_func = match stack.data_stack.last().unwrap() {
                    LuaValue::Function(LuaFunction::LuaFunc(f)) => f,
                    _ => unreachable!("stack top must be function"),
                };
                dst_func.borrow_mut().upvalues.push(value);
            }

            Instruction::FunctionUpvalue(upvalue_id) => {
                let func = stack.function_stack.last().unwrap();
                let value = RefCell::borrow(&func.function_object).upvalue(*upvalue_id);
                stack.data_stack.push(LuaValue::Ref(value));
            }
            Instruction::FunctionUpvalueSet(upvalue_id) => {
                let rhs = stack.data_stack.pop().unwrap();
                let func = stack.function_stack.last().unwrap();
                let upvalue = RefCell::borrow(&func.function_object).upvalue(*upvalue_id);
                *upvalue.value.borrow_mut() = rhs;
            }

            Instruction::InitRef(stack_offset) => {
                let data = stack.data_stack[stack.bp + *stack_offset].clone();
                let data_ref = match data {
                    LuaValue::Ref(r) => r,
                    _ => LuaRef {
                        value: Rc::new(RefCell::new(data)),
                    },
                };
                stack.data_stack[stack.bp + *stack_offset] = LuaValue::Ref(data_ref);
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
                let func = match top.deref() {
                    LuaValue::Function(f) => f,
                    _ => {
                        return Err(RuntimeError::FunctionCallOnNonFunction);
                    }
                };

                match func {
                    LuaFunction::LuaFunc(lua_internal) => {
                        let func_id: usize = RefCell::borrow(Rc::borrow(&lua_internal)).function_id;
                        let func_info = &self.functions[func_id];
                        let mut variadic = if args.len() < func_info.args.len() {
                            for _ in args.len()..func_info.args.len() {
                                args.push(LuaValue::Nil);
                            }
                            Vec::new()
                        } else {
                            args.split_off(func_info.args.len())
                        };
                        if !func_info.variadic {
                            variadic.clear()
                        }

                        let function_label = &self.function_map[func_id];
                        let function_address = *self.label_map.get(function_label).unwrap();
                        let func_stack = FunctionStack {
                            function_object: Rc::clone(&lua_internal),
                            return_expected: *expected_ret,
                            variadic,
                        };

                        stack.usize_stack.push(stack.counter + 1);
                        stack.usize_stack.push(stack.bp);
                        stack.usize_stack.push(stack.data_stack.len());
                        stack.bp = stack.data_stack.len();
                        stack
                            .data_stack
                            .resize_with(stack.data_stack.len() + func_info.stack_size, || {
                                LuaValue::Nil
                            });
                        for (idx, arg) in args.into_iter().enumerate() {
                            stack.data_stack[idx + stack.bp] = arg;
                        }

                        stack.function_stack.push(func_stack);
                        stack.counter = function_address;
                        return Ok(false);
                    }
                    LuaFunction::RustFunc(rust_internal) => {
                        let func: &RustFuncType = Rc::borrow(&rust_internal.func);
                        let mut res = func(args)?;
                        if let Some(expected) = *expected_ret {
                            res.resize_with(expected, || LuaValue::Nil);
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

                    let old_sp = stack.usize_stack.pop().unwrap();
                    let old_bp = stack.usize_stack.pop().unwrap();
                    let old_pc = stack.usize_stack.pop().unwrap();
                    stack.bp = old_bp;
                    stack.counter = old_pc;
                    stack.data_stack.truncate(old_sp);

                    if let Some(expected) = func.return_expected {
                        if returned_values.len() < expected {
                            for _ in returned_values.len()..expected {
                                returned_values.push(LuaValue::Nil);
                            }
                        } else if returned_values.len() > expected {
                            returned_values.truncate(expected);
                        }
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