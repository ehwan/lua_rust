use crate::Expression;
use crate::Span;

/// block of statements.
/// return statement must be optionally placed at the end of the block.
#[derive(Clone, Debug)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub return_statement: Option<ReturnStatement>,
    pub span: Span,
}
impl Block {
    pub fn new(
        statements: Vec<Statement>,
        return_statement: Option<ReturnStatement>,
        span: Span,
    ) -> Self {
        Self {
            statements,
            return_statement,
            span,
        }
    }
    /// get the span of the block.
    /// *NOTE* if the block is empty, the span will hold `usize::MAX` (by `Span::new_none()`)
    pub fn span(&self) -> Span {
        self.span
    }
}

/// return statement
#[derive(Clone, Debug)]
pub struct ReturnStatement {
    pub values: Vec<Expression>,
    pub span: Span,
}
impl ReturnStatement {
    pub fn new(values: Vec<Expression>, span: Span) -> Self {
        Self { values, span }
    }
    /// get the span of the return statement
    pub fn span(&self) -> Span {
        self.span
    }
}

/// lua statement
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum Statement {
    /// `;`
    None(StmtNone),
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
impl Statement {
    /// get the span of the statement
    pub fn span(&self) -> Span {
        match self {
            Self::None(v) => v.span(),
            Self::Assignment(v) => v.span(),
            Self::Label(v) => v.span(),
            Self::Break(v) => v.span(),
            Self::Goto(v) => v.span(),
            Self::Do(v) => v.span(),
            Self::While(v) => v.span(),
            Self::Repeat(v) => v.span(),
            Self::If(v) => v.span(),
            Self::For(v) => v.span(),
            Self::ForGeneric(v) => v.span(),
            Self::LocalDeclaration(v) => v.span(),
            Self::FunctionDefinition(v) => v.span(),
            Self::FunctionDefinitionLocal(v) => v.span(),
            Self::FunctionCall(v) => v.span(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StmtNone {
    pub span: Span,
}
impl StmtNone {
    pub fn new(span: Span) -> Self {
        Self { span }
    }
    /// get the span of the `;`
    pub fn span(&self) -> Span {
        self.span
    }
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
pub use if_::StmtElseIf;
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
