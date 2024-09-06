use super::Block;
use crate::Expression;

#[derive(Clone, Debug)]
pub struct StmtRepeat {
    pub block: Block,
    pub condition: Expression,
}
impl StmtRepeat {
    pub fn new(block: Block, condition: Expression) -> Self {
        Self { block, condition }
    }
}
