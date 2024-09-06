use crate::{expression, Expression};

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

#[derive(Clone, Debug)]
pub struct ReturnStatement {
    pub values: Vec<Expression>,
}
impl ReturnStatement {
    pub fn new(values: Vec<Expression>) -> Self {
        Self { values }
    }
}

#[derive(Clone, Debug)]
pub enum Statement {
    /// `;`
    None,
    /// `l0, l1, l2 = r0, r1, r2`.
    /// variadic `...` can be used in both `l` and `r`
    Assignment(Assignment),
    Label(Label),
    Break,
    Goto(Goto),
    Do(Do),
    While(While),
    Repeat(Repeat),
    If(If),
    For(For),
    ForGeneric(ForGeneric),
    LocalDeclaration(LocalDeclaration),
    FunctionDefinition(FunctionDefinition),
    FunctionDefinitionLocal(FunctionDefinitionLocal),
    FunctionCall(expression::FunctionCall),
}

/// `l0, l1, l2 = r0, r1, r2`.
/// variadic `...` can be used in both `l` and `r`
#[derive(Clone, Debug)]
pub struct Assignment {
    pub lhs: Vec<Expression>,
    pub rhs: Vec<Expression>,
}
impl Assignment {
    pub fn new(lhs: Vec<Expression>, rhs: Vec<Expression>) -> Self {
        // @TODO check variadic here
        Self { lhs, rhs }
    }
}

/// label definition
#[derive(Clone, Debug)]
pub struct Label {
    pub name: String,
}
impl Label {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
#[derive(Clone, Debug)]
pub struct Goto {
    pub name: String,
}
impl Goto {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Clone, Debug)]
pub struct Do {
    pub block: Block,
}
impl Do {
    pub fn new(block: Block) -> Self {
        Self { block }
    }
}

#[derive(Clone, Debug)]
pub struct While {
    pub condition: Expression,
    pub block: Block,
}
impl While {
    pub fn new(condition: Expression, block: Block) -> Self {
        Self { condition, block }
    }
}

#[derive(Clone, Debug)]
pub struct Repeat {
    pub block: Block,
    pub condition: Expression,
}
impl Repeat {
    pub fn new(block: Block, condition: Expression) -> Self {
        Self { block, condition }
    }
}

#[derive(Clone, Debug)]
pub struct If {
    pub condition: Expression,
    pub block: Block,
    pub else_ifs: Vec<(Expression, Block)>,
    pub else_block: Option<Block>,
}
impl If {
    pub fn new(
        condition: Expression,
        block: Block,
        else_ifs: Vec<(Expression, Block)>,
        else_block: Option<Block>,
    ) -> Self {
        Self {
            condition,
            block,
            else_ifs,
            else_block,
        }
    }
}

#[derive(Clone, Debug)]
pub struct For {
    pub name: String,
    pub start: Expression,
    pub end: Expression,
    pub step: Option<Expression>,
    pub block: Block,
}
impl For {
    pub fn new(
        name: String,
        start: Expression,
        end: Expression,
        step: Option<Expression>,
        block: Block,
    ) -> Self {
        Self {
            name,
            start,
            end,
            step,
            block,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ForGeneric {
    pub names: Vec<String>,
    pub expressions: Vec<Expression>,
    pub block: Block,
}
impl ForGeneric {
    pub fn new(names: Vec<String>, expressions: Vec<Expression>, block: Block) -> Self {
        Self {
            names,
            expressions,
            block,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Attrib {
    Const,
    Close,
}
#[derive(Clone, Debug)]
pub struct AttName {
    pub name: String,
    pub attrib: Option<Attrib>,
}
impl AttName {
    pub fn new(name: String, attrib: Option<Attrib>) -> Self {
        Self { name, attrib }
    }
}

#[derive(Clone, Debug)]
pub struct LocalDeclaration {
    pub names: Vec<AttName>,
    pub values: Option<Vec<Expression>>,
}
impl LocalDeclaration {
    pub fn new(names: Vec<AttName>, values: Option<Vec<Expression>>) -> Self {
        Self { names, values }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionName {
    /// dot chain
    pub names: Vec<String>,
    /// colon chain at the end
    pub colon: Option<String>,
}
impl FunctionName {
    pub fn new(names: Vec<String>, colon: Option<String>) -> Self {
        Self { names, colon }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionDefinition {
    pub name: FunctionName,
    pub body: crate::expression::FunctionBody,
}
impl FunctionDefinition {
    pub fn new(name: FunctionName, body: crate::expression::FunctionBody) -> Self {
        Self { name, body }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionDefinitionLocal {
    pub name: String,
    pub body: crate::expression::FunctionBody,
}
impl FunctionDefinitionLocal {
    pub fn new(name: String, body: crate::expression::FunctionBody) -> Self {
        Self { name, body }
    }
}
