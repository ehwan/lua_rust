use lua_semantics::IntType;

use crate::IntOrFloat;

#[derive(Debug, Clone)]
pub enum Instruction {
    Clear(usize),
    /// clone stack top and push it to stack
    Clone,
    /// swap top two elements of stack
    Swap,
    /// push length of stack to usize_stack
    Sp,

    /// jump to label
    Jump(String),
    /// jump to label if stack_top is true
    JumpTrue(String),
    /// jump to label if stack_top is false
    JumpFalse(String),

    /// push stack[i] to stack_top
    GetStack(usize),
    /// set stack[i] from stack_top
    SetStack(usize),

    /// push nil
    Nil,
    /// push bool
    Boolean(bool),
    /// push int or float
    Numeric(IntOrFloat),
    /// push string
    String(String),

    /// push _G
    GetGlobal,
    /// push _ENV
    GetEnv,

    /// init new table with capacity
    TableInit(usize),
    /// table -> key -> value -> stack_top.
    /// table must not be popped.
    TableIndexInit,
    ///
    TableInitLast(IntType),
    /// table -> index -> stack_top
    TableIndex,
    /// value -> table -> index -> stack_top
    TableIndexSet,

    /// function_id, number of upvalues
    FunctionInit(usize, usize),
    /// func -> top.
    /// src_stack_id
    FunctionUpvaluePushWithStack(usize),
    /// src_upvalue_id
    FunctionUpvaluePushWithUpvalue(usize),

    /// get i'th upvalue of current function
    FunctionUpvalue(usize),
    /// set i'th upvalue of current function
    FunctionUpvalueSet(usize),

    /// set reference
    Ref(usize),
    /// Dereference
    Deref(usize),

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
    BinaryNotEqual,
    BinaryLessThan,
    BinaryLessEqual,
    BinaryGreaterThan,
    BinaryGreaterEqual,

    UnaryMinus,
    UnaryBitwiseNot,
    UnaryLength,
    UnaryLogicalNot,

    /// return expected can be 0.
    /// function -> args -> top
    FunctionCall(Option<usize>),

    Return,

    /// Invalid call ( using `...` in a non-variadic function ) was filtered out in parser.
    GetVariadic(Option<usize>),

    AdjustMultire(usize),
}
