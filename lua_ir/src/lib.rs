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

pub use function::LuaFunction;
pub use function::LuaFunctionLua;
/// Type for any Lua value.
pub use luaval::LuaValue;
/// Type for Lua number.
pub use number::LuaNumber;
/// Type for Lua table.
pub use table::LuaTable;

use context::Context;
pub use error::RuntimeError;
pub use instruction::Instruction;
use vm::Chunk;
pub use vm::LuaEnv;
pub use vm::LuaThread;
pub use vm::ThreadStatus;
