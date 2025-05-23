use lua_semantics::IntType;

use crate::{LabelType, LuaFunctionLua, LuaNumber};

/// Instructions for Lua VM.
#[derive(Debug, Clone)]
pub enum Instruction {
    /// clone top of the data_stack and push it
    Clone,
    /// push current length of data_stack to usize_stack
    Sp,
    ///
    Pop,

    /// clone where top of the usize_stack points to.
    /// Does not pop from usize_stack.
    Deref,

    /// jump to label
    Jump(LabelType),
    /// pops data_stack and jump to label if stack_top is true.
    JumpTrue(LabelType),
    /// pops data_stack and jump to label if stack_top is false
    JumpFalse(LabelType),

    /// get i'th local variable and push the value to stack_top
    GetLocalVariable(usize, String),
    /// pop data_stack and set i'th local variable to the value.
    /// If i'th local variable is `Ref`, the internal value will be set.
    SetLocalVariable(usize),
    /// pop data_stack and initialize i'th local variable to the `Value`.
    InitLocalVariable(usize),

    /// pop data_stack and check if it is nil.
    IsNil,

    /// push nil
    Nil,
    /// push bool
    Boolean(bool),
    /// push int or float
    Numeric(LuaNumber),
    /// push string
    String(Vec<u8>),

    /// push _ENV
    GetEnv,

    /// init new table with capacity
    TableInit(usize),
    /// table -> key -> value -> stack_top.
    /// table must not be popped.
    TableIndexInit,
    /// sp pushed to usize_stack, points to the start of args.
    /// table -> args... -> stack_top
    ///          ^ sp
    TableInitLast(IntType),
    /// table -> index -> stack_top
    TableIndex,
    /// value -> table -> index -> stack_top
    TableIndexSet,

    /// function_id, number of upvalues
    FunctionInit(Box<LuaFunctionLua>),
    /// func -> top.
    /// src_stack_id
    FunctionInitUpvalueFromLocalVar(usize),
    /// src_upvalue_id
    FunctionInitUpvalueFromUpvalue(usize),

    /// get i'th upvalue of current function
    FunctionUpvalue(usize, String),
    /// set i'th upvalue of current function
    FunctionUpvalueSet(usize),

    BinaryAdd,
    BinarySub,
    BinaryMul,
    BinaryDiv,
    BinaryFloorDiv,
    BinaryMod,
    BinaryPow,
    BinaryConcat,
    BinaryBitwiseAnd,
    BinaryBitwiseOr,
    BinaryBitwiseXor,
    BinaryShiftLeft,
    BinaryShiftRight,
    BinaryEqual,
    BinaryLessThan,
    BinaryLessEqual,

    UnaryMinus,
    UnaryBitwiseNot,
    UnaryLength,
    UnaryLogicalNot,

    /// return expected can be 0.
    /// sp pushed to usize_stack, points to the start of args.
    /// args -> function -> stack top
    FunctionCall(Option<usize>),

    Return,

    /// Invalid call ( using `...` in a non-variadic function ) was filtered out in parser.
    GetVariadic(Option<usize>),
}
