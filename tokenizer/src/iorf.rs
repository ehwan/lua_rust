use crate::FloatType;
use crate::IntType;

/// Lua's numeric representation, can be either integer or float.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum IntOrFloat {
    Int(IntType),
    Float(FloatType),
}
impl IntOrFloat {
    pub fn to_float(&self) -> FloatType {
        match self {
            IntOrFloat::Int(int) => *int as FloatType,
            IntOrFloat::Float(float) => *float,
        }
    }
    /// Convert to integer, if it's a float, it will be truncated.
    pub fn to_int(&self) -> IntType {
        match self {
            IntOrFloat::Int(int) => *int,
            IntOrFloat::Float(float) => *float as IntType,
        }
    }
}
impl From<IntType> for IntOrFloat {
    fn from(int: IntType) -> Self {
        IntOrFloat::Int(int)
    }
}
impl From<FloatType> for IntOrFloat {
    fn from(float: FloatType) -> Self {
        IntOrFloat::Float(float)
    }
}
impl Into<FloatType> for IntOrFloat {
    fn into(self) -> FloatType {
        self.to_float()
    }
}
impl Into<IntType> for IntOrFloat {
    fn into(self) -> IntType {
        self.to_int()
    }
}
impl std::fmt::Display for IntOrFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntOrFloat::Int(int) => {
                write!(f, "{}", int)
            }
            IntOrFloat::Float(float) => {
                write!(f, "{}", float)
            }
        }
    }
}

impl std::ops::Add<IntType> for IntOrFloat {
    type Output = Self;

    fn add(self, rhs: IntType) -> Self::Output {
        match self {
            IntOrFloat::Int(x) => match x.checked_add(rhs) {
                Some(sum) => IntOrFloat::Int(sum),
                None => IntOrFloat::Float(x as FloatType + rhs as FloatType),
            },
            IntOrFloat::Float(x) => IntOrFloat::Float(x + rhs as FloatType),
        }
    }
}
impl std::ops::AddAssign<IntType> for IntOrFloat {
    fn add_assign(&mut self, rhs: IntType) {
        *self = *self + rhs;
    }
}
impl std::ops::Add<FloatType> for IntOrFloat {
    type Output = Self;

    fn add(self, rhs: FloatType) -> Self::Output {
        match self {
            IntOrFloat::Int(x) => IntOrFloat::Float(x as FloatType + rhs),
            IntOrFloat::Float(x) => IntOrFloat::Float(x + rhs),
        }
    }
}
impl std::ops::AddAssign<FloatType> for IntOrFloat {
    fn add_assign(&mut self, rhs: FloatType) {
        *self = *self + rhs;
    }
}
impl std::ops::Mul<IntType> for IntOrFloat {
    type Output = Self;

    fn mul(self, rhs: IntType) -> Self::Output {
        match self {
            IntOrFloat::Int(x) => match x.checked_mul(rhs) {
                Some(product) => IntOrFloat::Int(product),
                None => IntOrFloat::Float(x as FloatType * rhs as FloatType),
            },
            IntOrFloat::Float(x) => IntOrFloat::Float(x * rhs as FloatType),
        }
    }
}
impl std::ops::MulAssign<IntType> for IntOrFloat {
    fn mul_assign(&mut self, rhs: IntType) {
        *self = *self * rhs;
    }
}
impl std::ops::Mul<FloatType> for IntOrFloat {
    type Output = Self;

    fn mul(self, rhs: FloatType) -> Self::Output {
        match self {
            IntOrFloat::Int(x) => IntOrFloat::Float(x as FloatType * rhs),
            IntOrFloat::Float(x) => IntOrFloat::Float(x * rhs),
        }
    }
}
impl std::ops::MulAssign<FloatType> for IntOrFloat {
    fn mul_assign(&mut self, rhs: FloatType) {
        *self = *self * rhs;
    }
}

impl std::ops::Add for IntOrFloat {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        match rhs {
            IntOrFloat::Int(x) => self + x,
            IntOrFloat::Float(x) => self + x,
        }
    }
}
impl std::ops::AddAssign for IntOrFloat {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl std::ops::Mul for IntOrFloat {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match rhs {
            IntOrFloat::Int(x) => self * x,
            IntOrFloat::Float(x) => self * x,
        }
    }
}
impl std::ops::MulAssign for IntOrFloat {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl std::ops::Neg for IntOrFloat {
    type Output = Self;
    fn neg(self) -> Self {
        match self {
            IntOrFloat::Int(x) => IntOrFloat::Int(-x),
            IntOrFloat::Float(x) => IntOrFloat::Float(-x),
        }
    }
}
