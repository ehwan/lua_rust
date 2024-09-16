use crate::Expression;

/// block of statements.
/// return statement must be optionally placed at the end of the block.
#[derive(Clone, Debug)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub return_statement: Option<ReturnStatement>,
}
impl Block {
    pub fn new(statements: Vec<Statement>, return_statement: Option<ReturnStatement>) -> Self {
        Self {
            statements,
            return_statement,
        }
    }
}

/// return statement
#[derive(Clone, Debug)]
pub struct ReturnStatement {
    pub values: Vec<Expression>,
}
impl ReturnStatement {
    pub fn new(values: Vec<Expression>) -> Self {
        Self { values }
    }
}

/// lua statement
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum Statement {
    /// `l0, l1, l2 = r0, r1, r2`.
    /// variadic `...` can be used in both `l` and `r`
    Assignment(StmtAssignment),
    Break,
    Do(Block),
    While(StmtWhile),
    Repeat(StmtRepeat),
    If(StmtIf),
    For(StmtFor),
    ForGeneric(StmtForGeneric),
    FunctionCall(StmtFunctionCall),
    LocalDeclaration(StmtLocalDeclaration),
    Goto(StmtGoto),
    Label(StmtLabel),
}
impl Statement {}

mod assignment;
pub use assignment::StmtAssignment;

mod while_;
pub use while_::StmtWhile;

mod repeat;
pub use repeat::StmtRepeat;

mod if_;
pub use if_::StmtIf;

mod for_;
pub use for_::StmtFor;
pub use for_::StmtForGeneric;

mod localdecl;
pub use localdecl::Attrib;
pub use localdecl::StmtLocalDeclaration;

mod goto_;
pub use goto_::StmtGoto;

mod label;
pub use label::StmtLabel;

mod functioncall;
pub use functioncall::StmtFunctionCall;
