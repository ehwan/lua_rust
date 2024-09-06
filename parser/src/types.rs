/// lua types
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Nil,
    Boolean,
    Number,
    String,
    Table,
    Function,
    UserData,
    Thread,
}
