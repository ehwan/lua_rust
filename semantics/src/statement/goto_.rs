use crate::LabelInfo;
use std::cell::RefCell;
use std::rc::Rc;

/// Goto statement
#[derive(Clone, Debug)]
pub struct StmtGoto {
    pub label: Rc<RefCell<LabelInfo>>,
}
impl StmtGoto {
    pub fn new(label: Rc<RefCell<LabelInfo>>) -> Self {
        Self { label }
    }
}
