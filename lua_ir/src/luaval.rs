use crate::FloatType;
use crate::IntType;
use crate::LuaFunction;
use crate::LuaTable;
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
impl From<RefOrValue> for LuaValue {
    fn from(rv: RefOrValue) -> Self {
        match rv {
            RefOrValue::Ref(r) => r.borrow().clone(),
            RefOrValue::Value(v) => v,
        }
    }
}
impl RefOrValue {
    pub fn set(&mut self, value: LuaValue) {
        match self {
            RefOrValue::Ref(r) => {
                *r.borrow_mut() = value;
            }
            RefOrValue::Value(v) => {
                *v = value;
            }
        }
    }
    /// set `self` to `Ref` if it is `Value`.
    pub fn to_ref(&mut self) {
        match self.clone() {
            RefOrValue::Value(v) => {
                let r = Rc::new(RefCell::new(v));
                *self = RefOrValue::Ref(r);
            }
            RefOrValue::Ref(_) => {}
        }
    }
}

#[derive(Debug, Clone)]
pub enum LuaValue {
    Nil,
    Boolean(bool),
    Int(IntType),
    Float(FloatType),
    String(String),
    Table(Rc<RefCell<LuaTable>>),
    Function(LuaFunction),
    UserData(LuaUserData),
    Thread(LuaThread),
}

impl std::fmt::Display for LuaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaValue::Nil => write!(f, "nil"),
            LuaValue::Boolean(b) => write!(f, "{}", b),
            LuaValue::Int(n) => write!(f, "{}", n),
            LuaValue::Float(n) => write!(f, "{}", n),
            LuaValue::String(s) => write!(f, "{}", s),
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
    pub fn add(&self, other: &LuaValue) -> Result<LuaValue, RuntimeError> {
        // @TODO String conversion
        match self {
            LuaValue::Int(lhs) => match other {
                LuaValue::Int(rhs) => Ok(LuaValue::Int(lhs.wrapping_add(*rhs))),
                LuaValue::Float(rhs) => Ok(LuaValue::Float((*lhs as FloatType) + *rhs)),
                _ => Err(RuntimeError::InvalidArith),
            },
            LuaValue::Float(lhs) => match other {
                LuaValue::Int(rhs) => Ok(LuaValue::Float(*lhs + *rhs as FloatType)),
                LuaValue::Float(rhs) => Ok(LuaValue::Float(lhs + rhs)),
                _ => Err(RuntimeError::InvalidArith),
            },
            _ => Err(RuntimeError::InvalidArith),
        }
    }
    pub fn sub(&self, other: &LuaValue) -> Result<LuaValue, RuntimeError> {
        // @TODO String conversion
        match self {
            LuaValue::Int(lhs) => match other {
                LuaValue::Int(rhs) => Ok(LuaValue::Int(lhs.wrapping_sub(*rhs))),
                LuaValue::Float(rhs) => Ok(LuaValue::Float((*lhs as FloatType) - *rhs)),
                _ => Err(RuntimeError::InvalidArith),
            },
            LuaValue::Float(lhs) => match other {
                LuaValue::Int(rhs) => Ok(LuaValue::Float(*lhs - *rhs as FloatType)),
                LuaValue::Float(rhs) => Ok(LuaValue::Float(lhs - rhs)),
                _ => Err(RuntimeError::InvalidArith),
            },
            _ => Err(RuntimeError::InvalidArith),
        }
    }
    pub fn mul(&self, other: &LuaValue) -> Result<LuaValue, RuntimeError> {
        // @TODO String conversion
        match self {
            LuaValue::Int(lhs) => match other {
                LuaValue::Int(rhs) => Ok(LuaValue::Int(lhs.wrapping_mul(*rhs))),
                LuaValue::Float(rhs) => Ok(LuaValue::Float((*lhs as FloatType) * *rhs)),
                _ => Err(RuntimeError::InvalidArith),
            },
            LuaValue::Float(lhs) => match other {
                LuaValue::Int(rhs) => Ok(LuaValue::Float(*lhs * *rhs as FloatType)),
                LuaValue::Float(rhs) => Ok(LuaValue::Float(lhs * rhs)),
                _ => Err(RuntimeError::InvalidArith),
            },
            _ => Err(RuntimeError::InvalidArith),
        }
    }
    pub fn div(&self, other: &LuaValue) -> Result<LuaValue, RuntimeError> {
        // @TODO String conversion
        match self {
            LuaValue::Int(lhs) => match other {
                LuaValue::Int(rhs) => Ok(LuaValue::Float(*lhs as FloatType / *rhs as FloatType)),
                LuaValue::Float(rhs) => Ok(LuaValue::Float((*lhs as FloatType) / *rhs)),
                _ => Err(RuntimeError::InvalidArith),
            },
            LuaValue::Float(lhs) => match other {
                LuaValue::Int(rhs) => Ok(LuaValue::Float(*lhs / *rhs as FloatType)),
                LuaValue::Float(rhs) => Ok(LuaValue::Float(lhs / rhs)),
                _ => Err(RuntimeError::InvalidArith),
            },
            _ => Err(RuntimeError::InvalidArith),
        }
    }
    pub fn fdiv(&self, other: &LuaValue) -> Result<LuaValue, RuntimeError> {
        // @TODO String conversion
        match self {
            LuaValue::Int(lhs) => match other {
                LuaValue::Int(rhs) => Ok(LuaValue::Float(
                    (*lhs as FloatType / *rhs as FloatType).floor(),
                )),
                LuaValue::Float(rhs) => Ok(LuaValue::Float(((*lhs as FloatType) / *rhs).floor())),
                _ => Err(RuntimeError::InvalidArith),
            },
            LuaValue::Float(lhs) => match other {
                LuaValue::Int(rhs) => Ok(LuaValue::Float((*lhs / *rhs as FloatType).floor())),
                LuaValue::Float(rhs) => Ok(LuaValue::Float((lhs / rhs).floor())),
                _ => Err(RuntimeError::InvalidArith),
            },
            _ => Err(RuntimeError::InvalidArith),
        }
    }
    pub fn modu(&self, other: &LuaValue) -> Result<LuaValue, RuntimeError> {
        // @TODO String conversion
        match self {
            LuaValue::Int(lhs) => match other {
                LuaValue::Int(rhs) => Ok(LuaValue::Int(lhs % rhs)),
                LuaValue::Float(rhs) => Ok(LuaValue::Float((*lhs as FloatType) % *rhs)),
                _ => Err(RuntimeError::InvalidArith),
            },
            LuaValue::Float(lhs) => match other {
                LuaValue::Int(rhs) => Ok(LuaValue::Float(*lhs % *rhs as FloatType)),
                LuaValue::Float(rhs) => Ok(LuaValue::Float(lhs % rhs)),
                _ => Err(RuntimeError::InvalidArith),
            },
            _ => Err(RuntimeError::InvalidArith),
        }
    }
    pub fn pow(&self, other: &LuaValue) -> Result<LuaValue, RuntimeError> {
        // @TODO String conversion
        match self {
            LuaValue::Int(lhs) => match other {
                LuaValue::Int(rhs) => {
                    Ok(LuaValue::Float((*lhs as FloatType).powf(*rhs as FloatType)))
                }
                LuaValue::Float(rhs) => Ok(LuaValue::Float((*lhs as FloatType).powf(*rhs))),
                _ => Err(RuntimeError::InvalidArith),
            },
            LuaValue::Float(lhs) => match other {
                LuaValue::Int(rhs) => Ok(LuaValue::Float(lhs.powf(*rhs as FloatType))),
                LuaValue::Float(rhs) => Ok(LuaValue::Float(lhs.powf(*rhs))),
                _ => Err(RuntimeError::InvalidArith),
            },
            _ => Err(RuntimeError::InvalidArith),
        }
    }
    pub fn neg(&self) -> Result<LuaValue, RuntimeError> {
        // @TODO String conversion
        match self {
            LuaValue::Int(lhs) => Ok(LuaValue::Int(-*lhs)),
            LuaValue::Float(lhs) => Ok(LuaValue::Float(-*lhs)),
            _ => Err(RuntimeError::InvalidArith),
        }
    }

    /// convert float to int, if float has exact integer representation.
    pub fn strict_to_int(&self) -> Result<IntType, RuntimeError> {
        match self {
            LuaValue::Int(i) => Ok(*i),
            LuaValue::Float(f) => {
                if f.fract() == 0.0 {
                    Ok(*f as IntType)
                } else {
                    Err(RuntimeError::InvalidArith)
                }
            }
            _ => Err(RuntimeError::InvalidArith),
        }
    }
    pub fn bit_and(&self, other: &LuaValue) -> Result<LuaValue, RuntimeError> {
        let lhs = self.strict_to_int()?;
        let rhs = other.strict_to_int()?;
        let result = lhs & rhs;
        Ok(LuaValue::Int(result))
    }
    pub fn bit_or(&self, other: &LuaValue) -> Result<LuaValue, RuntimeError> {
        let lhs = self.strict_to_int()?;
        let rhs = other.strict_to_int()?;
        let result = lhs | rhs;
        Ok(LuaValue::Int(result))
    }
    pub fn bit_xor(&self, other: &LuaValue) -> Result<LuaValue, RuntimeError> {
        let lhs = self.strict_to_int()?;
        let rhs = other.strict_to_int()?;
        let result = lhs ^ rhs;
        Ok(LuaValue::Int(result))
    }
    pub fn bit_lshift(&self, other: &LuaValue) -> Result<LuaValue, RuntimeError> {
        let lhs = self.strict_to_int()?;
        let rhs = other.strict_to_int()?;
        let result = lhs << rhs;
        Ok(LuaValue::Int(result))
    }
    pub fn bit_rshift(&self, other: &LuaValue) -> Result<LuaValue, RuntimeError> {
        let lhs = self.strict_to_int()?;
        let rhs = other.strict_to_int()?;
        let result = lhs >> rhs;
        Ok(LuaValue::Int(result))
    }
    pub fn bit_not(&self) -> Result<LuaValue, RuntimeError> {
        let lhs = self.strict_to_int()?;
        let result = lhs.reverse_bits();
        Ok(LuaValue::Int(result))
    }

    pub fn eq(&self, other: &LuaValue) -> bool {
        match (self, other) {
            (LuaValue::Nil, LuaValue::Nil) => true,
            (LuaValue::Boolean(a), LuaValue::Boolean(b)) => a == b,
            (LuaValue::Int(a), LuaValue::Int(b)) => a == b,
            (LuaValue::Float(a), LuaValue::Float(b)) => a == b,
            (LuaValue::String(a), LuaValue::String(b)) => a == b,
            _ => false,
        }
    }
    pub fn eq_raw(&self, other: &LuaValue) -> bool {
        match (self, other) {
            (LuaValue::Nil, LuaValue::Nil) => true,
            (LuaValue::Boolean(a), LuaValue::Boolean(b)) => a == b,
            (LuaValue::Int(a), LuaValue::Int(b)) => a == b,
            (LuaValue::Float(a), LuaValue::Float(b)) => a == b,
            (LuaValue::String(a), LuaValue::String(b)) => a == b,
            _ => false,
        }
    }
    pub fn lt(&self, other: &LuaValue) -> Result<bool, RuntimeError> {
        match self {
            LuaValue::Int(lhs) => match other {
                LuaValue::Int(rhs) => Ok(lhs < rhs),
                LuaValue::Float(rhs) => Ok((*lhs as FloatType) < *rhs),
                _ => Err(RuntimeError::InvalidArith),
            },
            LuaValue::Float(lhs) => match other {
                LuaValue::Int(rhs) => Ok(*lhs < *rhs as FloatType),
                LuaValue::Float(rhs) => Ok(lhs < rhs),
                _ => Err(RuntimeError::InvalidArith),
            },
            _ => Err(RuntimeError::InvalidArith),
        }
    }

    pub fn len(&self) -> Result<IntType, RuntimeError> {
        // @TODO
        match self {
            LuaValue::String(s) => Ok(s.len() as IntType),
            LuaValue::Table(t) => Ok(t.borrow().map.len() as IntType),
            _ => Err(RuntimeError::InvalidArith),
        }
    }
    pub fn len_raw(&self) -> Result<IntType, RuntimeError> {
        // @TODO
        match self {
            LuaValue::String(s) => Ok(s.len() as IntType),
            LuaValue::Table(t) => Ok(t.borrow().map.len() as IntType),
            _ => Err(RuntimeError::InvalidArith),
        }
    }
    pub fn concat(&self, other: &LuaValue) -> Result<LuaValue, RuntimeError> {
        // @TODO
        let str = format!("{}{}", self, other);
        Ok(LuaValue::String(str))
    }
    pub fn not(&self) -> LuaValue {
        LuaValue::Boolean(!self.to_bool())
    }

    pub fn table_index_get(&self, idx: LuaValue) -> Result<LuaValue, RuntimeError> {
        match self {
            LuaValue::Table(table) => table.borrow().table_index_get(idx),
            _ => Ok(LuaValue::Nil),
        }
    }
    pub fn is_nil(&self) -> bool {
        match self {
            LuaValue::Nil => true,
            _ => false,
        }
    }
    pub fn table_index_set(&mut self, idx: LuaValue, value: LuaValue) -> Result<(), RuntimeError> {
        match self {
            LuaValue::Table(table) => table.borrow_mut().table_index_set(idx, value),
            _ => Err(RuntimeError::InvalidArith),
        }
    }
    pub fn table_index_init(&mut self, idx: LuaValue, value: LuaValue) -> Result<(), RuntimeError> {
        match self {
            LuaValue::Table(table) => table.borrow_mut().table_index_init(idx, value),
            _ => Err(RuntimeError::InvalidArith),
        }
    }

    pub fn as_key(&self) -> Result<String, RuntimeError> {
        Ok(match self {
            LuaValue::Nil => "nil".to_string(),
            LuaValue::Boolean(b) => b.to_string(),
            LuaValue::Int(n) => n.to_string(),
            LuaValue::Float(n) => n.to_string(),
            LuaValue::String(s) => format!("\"{}\"", s),
            LuaValue::Table(_) => "table".to_string(),
            LuaValue::Function(_) => "function".to_string(),
            LuaValue::UserData(_) => "userdata".to_string(),
            LuaValue::Thread(_) => "thread".to_string(),
        })
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
        LuaValue::Int(n)
    }
}
impl From<FloatType> for LuaValue {
    fn from(n: FloatType) -> Self {
        LuaValue::Float(n)
    }
}
impl From<String> for LuaValue {
    fn from(s: String) -> Self {
        LuaValue::String(s)
    }
}
impl From<&str> for LuaValue {
    fn from(s: &str) -> Self {
        LuaValue::String(s.to_string())
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
