use super::Block;

#[derive(Clone, Debug)]
pub struct StmtDo {
    pub block: Block,
}
impl StmtDo {
    pub fn new(block: Block) -> Self {
        Self { block }
    }
}
