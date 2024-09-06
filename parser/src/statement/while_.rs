use super::Block;
use crate::Expression;

#[derive(Clone, Debug)]
pub struct StmtWhile {
    pub condition: Expression,
    pub block: Block,
}
impl StmtWhile {
    pub fn new(condition: Expression, block: Block) -> Self {
        Self { condition, block }
    }
}
