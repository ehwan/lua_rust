use crate::FloatType;
use crate::IntType;

#[derive(Debug, Clone, Copy)]
pub enum LuaNumber {
    Int(IntType),
    Float(FloatType),
}

impl LuaNumber {
    pub fn to_float(&self) -> FloatType {
        match self {
            LuaNumber::Int(i) => *i as FloatType,
            LuaNumber::Float(f) => *f,
        }
    }
    pub fn try_to_int(&self) -> Option<IntType> {
        match self {
            LuaNumber::Int(i) => Some(*i),
            LuaNumber::Float(f) => {
                if f.fract() == 0.0 {
                    Some(*f as IntType)
                } else {
                    None
                }
            }
        }
    }
    pub fn is_nan(&self) -> bool {
        match self {
            LuaNumber::Int(_) => false,
            LuaNumber::Float(f) => f.is_nan(),
        }
    }

    pub fn floor_div(self, rhs: Self) -> Self {
        match (self, rhs) {
            (LuaNumber::Int(a), LuaNumber::Int(b)) => {
                LuaNumber::Float((a as FloatType / b as FloatType).floor())
            }
            (LuaNumber::Int(a), LuaNumber::Float(b)) => {
                LuaNumber::Float((a as FloatType / b).floor())
            }
            (LuaNumber::Float(a), LuaNumber::Int(b)) => {
                LuaNumber::Float((a / b as FloatType).floor())
            }
            (LuaNumber::Float(a), LuaNumber::Float(b)) => LuaNumber::Float((a / b).floor()),
        }
    }
    pub fn pow(self, exp: Self) -> Self {
        match (self, exp) {
            (LuaNumber::Int(a), LuaNumber::Int(b)) => {
                LuaNumber::Float((a as FloatType).powf(b as FloatType))
            }
            (LuaNumber::Int(a), LuaNumber::Float(b)) => LuaNumber::Float((a as FloatType).powf(b)),
            (LuaNumber::Float(a), LuaNumber::Int(b)) => LuaNumber::Float(a.powf(b as FloatType)),
            (LuaNumber::Float(a), LuaNumber::Float(b)) => LuaNumber::Float(a.powf(b)),
        }
    }
}

impl From<IntType> for LuaNumber {
    fn from(i: IntType) -> Self {
        LuaNumber::Int(i)
    }
}
impl From<FloatType> for LuaNumber {
    fn from(f: FloatType) -> Self {
        LuaNumber::Float(f)
    }
}

impl std::fmt::Display for LuaNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaNumber::Int(i) => write!(f, "{}", i),
            LuaNumber::Float(i) => write!(f, "{}", i),
        }
    }
}

impl std::cmp::PartialEq for LuaNumber {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LuaNumber::Int(a), LuaNumber::Int(b)) => a == b,
            (LuaNumber::Int(a), LuaNumber::Float(_)) => match other.try_to_int() {
                Some(b) => *a == b,
                None => false,
            },
            (LuaNumber::Float(_), LuaNumber::Int(b)) => match self.try_to_int() {
                Some(a) => a == *b,
                None => false,
            },
            (LuaNumber::Float(a), LuaNumber::Float(b)) => a == b,
        }
    }
}
impl std::cmp::Eq for LuaNumber {}

impl std::cmp::PartialOrd for LuaNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl std::cmp::Ord for LuaNumber {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (LuaNumber::Int(a), LuaNumber::Int(b)) => a.cmp(b),
            (LuaNumber::Int(_), LuaNumber::Float(b)) => self
                .to_float()
                .partial_cmp(b)
                .unwrap_or(std::cmp::Ordering::Less),
            (LuaNumber::Float(a), LuaNumber::Int(_)) => a
                .partial_cmp(&other.to_float())
                .unwrap_or(std::cmp::Ordering::Less),
            (LuaNumber::Float(a), LuaNumber::Float(b)) => {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less)
            }
        }
    }
}

impl std::hash::Hash for LuaNumber {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if let Some(i) = self.try_to_int() {
            i.hash(state);
            return;
        } else {
            match self {
                LuaNumber::Int(i) => i.hash(state),
                LuaNumber::Float(f) => f.to_bits().hash(state),
            }
        }
    }
}

impl std::ops::Add for LuaNumber {
    type Output = LuaNumber;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (LuaNumber::Int(a), LuaNumber::Int(b)) => LuaNumber::Int(a.wrapping_add(b)),
            (LuaNumber::Int(a), LuaNumber::Float(b)) => LuaNumber::Float(a as FloatType + b),
            (LuaNumber::Float(a), LuaNumber::Int(b)) => LuaNumber::Float(a + b as FloatType),
            (LuaNumber::Float(a), LuaNumber::Float(b)) => LuaNumber::Float(a + b),
        }
    }
}

impl std::ops::Sub for LuaNumber {
    type Output = LuaNumber;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (LuaNumber::Int(a), LuaNumber::Int(b)) => LuaNumber::Int(a.wrapping_sub(b)),
            (LuaNumber::Int(a), LuaNumber::Float(b)) => LuaNumber::Float(a as FloatType - b),
            (LuaNumber::Float(a), LuaNumber::Int(b)) => LuaNumber::Float(a - b as FloatType),
            (LuaNumber::Float(a), LuaNumber::Float(b)) => LuaNumber::Float(a - b),
        }
    }
}
impl std::ops::Mul for LuaNumber {
    type Output = LuaNumber;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (LuaNumber::Int(a), LuaNumber::Int(b)) => LuaNumber::Int(a.wrapping_mul(b)),
            (LuaNumber::Int(a), LuaNumber::Float(b)) => LuaNumber::Float(a as FloatType * b),
            (LuaNumber::Float(a), LuaNumber::Int(b)) => LuaNumber::Float(a * b as FloatType),
            (LuaNumber::Float(a), LuaNumber::Float(b)) => LuaNumber::Float(a * b),
        }
    }
}
impl std::ops::Div for LuaNumber {
    type Output = LuaNumber;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (LuaNumber::Int(a), LuaNumber::Int(b)) => {
                LuaNumber::Float(a as FloatType / b as FloatType)
            }
            (LuaNumber::Int(a), LuaNumber::Float(b)) => LuaNumber::Float(a as FloatType / b),
            (LuaNumber::Float(a), LuaNumber::Int(b)) => LuaNumber::Float(a / b as FloatType),
            (LuaNumber::Float(a), LuaNumber::Float(b)) => LuaNumber::Float(a / b),
        }
    }
}

impl std::ops::Rem for LuaNumber {
    type Output = LuaNumber;
    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (LuaNumber::Int(a), LuaNumber::Int(b)) => LuaNumber::Int(a % b),
            (LuaNumber::Int(a), LuaNumber::Float(b)) => LuaNumber::Float(a as FloatType % b),
            (LuaNumber::Float(a), LuaNumber::Int(b)) => LuaNumber::Float(a % b as FloatType),
            (LuaNumber::Float(a), LuaNumber::Float(b)) => LuaNumber::Float(a % b),
        }
    }
}

impl std::ops::Neg for LuaNumber {
    type Output = LuaNumber;
    fn neg(self) -> Self::Output {
        match self {
            LuaNumber::Int(i) => LuaNumber::Int(-i),
            LuaNumber::Float(f) => LuaNumber::Float(-f),
        }
    }
}
