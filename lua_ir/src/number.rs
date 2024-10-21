use crate::FloatType;
use crate::IntType;
use crate::RuntimeError;

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
    pub fn try_to_int(&self) -> Result<IntType, RuntimeError> {
        match self {
            LuaNumber::Int(i) => Ok(*i),
            LuaNumber::Float(f) => {
                if f.fract() == 0.0 {
                    Ok(*f as IntType)
                } else {
                    Err(RuntimeError::NoIntegerRepresentation)
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

    pub fn abs(self) -> LuaNumber {
        match self {
            LuaNumber::Int(i) => LuaNumber::Int(i.abs()),
            LuaNumber::Float(f) => LuaNumber::Float(f.abs()),
        }
    }
    pub fn ceil(self) -> LuaNumber {
        match self {
            LuaNumber::Int(i) => LuaNumber::Int(i),
            LuaNumber::Float(f) => LuaNumber::Float(f.ceil()),
        }
    }
    pub fn floor(self) -> LuaNumber {
        match self {
            LuaNumber::Int(i) => LuaNumber::Int(i),
            LuaNumber::Float(f) => LuaNumber::Float(f.floor()),
        }
    }
    pub fn cos(self) -> LuaNumber {
        match self {
            LuaNumber::Int(i) => LuaNumber::Float((i as FloatType).cos()),
            LuaNumber::Float(f) => LuaNumber::Float(f.cos()),
        }
    }
    pub fn sin(self) -> LuaNumber {
        match self {
            LuaNumber::Int(i) => LuaNumber::Float((i as FloatType).sin()),
            LuaNumber::Float(f) => LuaNumber::Float(f.sin()),
        }
    }

    pub fn deg(self) -> LuaNumber {
        match self {
            LuaNumber::Int(i) => LuaNumber::Float((i as FloatType).to_degrees()),
            LuaNumber::Float(f) => LuaNumber::Float(f.to_degrees()),
        }
    }
    pub fn rad(self) -> LuaNumber {
        match self {
            LuaNumber::Int(i) => LuaNumber::Float((i as FloatType).to_radians()),
            LuaNumber::Float(f) => LuaNumber::Float(f.to_radians()),
        }
    }
    pub fn exp(self) -> LuaNumber {
        match self {
            LuaNumber::Int(i) => LuaNumber::Float((i as FloatType).exp()),
            LuaNumber::Float(f) => LuaNumber::Float(f.exp()),
        }
    }
    pub fn log(self, base: LuaNumber) -> LuaNumber {
        let base = base.to_float();
        match self {
            LuaNumber::Int(i) => LuaNumber::Float((i as FloatType).log(base)),
            LuaNumber::Float(f) => LuaNumber::Float(f.log(base)),
        }
    }
    pub fn ln(self) -> LuaNumber {
        match self {
            LuaNumber::Int(i) => LuaNumber::Float((i as FloatType).ln()),
            LuaNumber::Float(f) => LuaNumber::Float(f.ln()),
        }
    }
    pub fn sqrt(self) -> LuaNumber {
        match self {
            LuaNumber::Int(i) => LuaNumber::Float((i as FloatType).sqrt()),
            LuaNumber::Float(f) => LuaNumber::Float(f.sqrt()),
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
                Ok(b) => *a == b,
                Err(_) => false,
            },
            (LuaNumber::Float(_), LuaNumber::Int(b)) => match self.try_to_int() {
                Ok(a) => a == *b,
                Err(_) => false,
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
        if let Ok(i) = self.try_to_int() {
            i.hash(state);
            return;
        } else {
            match self {
                LuaNumber::Int(i) => i.hash(state),
                // @TODO this is not a good way to hash a float
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
