//! ```rust
//! // type of `lua_parser::Block`
//! let block = lua_parser::parse_str( ... )?;
//!
//! // semantic analysis, and generate enhanced AST for later use.
//! let enhanced_ast = lua_semantics::process(block)?;
//! ```
//!

mod context;
mod error;
mod expression;
mod label;
mod scope;
mod statement;

pub use lua_parser::FloatType;
pub use lua_parser::IntOrFloat;
pub use lua_parser::IntType;

pub use expression::ExprBinary;
pub use expression::ExprBinaryData;
pub use expression::ExprFunctionCall;
pub use expression::ExprFunctionObject;
pub use expression::ExprLocalVariable;
pub use expression::ExprTableConstructor;
pub use expression::ExprTableIndex;
pub use expression::ExprUnary;
pub use expression::ExprUnaryData;
pub use expression::Expression;
pub use expression::FunctionDefinition;

pub use statement::Attrib;
pub use statement::Block;
pub use statement::ReturnStatement;
pub use statement::Statement;
pub use statement::StmtAssignment;
pub use statement::StmtFor;
pub use statement::StmtForGeneric;
pub use statement::StmtFunctionCall;
pub use statement::StmtGoto;
pub use statement::StmtIf;
pub use statement::StmtLabel;
pub use statement::StmtLocalDeclaration;
pub use statement::StmtRepeat;
pub use statement::StmtWhile;

pub use context::Context;
pub use label::LabelInfo;
pub use scope::Scope;
pub use scope::ScopeBlock;
pub use scope::ScopeFunction;
pub use scope::VariableInfo;

pub use error::ProcessError;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub block: Block,
    pub functions: Vec<FunctionDefinition>,
}
/// perform semantic analysis on the given block and generate enhanced AST.
pub fn process(block: lua_parser::Block) -> Result<Chunk, ProcessError> {
    let mut context = Context::new();
    let block = context.process(block)?;

    // check all goto label is defined
    for (_, label_info) in context.labels.iter() {
        if label_info.borrow().scope.is_none() {
            let span = label_info.borrow().from[0].1;
            return Err(ProcessError::InvalidLabel(span));
        }
    }

    Ok(Chunk {
        block,
        functions: context.functions,
    })
}
