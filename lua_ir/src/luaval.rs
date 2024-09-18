use crate::FloatType;
use crate::IntType;
use crate::RuntimeError;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum LuaValue {
    Nil,
    Boolean(bool),
    Int(IntType),
    Float(FloatType),
    String(String),
    Table(LuaTable),
    Function(LuaFunction),
    UserData(LuaUserData),
    Thread(LuaThread),

    Ref(LuaRef),
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
                write!(f, "table {:p}", Rc::as_ptr(&t.internal))
            }
            LuaValue::Function(func) => match func {
                LuaFunction::LuaFunc(internal) => {
                    write!(f, "function{:p}", Rc::as_ptr(&internal))
                }
                LuaFunction::RustFunc(_) => {
                    write!(f, "built-in function")
                }
            },
            LuaValue::UserData(_) => write!(f, "userdata"),
            LuaValue::Thread(_) => write!(f, "thread"),
            LuaValue::Ref(r) => write!(f, "{}", r.value.borrow()),
        }
    }
}

impl LuaValue {
    pub fn new_global() -> Self {
        // @TODO
        let mut table_map: HashMap<String, LuaValue> = HashMap::new();
        table_map.insert(
            "\"print\"".to_string(),
            LuaFunction::from_func(|args| {
                for (idx, arg) in args.into_iter().enumerate() {
                    if idx > 0 {
                        print!("\t");
                    }
                    print!("{}", arg);
                }
                println!();
                Ok(vec![])
            })
            .into(),
        );

        LuaValue::Table(LuaTable {
            internal: Rc::new(RefCell::new(LuaTableInternal { map: table_map })),
        })
    }
}

impl LuaValue {
    pub fn deref(&self) -> LuaValue {
        match self {
            LuaValue::Ref(r) => LuaValue::deref(&r.value.borrow()),
            _ => self.clone(),
        }
    }
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
            // LuaValue::Table(t) => Ok(t.internal.borrow().map.len()),
            _ => Err(RuntimeError::InvalidArith),
        }
    }
    pub fn concat(&self, other: &LuaValue) -> Result<LuaValue, RuntimeError> {
        // @TODO
        match self {
            LuaValue::String(lhs) => match other {
                LuaValue::String(rhs) => Ok(LuaValue::String(lhs.clone() + rhs)),
                _ => Err(RuntimeError::InvalidArith),
            },
            _ => Err(RuntimeError::InvalidArith),
        }
    }
    pub fn not(&self) -> LuaValue {
        LuaValue::Boolean(!self.to_bool())
    }

    pub fn table_index_get(&self, idx: LuaValue) -> Result<LuaValue, RuntimeError> {
        match self {
            LuaValue::Table(table) => table.table_index_get(idx),
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
            LuaValue::Table(table) => table.table_index_set(idx, value),
            _ => Err(RuntimeError::InvalidArith),
        }
    }
    pub fn table_index_init(&mut self, idx: LuaValue, value: LuaValue) -> Result<(), RuntimeError> {
        match self {
            LuaValue::Table(table) => table.table_index_init(idx, value),
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
            LuaValue::Ref(_) => "ref".to_string(),
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
        LuaValue::Table(t)
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
impl From<LuaRef> for LuaValue {
    fn from(r: LuaRef) -> Self {
        LuaValue::Ref(r)
    }
}

#[derive(Debug, Clone)]
pub struct LuaTable {
    pub internal: Rc<RefCell<LuaTableInternal>>,
}
impl LuaTable {
    pub fn with_capacity(capacity: usize) -> Self {
        LuaTable {
            internal: Rc::new(RefCell::new(LuaTableInternal {
                map: HashMap::with_capacity(capacity),
            })),
        }
    }

    pub fn table_index_get(&self, idx: LuaValue) -> Result<LuaValue, RuntimeError> {
        Ok(self
            .internal
            .borrow()
            .map
            .get(&idx.as_key()?)
            .unwrap_or(&LuaValue::Nil)
            .clone())
    }
    pub fn table_index_set(&mut self, idx: LuaValue, value: LuaValue) -> Result<(), RuntimeError> {
        if idx.is_nil() {
            return Ok(());
        }
        self.internal.borrow_mut().map.insert(idx.as_key()?, value);
        Ok(())
    }
    pub fn table_index_init(&mut self, idx: LuaValue, value: LuaValue) -> Result<(), RuntimeError> {
        if idx.is_nil() {
            return Ok(());
        }
        self.internal.borrow_mut().map.insert(idx.as_key()?, value);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LuaTableInternal {
    pub map: HashMap<String, LuaValue>,
}

#[derive(Debug, Clone)]
pub enum LuaFunction {
    LuaFunc(Rc<RefCell<LuaFunctionInternal>>),
    RustFunc(LuaFunctionRust),
}
impl LuaFunction {
    pub fn from_func<F: Fn(Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> + 'static>(
        func: F,
    ) -> LuaFunction {
        LuaFunction::RustFunc(LuaFunctionRust {
            func: Rc::new(func),
        })
    }
}
#[derive(Debug, Clone)]
pub struct LuaFunctionInternal {
    pub upvalues: Vec<LuaRef>,
    pub function_id: usize,
}

impl LuaFunctionInternal {
    pub fn upvalue(&self, i: usize) -> LuaRef {
        self.upvalues[i].clone()
    }
}

pub type RustFuncType = dyn Fn(Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError>;
#[derive(Clone)]
pub struct LuaFunctionRust {
    pub func: Rc<RustFuncType>,
}
impl std::fmt::Debug for LuaFunctionRust {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LuaFunctionRust")
    }
}

#[derive(Debug, Clone)]
pub struct LuaUserData {}

#[derive(Debug, Clone)]
pub struct LuaThread {}

#[derive(Debug, Clone)]
pub struct LuaRef {
    pub value: Rc<RefCell<LuaValue>>,
}

impl Default for LuaRef {
    fn default() -> Self {
        LuaRef {
            value: Rc::new(RefCell::new(LuaValue::Nil)),
        }
    }
}
