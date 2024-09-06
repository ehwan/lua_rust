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
    /// `;`
    None,
    /// `l0, l1, l2 = r0, r1, r2`.
    /// variadic `...` can be used in both `l` and `r`
    Assignment(StmtAssignment),
    Label(StmtLabel),
    Break(StmtBreak),
    Goto(StmtGoto),
    Do(StmtDo),
    While(StmtWhile),
    Repeat(StmtRepeat),
    If(StmtIf),
    For(StmtFor),
    ForGeneric(StmtForGeneric),
    LocalDeclaration(StmtLocalDeclaration),
    FunctionDefinition(StmtFunctionDefinition),
    FunctionDefinitionLocal(StmtFunctionDefinitionLocal),
    FunctionCall(StmtFunctionCall),
}

mod assignment;
pub use assignment::StmtAssignment;

mod break_;
pub use break_::StmtBreak;

mod label;
pub use label::StmtLabel;

mod goto_;
pub use goto_::StmtGoto;

mod do_;
pub use do_::StmtDo;

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
pub use localdecl::AttName;
pub use localdecl::Attrib;
pub use localdecl::StmtLocalDeclaration;

mod functiondef;
pub use functiondef::FunctionName;
pub use functiondef::StmtFunctionDefinition;
pub use functiondef::StmtFunctionDefinitionLocal;

mod functioncall;
pub use functioncall::StmtFunctionCall;
