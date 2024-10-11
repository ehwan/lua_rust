use std::cell::RefCell;
use std::rc::Rc;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

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
        .insert("atan".into(), LuaFunction::from_func(atan).into());
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
    math.map
        .insert("modf".into(), LuaFunction::from_func(modf).into());
    math.map
        .insert("fmod".into(), LuaFunction::from_func(fmod).into());

    math.map
        .insert("random".into(), LuaFunction::from_func(random).into());

    math.map.insert(
        "randomseed".into(),
        LuaFunction::from_func(randomseed).into(),
    );
    Ok(LuaValue::Table(Rc::new(RefCell::new(math))))
}

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
pub fn atan(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let ret = match args.len() {
        0 => return Err(RuntimeError::NotNumber),
        1 => match args.into_iter().next().unwrap().try_to_number() {
            Some(num) => num.to_float().atan2(1.0).into(),
            None => return Err(RuntimeError::NotNumber),
        },
        _ => {
            let mut it = args.into_iter();
            let y = match it.next().unwrap().try_to_number() {
                Some(num) => num.to_float(),
                None => return Err(RuntimeError::NotNumber),
            };
            let x = match it.next().unwrap().try_to_number() {
                Some(num) => num.to_float(),
                None => return Err(RuntimeError::NotNumber),
            };
            y.atan2(x).into()
        }
    };
    Ok(vec![ret])
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

pub fn random(stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let rand = match args.len() {
        0 => {
            // [0,1)
            stack.rng.gen_range(0.0..1.0).into()
        }
        1 => {
            // [1,num]
            match args.into_iter().next().unwrap().try_to_int() {
                Some(num) => stack.rng.gen_range(1..=num).into(),
                None => return Err(RuntimeError::NotInteger),
            }
        }
        _ => {
            // [m, n]
            let mut it = args.into_iter();
            let m = it.next().unwrap().try_to_int();
            let n = it.next().unwrap().try_to_int();

            match (m, n) {
                (Some(m), Some(n)) => stack.rng.gen_range(m..=n).into(),
                _ => return Err(RuntimeError::NotInteger),
            }
        }
    };

    Ok(vec![rand])
}

pub fn randomseed(stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    match args.len() {
        0 => {
            stack.rng = StdRng::from_entropy();
        }
        1 => {
            let seed = match args.into_iter().next().unwrap().try_to_int() {
                Some(num) => num,
                None => return Err(RuntimeError::NotInteger),
            };
            stack.rng = StdRng::seed_from_u64(seed as u64);
        }
        _ => {
            let mut it = args.into_iter();
            let seed1 = it.next().unwrap().try_to_int();
            let seed2 = it.next().unwrap().try_to_int();
            match (seed1, seed2) {
                (Some(seed1), Some(seed2)) => {
                    let seed = (seed1.rotate_left(32) ^ seed2) as u64;
                    stack.rng = StdRng::seed_from_u64(seed as u64);
                }
                _ => return Err(RuntimeError::NotInteger),
            }
        }
    }
    Ok(vec![])
}

pub fn modf(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    match args.into_iter().next() {
        Some(LuaValue::Number(val)) => match val {
            LuaNumber::Int(i) => Ok(vec![i.into(), 0.0.into()]),
            LuaNumber::Float(f) => {
                let fract = f.fract();
                let int = f - fract;
                Ok(vec![int.into(), fract.into()])
            }
        },
        _ => Err(RuntimeError::NotNumber),
    }
}

pub fn fmod(_stack: &mut Stack, args: Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> {
    let mut it = args.into_iter();
    let x = match it.next() {
        Some(val) => match val.try_to_number() {
            Some(num) => num,
            None => return Err(RuntimeError::NotNumber),
        },
        _ => return Err(RuntimeError::NotNumber),
    };
    let y = match it.next() {
        Some(val) => match val.try_to_number() {
            Some(num) => num,
            None => return Err(RuntimeError::NotNumber),
        },
        _ => return Err(RuntimeError::NotNumber),
    };

    Ok(vec![(x % y).into()])
}
