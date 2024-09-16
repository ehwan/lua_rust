use super::Block;
use crate::Expression;

/// if statement
#[derive(Clone, Debug)]
pub struct StmtIf {
    pub condition: Expression,
    pub block: Block,
    pub else_ifs: Vec<(Expression, Block)>,
    pub else_: Option<Block>,
}
impl StmtIf {
    pub fn new(
        condition: Expression,
        block: Block,
        else_ifs: Vec<(Expression, Block)>,
        else_: Option<Block>,
    ) -> Self {
        Self {
            condition,
            block,
            else_ifs,
            else_,
        }
    }
}
