use crate::number::LuaNumber;
use crate::FloatType;
use crate::IntType;
use crate::LuaFunction;
use crate::LuaTable;

use std::cell::RefCell;
use std::rc::Rc;

/// for local variables and upvalues.
#[derive(Debug, Clone)]
pub enum RefOrValue {
    Ref(Rc<RefCell<LuaValue>>),
    Value(LuaValue),
}
impl Default for RefOrValue {
    fn default() -> Self {
        RefOrValue::Value(LuaValue::Nil)
    }
}

#[derive(Debug, Clone)]
pub enum LuaValue {
    Nil,
    Boolean(bool),
    Number(LuaNumber),
    String(Vec<u8>),
    Table(Rc<RefCell<LuaTable>>),
    Function(LuaFunction),
    UserData(LuaUserData),
    Thread(LuaThread),
}
impl std::hash::Hash for LuaValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);

        match self {
            LuaValue::Nil => {
                unreachable!("hash for nil; this should be filtered out");
            }
            LuaValue::Boolean(b) => b.hash(state),
            LuaValue::Number(n) => n.hash(state),
            LuaValue::String(s) => s.hash(state),
            LuaValue::Table(t) => Rc::as_ptr(t).hash(state),
            LuaValue::Function(_) => {
                unimplemented!("hash for function");
                // func.hash(state)
            }
            LuaValue::UserData(_) => {
                unimplemented!("hash for userdata")
            }
            LuaValue::Thread(_) => {
                unimplemented!("hash for thread")
            }
        }
    }
}
impl std::cmp::PartialEq for LuaValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LuaValue::Nil, LuaValue::Nil) => true,
            (LuaValue::Boolean(a), LuaValue::Boolean(b)) => a == b,
            (LuaValue::Number(a), LuaValue::Number(b)) => a == b,
            (LuaValue::String(a), LuaValue::String(b)) => a == b,
            (LuaValue::Table(a), LuaValue::Table(b)) => Rc::ptr_eq(a, b),
            (LuaValue::Function(_a), LuaValue::Function(_b)) => {
                unimplemented!("function equality")
            }
            (LuaValue::UserData(_), LuaValue::UserData(_)) => {
                unimplemented!("userdata equality")
            }
            (LuaValue::Thread(_), LuaValue::Thread(_)) => {
                unimplemented!("thread equality")
            }
            _ => false,
        }
    }
}
impl std::cmp::Eq for LuaValue {}

impl std::fmt::Display for LuaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaValue::Nil => write!(f, "nil"),
            LuaValue::Boolean(b) => write!(f, "{}", b),
            LuaValue::Number(n) => write!(f, "{}", n),
            LuaValue::String(s) => write!(f, "{}", String::from_utf8_lossy(s)),
            LuaValue::Table(t) => {
                write!(f, "table {:p}", Rc::as_ptr(t))
            }
            LuaValue::Function(func) => write!(f, "{}", func),
            LuaValue::UserData(_) => write!(f, "userdata"),
            LuaValue::Thread(_) => write!(f, "thread"),
        }
    }
}

impl LuaValue {
    pub fn to_bool(&self) -> bool {
        match self {
            LuaValue::Nil | LuaValue::Boolean(false) => false,
            _ => true,
        }
    }
    pub fn try_to_int(&self) -> Option<IntType> {
        match self {
            LuaValue::Number(n) => n.try_to_int(),
            LuaValue::String(s) => {
                let s = String::from_utf8_lossy(s);
                s.parse().ok()
            }
            _ => None,
        }
    }
    pub fn try_to_number(&self) -> Option<LuaNumber> {
        match self {
            LuaValue::Number(n) => Some(*n),
            LuaValue::String(s) => {
                let s = String::from_utf8_lossy(s);
                s.parse().ok().map(LuaNumber::Float)
            }
            _ => None,
        }
    }
    pub fn try_to_string(&self) -> Option<Vec<u8>> {
        match self {
            LuaValue::String(s) => Some(s.clone()),
            LuaValue::Number(n) => Some(n.to_string().into_bytes()),
            _ => None,
        }
    }

    pub fn not(&self) -> LuaValue {
        LuaValue::Boolean(!self.to_bool())
    }

    pub fn is_nil(&self) -> bool {
        match self {
            LuaValue::Nil => true,
            _ => false,
        }
    }
    pub fn is_nan(&self) -> bool {
        match self {
            LuaValue::Number(n) => n.is_nan(),
            _ => false,
        }
    }
}

impl Default for LuaValue {
    fn default() -> Self {
        LuaValue::Nil
    }
}

impl From<()> for LuaValue {
    fn from(_: ()) -> Self {
        LuaValue::Nil
    }
}
impl From<bool> for LuaValue {
    fn from(b: bool) -> Self {
        LuaValue::Boolean(b)
    }
}
impl From<IntType> for LuaValue {
    fn from(n: IntType) -> Self {
        LuaValue::Number(n.into())
    }
}
impl From<FloatType> for LuaValue {
    fn from(n: FloatType) -> Self {
        LuaValue::Number(n.into())
    }
}
impl From<LuaNumber> for LuaValue {
    fn from(n: LuaNumber) -> Self {
        LuaValue::Number(n)
    }
}
impl From<String> for LuaValue {
    fn from(s: String) -> Self {
        LuaValue::String(s.into_bytes())
    }
}
impl From<&str> for LuaValue {
    fn from(s: &str) -> Self {
        LuaValue::String(s.bytes().collect())
    }
}
impl From<LuaTable> for LuaValue {
    fn from(t: LuaTable) -> Self {
        LuaValue::Table(Rc::new(RefCell::new(t)))
    }
}
impl From<LuaFunction> for LuaValue {
    fn from(f: LuaFunction) -> Self {
        LuaValue::Function(f)
    }
}
impl From<LuaUserData> for LuaValue {
    fn from(u: LuaUserData) -> Self {
        LuaValue::UserData(u)
    }
}
impl From<LuaThread> for LuaValue {
    fn from(t: LuaThread) -> Self {
        LuaValue::Thread(t)
    }
}

#[derive(Debug, Clone)]
pub struct LuaUserData {}

#[derive(Debug, Clone)]
pub struct LuaThread {}
