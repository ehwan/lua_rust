use std::cell::RefCell;
use std::rc::Rc;

use crate::LuaFunction;
use crate::LuaNumber;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;
use crate::Stack;

/// init math module
pub fn init() -> Result<LuaValue, RuntimeError> {
    let mut math = LuaTable::new();
    #[cfg(feature = "32bit")]
    {
        math.map.insert("pi".into(), std::f32::consts::PI.into());
        math.map.insert("huge".into(), std::f32::INFINITY.into());
        math.map.insert("maxinteger".into(), std::i32::MAX.into());
    }
    #[cfg(not(feature = "32bit"))]
    {
        math.map.insert("pi".into(), std::f64::consts::PI.into());
        math.map.insert("huge".into(), std::f64::INFINITY.into());
        math.map.insert("maxinteger".into(), std::i64::MAX.into());
    }
    math.map
        .insert("abs".into(), LuaFunction::from_func(abs).into());
    math.map
        .insert("acos".into(), LuaFunction::from_func(acos).into());
    math.map
        .insert("asin".into(), LuaFunction::from_func(asin).into());
    math.map
        .insert("ceil".into(), LuaFunction::from_func(ceil).into());
    math.map
        .insert("floor".into(), LuaFunction::from_func(floor).into());
    math.map
        .insert("cos".into(), LuaFunction::from_func(cos).into());
    math.map
        .insert("sin".into(), LuaFunction::from_func(sin).into());
    math.map
        .insert("deg".into(), LuaFunction::from_func(deg).into());
    math.map
        .insert("rad".into(), LuaFunction::from_func(rad).into());
    math.map
        .insert("exp".into(), LuaFunction::from_func(exp).into());
    math.map
        .insert("log".into(), LuaFunction::from_func(log).into());
    math.map
        .insert("sqrt".into(), LuaFunction::from_func(sqrt).into());
    math.map
        .insert("type".into(), LuaFunction::from_func(type_).into());
    math.map
        .insert("tointeger".into(), LuaFunction::from_func(tointeger).into());
    math.map
        .insert("ult".into(), LuaFunction::from_func(ult).into());
    Ok(LuaValue::Table(Rc::new(RefCell::new(math))))
}

// @TODO
// atan
// fmod
// modf
// random
// randomseed

pub fn abs(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    match args.into_iter().next() {
        Some(val) => match val.try_to_number() {
            Some(num) => {
                let abs = num.abs();
                Ok(vec![abs.into()])
            }
            None => Err(RuntimeError::NotNumber),
        },
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn acos(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    match args.into_iter().next() {
        Some(val) => match val.try_to_number() {
            Some(num) => Ok(vec![num.to_float().acos().into()]),
            None => Err(RuntimeError::NotNumber),
        },
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn asin(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    match args.into_iter().next() {
        Some(val) => match val.try_to_number() {
            Some(num) => Ok(vec![num.to_float().asin().into()]),
            None => Err(RuntimeError::NotNumber),
        },
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn ceil(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    match args.into_iter().next() {
        Some(val) => match val.try_to_number() {
            Some(num) => Ok(vec![num.ceil().into()]),
            None => Err(RuntimeError::NotNumber),
        },
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn floor(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    match args.into_iter().next() {
        Some(val) => match val.try_to_number() {
            Some(num) => Ok(vec![num.floor().into()]),
            None => Err(RuntimeError::NotNumber),
        },
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn cos(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    match args.into_iter().next() {
        Some(val) => match val.try_to_number() {
            Some(num) => Ok(vec![num.cos().into()]),
            None => Err(RuntimeError::NotNumber),
        },
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn sin(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    match args.into_iter().next() {
        Some(val) => match val.try_to_number() {
            Some(num) => Ok(vec![num.sin().into()]),
            None => Err(RuntimeError::NotNumber),
        },
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn deg(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    match args.into_iter().next() {
        Some(val) => match val.try_to_number() {
            Some(num) => Ok(vec![num.deg().into()]),
            None => Err(RuntimeError::NotNumber),
        },
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn rad(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    match args.into_iter().next() {
        Some(val) => match val.try_to_number() {
            Some(num) => Ok(vec![num.rad().into()]),
            None => Err(RuntimeError::NotNumber),
        },
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn exp(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    match args.into_iter().next() {
        Some(val) => match val.try_to_number() {
            Some(num) => Ok(vec![num.exp().into()]),
            None => Err(RuntimeError::NotNumber),
        },
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn log(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let x = match it.next() {
        Some(val) => match val.try_to_number() {
            Some(num) => num,
            None => return Err(RuntimeError::NotNumber),
        },
        _ => return Err(RuntimeError::NotNumber),
    };
    match it.next() {
        Some(base) => match base.try_to_number() {
            Some(base) => Ok(vec![x.log(base).into()]),
            None => Err(RuntimeError::NotNumber),
        },
        None => {
            // default to e
            Ok(vec![x.ln().into()])
        }
    }
}

pub fn sqrt(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    match args.into_iter().next() {
        Some(val) => match val.try_to_number() {
            Some(num) => Ok(vec![num.sqrt().into()]),
            None => Err(RuntimeError::NotNumber),
        },
        _ => Err(RuntimeError::NotNumber),
    }
}

pub fn type_(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let ret = match args.into_iter().next() {
        Some(LuaValue::Number(num)) => match num {
            LuaNumber::Int(_) => "integer".into(),
            LuaNumber::Float(_) => "float".into(),
        },
        _ => ().into(),
    };

    Ok(vec![ret])
}
pub fn tointeger(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let ret = match args.into_iter().next() {
        Some(val) => match val.try_to_int() {
            Some(num) => num.into(),
            None => LuaValue::Nil,
        },
        _ => LuaValue::Nil,
    };

    Ok(vec![ret])
}
pub fn ult(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let a = match it.next() {
        Some(val) => match val.try_to_int() {
            Some(num) => num,
            None => return Err(RuntimeError::NotInteger),
        },
        _ => return Err(RuntimeError::NotInteger),
    };
    let b = match it.next() {
        Some(val) => match val.try_to_int() {
            Some(num) => num,
            None => return Err(RuntimeError::NotInteger),
        },
        _ => return Err(RuntimeError::NotInteger),
    };

    #[cfg(feature = "32bit")]
    let res = (a as u32) < (b as u32);
    #[cfg(not(feature = "32bit"))]
    let res = (a as u64) < (b as u64);

    Ok(vec![res.into()])
}
