use crate::number::LuaNumber;
use crate::FloatType;
use crate::IntType;
use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaThread;
use crate::RuntimeError;

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
    Function(Rc<RefCell<LuaFunction>>),
    UserData(Rc<RefCell<LuaUserData>>),
    Thread(Rc<RefCell<LuaThread>>),
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
            LuaValue::Function(f) => Rc::as_ptr(f).hash(state),
            LuaValue::UserData(u) => Rc::as_ptr(u).hash(state),
            LuaValue::Thread(t) => Rc::as_ptr(t).hash(state),
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
            (LuaValue::Function(a), LuaValue::Function(b)) => Rc::ptr_eq(a, b),
            (LuaValue::UserData(a), LuaValue::UserData(b)) => Rc::ptr_eq(a, b),
            (LuaValue::Thread(a), LuaValue::Thread(b)) => Rc::ptr_eq(a, b),
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
                write!(f, "table: {:p}", Rc::as_ptr(t))
            }
            LuaValue::Function(func) => write!(f, "function: {:p}", Rc::as_ptr(func)),
            LuaValue::UserData(userdata) => write!(f, "userdata: {:p}", Rc::as_ptr(userdata)),
            LuaValue::Thread(thread) => write!(f, "thread: {:p}", Rc::as_ptr(thread)),
        }
    }
}

impl LuaValue {
    pub fn type_str(&self) -> &'static str {
        match self {
            LuaValue::Nil => "nil",
            LuaValue::Boolean(_) => "boolean",
            LuaValue::Number(_) => "number",
            LuaValue::String(_) => "string",
            LuaValue::Table(_) => "table",
            LuaValue::Function(_) => "function",
            LuaValue::Thread(_) => "thread",
            LuaValue::UserData(_) => "userdata",
        }
    }
    pub fn to_bool(&self) -> bool {
        match self {
            LuaValue::Nil | LuaValue::Boolean(false) => false,
            _ => true,
        }
    }
    pub fn try_to_int(&self) -> Result<IntType, RuntimeError> {
        self.try_to_number()?.try_to_int()
    }
    pub fn try_to_number(&self) -> Result<LuaNumber, RuntimeError> {
        match self {
            LuaValue::Number(n) => Ok(*n),
            LuaValue::String(s) => {
                // use `lua_tokenizer` to parse the string into a number
                let mut tokenizer = lua_tokenizer::Tokenizer::from_bytes(s);
                tokenizer.ignore_whitespace();
                // sign
                let neg = match tokenizer.peek() {
                    Some(b'-') => {
                        tokenizer.next();
                        true
                    }
                    Some(b'+') => {
                        tokenizer.next();
                        false
                    }
                    _ => false,
                };
                // number
                let tokenize_res = tokenizer.tokenize_numeric();
                match tokenize_res {
                    Ok(Some(res)) => {
                        tokenizer.ignore_whitespace();
                        if tokenizer.is_end() {
                            match res.token_type {
                                lua_tokenizer::TokenType::Numeric(numeric) => match numeric {
                                    lua_tokenizer::IntOrFloat::Int(i) => {
                                        if neg {
                                            Ok((-i).into())
                                        } else {
                                            Ok(i.into())
                                        }
                                    }
                                    lua_tokenizer::IntOrFloat::Float(f) => {
                                        if neg {
                                            Ok((-f).into())
                                        } else {
                                            Ok(f.into())
                                        }
                                    }
                                },
                                _ => Err(RuntimeError::Expected("number", Some("string"))),
                            }
                        } else {
                            Err(RuntimeError::Expected("number", Some("string")))
                        }
                    }
                    Ok(None) => Err(RuntimeError::Expected("number", Some("string"))),
                    Err(tokenize_error) => Err(RuntimeError::TokenizeError(tokenize_error)),
                }
            }
            _ => Err(RuntimeError::Expected("number", Some(self.type_str()))),
        }
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
        LuaValue::Number(LuaNumber::Int(n))
    }
}
impl From<FloatType> for LuaValue {
    fn from(n: FloatType) -> Self {
        LuaValue::Number(LuaNumber::Float(n))
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
        LuaValue::Function(Rc::new(RefCell::new(f)))
    }
}
impl From<LuaUserData> for LuaValue {
    fn from(u: LuaUserData) -> Self {
        LuaValue::UserData(Rc::new(RefCell::new(u)))
    }
}
impl From<LuaThread> for LuaValue {
    fn from(t: LuaThread) -> Self {
        LuaValue::Thread(Rc::new(RefCell::new(t)))
    }
}

#[derive(Debug, Clone)]
pub struct LuaUserData {}
