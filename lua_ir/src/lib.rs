mod builtin;
mod context;
mod error;
mod function;
mod instruction;
mod luaval;
mod number;
mod table;
mod vm;

/// The type of a label in the program.
/// It is actually `usize`,
/// we just use this type alias to make the code more readable,
/// and distinguish it from other `usize` like index of instructions.
type LabelType = usize;

pub use lua_semantics::FloatType;
pub use lua_semantics::IntType;

pub use function::FunctionInfo;
pub use function::LuaFunction;
pub use function::LuaFunctionLua;
pub use luaval::LuaValue;
pub use number::LuaNumber;
pub use table::LuaTable;

pub use context::Context;
pub use error::RuntimeError;
pub use instruction::Instruction;
pub use vm::Chunk;
pub use vm::LuaEnv;
pub use vm::Stack;
