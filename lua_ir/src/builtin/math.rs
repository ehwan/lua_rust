use std::cell::RefCell;
use std::rc::Rc;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use crate::Chunk;
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
        math.insert("pi".into(), std::f32::consts::PI.into());
        math.nsert("huge".into(), std::f32::INFINITY.into());
        math.nsert("mininteger".into(), std::i32::MIN.into());
        math.nsert("maxinteger".into(), std::i32::MAX.into());
    }
    #[cfg(not(feature = "32bit"))]
    {
        math.insert("pi".into(), std::f64::consts::PI.into());
        math.insert("huge".into(), std::f64::INFINITY.into());
        math.insert("mininteger".into(), std::i64::MIN.into());
        math.insert("maxinteger".into(), std::i64::MAX.into());
    }
    math.insert("abs".into(), LuaFunction::from_func(abs).into());
    math.insert("acos".into(), LuaFunction::from_func(acos).into());
    math.insert("asin".into(), LuaFunction::from_func(asin).into());
    math.insert("atan".into(), LuaFunction::from_func(atan).into());
    math.insert("ceil".into(), LuaFunction::from_func(ceil).into());
    math.insert("floor".into(), LuaFunction::from_func(floor).into());
    math.insert("cos".into(), LuaFunction::from_func(cos).into());
    math.insert("sin".into(), LuaFunction::from_func(sin).into());
    math.insert("deg".into(), LuaFunction::from_func(deg).into());
    math.insert("rad".into(), LuaFunction::from_func(rad).into());
    math.insert("exp".into(), LuaFunction::from_func(exp).into());
    math.insert("log".into(), LuaFunction::from_func(log).into());
    math.insert("sqrt".into(), LuaFunction::from_func(sqrt).into());
    math.insert("type".into(), LuaFunction::from_func(type_).into());
    math.insert("tointeger".into(), LuaFunction::from_func(tointeger).into());
    math.insert("ult".into(), LuaFunction::from_func(ult).into());
    math.insert("modf".into(), LuaFunction::from_func(modf).into());
    math.insert("fmod".into(), LuaFunction::from_func(fmod).into());

    math.insert("random".into(), LuaFunction::from_func(random).into());

    math.insert(
        "randomseed".into(),
        LuaFunction::from_func(randomseed).into(),
    );
    Ok(LuaValue::Table(Rc::new(RefCell::new(math))))
}

pub fn abs(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    match stack.pop1(args).try_to_number() {
        Some(num) => {
            let abs = num.abs();
            stack.data_stack.push(abs.into());
            Ok(1)
        }
        None => Err(RuntimeError::NotNumber),
    }
}
pub fn acos(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    match stack.pop1(args).try_to_number() {
        Some(num) => {
            stack.data_stack.push(num.to_float().acos().into());
            Ok(1)
        }
        None => Err(RuntimeError::NotNumber),
    }
}
pub fn asin(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    match stack.pop1(args).try_to_number() {
        Some(num) => {
            stack.data_stack.push(num.to_float().asin().into());
            Ok(1)
        }
        None => Err(RuntimeError::NotNumber),
    }
}
pub fn atan(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    match args {
        0 => Err(RuntimeError::ValueExpected),
        1 => match stack.pop1(args).try_to_number() {
            Some(num) => {
                stack.data_stack.push(num.to_float().atan2(1.0).into());
                Ok(1)
            }
            None => Err(RuntimeError::NotNumber),
        },
        _ => {
            let (y, x) = stack.pop2(args);
            let y = match y.try_to_number() {
                Some(num) => num.to_float(),
                None => return Err(RuntimeError::NotNumber),
            };
            let x = match x.try_to_number() {
                Some(num) => num.to_float(),
                None => return Err(RuntimeError::NotNumber),
            };
            stack.data_stack.push(y.atan2(x).into());
            Ok(1)
        }
    }
}
pub fn ceil(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    match stack.pop1(args).try_to_number() {
        Some(num) => {
            stack.data_stack.push(num.ceil().into());
            Ok(1)
        }
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn floor(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    match stack.pop1(args).try_to_number() {
        Some(num) => {
            stack.data_stack.push(num.floor().into());
            Ok(1)
        }
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn cos(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    match stack.pop1(args).try_to_number() {
        Some(num) => {
            stack.data_stack.push(num.cos().into());
            Ok(1)
        }
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn sin(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    match stack.pop1(args).try_to_number() {
        Some(num) => {
            stack.data_stack.push(num.sin().into());
            Ok(1)
        }
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn deg(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    match stack.pop1(args).try_to_number() {
        Some(num) => {
            stack.data_stack.push(num.deg().into());
            Ok(1)
        }
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn rad(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    match stack.pop1(args).try_to_number() {
        Some(num) => {
            stack.data_stack.push(num.deg().into());
            Ok(1)
        }
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn exp(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    match stack.pop1(args).try_to_number() {
        Some(num) => {
            stack.data_stack.push(num.exp().into());
            Ok(1)
        }
        _ => Err(RuntimeError::NotNumber),
    }
}
pub fn log(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let mut it = stack.pop_n(args);
    let x = match it.next().unwrap().try_to_number() {
        Some(num) => num,
        None => return Err(RuntimeError::NotNumber),
    };
    let next = it.next();
    drop(it);
    match next {
        Some(base) => match base.try_to_number() {
            Some(base) => {
                stack.data_stack.push(x.log(base).into());
                Ok(1)
            }
            None => Err(RuntimeError::NotNumber),
        },
        None => {
            // default to e
            stack.data_stack.push(x.ln().into());
            Ok(1)
        }
    }
}

pub fn sqrt(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    match stack.pop1(args).try_to_number() {
        Some(num) => {
            stack.data_stack.push(num.sqrt().into());
            Ok(1)
        }
        _ => Err(RuntimeError::NotNumber),
    }
}

pub fn type_(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let ret = match stack.pop1(args) {
        LuaValue::Number(num) => match num {
            LuaNumber::Int(_) => "integer".into(),
            LuaNumber::Float(_) => "float".into(),
        },
        _ => ().into(),
    };
    stack.data_stack.push(ret);
    Ok(1)
}
pub fn tointeger(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let ret = match stack.pop1(args).try_to_int() {
        Some(num) => num.into(),
        _ => LuaValue::Nil,
    };
    stack.data_stack.push(ret);
    Ok(1)
}
pub fn ult(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::ValueExpected);
    }
    let (a, b) = stack.pop2(args);
    let a = match a.try_to_int() {
        Some(num) => num,
        None => return Err(RuntimeError::NotInteger),
    };
    let b = match b.try_to_int() {
        Some(num) => num,
        None => return Err(RuntimeError::NotInteger),
    };

    #[cfg(feature = "32bit")]
    let res = (a as u32) < (b as u32);
    #[cfg(not(feature = "32bit"))]
    let res = (a as u64) < (b as u64);

    stack.data_stack.push(res.into());
    Ok(1)
}

pub fn random(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    let rand = match args {
        0 => {
            // [0,1)
            stack.rng.gen_range(0.0..1.0).into()
        }
        1 => {
            // [1,num]
            match stack.pop1(args).try_to_int() {
                Some(num) => stack.rng.gen_range(1..=num).into(),
                None => return Err(RuntimeError::NotInteger),
            }
        }
        _ => {
            // [m, n]
            let (m, n) = stack.pop2(args);

            match (m.try_to_int(), n.try_to_int()) {
                (Some(m), Some(n)) => stack.rng.gen_range(m..=n).into(),
                _ => return Err(RuntimeError::NotInteger),
            }
        }
    };
    stack.data_stack.push(rand);
    Ok(1)
}

pub fn randomseed(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    match args {
        0 => {
            stack.rng = StdRng::from_entropy();
        }
        1 => {
            let seed = match stack.pop1(args).try_to_int() {
                Some(num) => num,
                None => return Err(RuntimeError::NotInteger),
            };
            stack.rng = StdRng::seed_from_u64(seed as u64);
        }
        _ => {
            let (seed1, seed2) = stack.pop2(args);
            match (seed1.try_to_int(), seed2.try_to_int()) {
                (Some(seed1), Some(seed2)) => {
                    // @TODO this should be 128bit seed
                    let seed = (seed1.rotate_left(32) ^ seed2) as u64;
                    stack.rng = StdRng::seed_from_u64(seed as u64);
                }
                _ => return Err(RuntimeError::NotInteger),
            }
        }
    }
    Ok(0)
}

pub fn modf(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    match stack.pop1(args) {
        LuaValue::Number(num) => match num {
            LuaNumber::Int(i) => {
                stack.data_stack.push(i.into());
                stack.data_stack.push(0.0.into());
                Ok(2)
            }
            LuaNumber::Float(f) => {
                let fract = f.fract();
                let int = f - fract;
                stack.data_stack.push(int.into());
                stack.data_stack.push(fract.into());
                Ok(2)
            }
        },
        _ => Err(RuntimeError::NotNumber),
    }
}

pub fn fmod(stack: &mut Stack, _chunk: &Chunk, args: usize) -> Result<usize, RuntimeError> {
    if args < 2 {
        return Err(RuntimeError::ValueExpected);
    }
    let (x, y) = stack.pop2(args);
    let x = match x.try_to_number() {
        Some(num) => num,
        None => return Err(RuntimeError::NotNumber),
    };
    let y = match y.try_to_number() {
        Some(num) => num,
        None => return Err(RuntimeError::NotNumber),
    };
    stack.data_stack.push((x % y).into());
    Ok(1)
}
