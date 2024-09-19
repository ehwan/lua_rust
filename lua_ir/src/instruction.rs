use lua_semantics::IntType;

use crate::IntOrFloat;
use crate::LabelType;

#[derive(Debug, Clone)]
pub enum Instruction {
    /// clear i'th local variable to Nil
    Clear(usize),
    /// clone top of the data_stack and push it
    Clone,
    /// swap top two elements of stack
    Swap,
    /// push current length of data_stack to usize_stack
    Sp,
    ///
    Pop,

    /// jump to label
    Jump(LabelType),
    /// pops data_stack and jump to label if stack_top is true.
    JumpTrue(LabelType),
    /// pops data_stack and jump to label if stack_top is false
    JumpFalse(LabelType),

    /// get i'th local variable and push the value to stack_top
    GetLocalVariable(usize),
    /// pops data_stack and set i'th local variable to the value.
    /// If i'th local variable is `Ref`, the internal value will be set.
    SetLocalVariable(usize),

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
    FunctionUpvaluePushFromLocalVar(usize),
    /// src_upvalue_id
    FunctionUpvaluePushFromUpvalue(usize),

    /// get i'th upvalue of current function
    FunctionUpvalue(usize),
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
