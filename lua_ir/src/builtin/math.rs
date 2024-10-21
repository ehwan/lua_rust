use std::cell::RefCell;
use std::rc::Rc;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use crate::FloatType;
use crate::LuaEnv;
use crate::LuaFunction;
use crate::LuaNumber;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;

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

pub fn abs(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    let arg = arg
        .try_to_number()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    let abs = arg.abs();
    env.push(abs.into());
    Ok(1)
}
pub fn acos(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }

    let arg = env.pop();
    let arg = arg
        .try_to_number()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    env.push(arg.to_float().acos().into());
    Ok(1)
}
pub fn asin(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    let arg = arg
        .try_to_number()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    env.push(arg.to_float().asin().into());
    Ok(1)
}
pub fn atan(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    match args {
        0 => Err(RuntimeError::new_empty_argument(1, "number")),
        1 => {
            let arg = env.pop();
            let arg = arg
                .try_to_number()
                .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
            env.push(arg.to_float().atan2(1.0 as FloatType).into());
            Ok(1)
        }
        _ => {
            env.pop_n(args - 2);
            let (y, x) = env.pop2();
            let y = y
                .try_to_number()
                .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?
                .to_float();
            let x = x
                .try_to_number()
                .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?
                .to_float();

            env.push(y.atan2(x).into());
            Ok(1)
        }
    }
}
pub fn ceil(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }

    let arg = env.pop();
    let arg = arg
        .try_to_number()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    env.push(arg.ceil().into());
    Ok(1)
}
pub fn floor(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    let arg = arg
        .try_to_number()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    env.push(arg.floor().into());
    Ok(1)
}
pub fn cos(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    let arg = arg
        .try_to_number()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    env.push(arg.cos().into());
    Ok(1)
}
pub fn sin(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    let arg = arg
        .try_to_number()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    env.push(arg.sin().into());
    Ok(1)
}
pub fn deg(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    let arg = arg
        .try_to_number()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    env.push(arg.deg().into());
    Ok(1)
}
pub fn rad(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    let arg = arg
        .try_to_number()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    env.push(arg.rad().into());
    Ok(1)
}
pub fn exp(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    let arg = arg
        .try_to_number()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    env.push(arg.exp().into());
    Ok(1)
}
pub fn log(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    match args {
        0 => Err(RuntimeError::new_empty_argument(1, "number")),

        1 => {
            let x = env.pop();
            let x = x
                .try_to_number()
                .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
            env.push(x.ln().into());
            Ok(1)
        }

        _ => {
            env.pop_n(args - 2);
            let (x, base) = env.pop2();
            let x = x
                .try_to_number()
                .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
            let base = base
                .try_to_number()
                .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;

            env.push(x.log(base).into());
            Ok(1)
        }
    }
}

pub fn sqrt(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    let arg = arg
        .try_to_number()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    env.push(arg.sqrt().into());
    Ok(1)
}

pub fn type_(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    let ret = match arg {
        LuaValue::Number(num) => match num {
            LuaNumber::Int(_) => "integer".into(),
            LuaNumber::Float(_) => "float".into(),
        },
        _ => LuaValue::Nil,
    };
    env.push(ret);
    Ok(1)
}
pub fn tointeger(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }
    let arg = env.pop();
    let arg = arg
        .try_to_int()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    env.push(arg.into());
    Ok(1)
}
pub fn ult(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args == 1 {
        env.pop();
        return Err(RuntimeError::new_empty_argument(2, "number"));
    } else if args > 2 {
        env.pop_n(args - 2);
    }
    let (a, b) = env.pop2();
    let a = a
        .try_to_int()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    let b = b
        .try_to_int()
        .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;

    #[cfg(feature = "32bit")]
    let res = (a as u32) < (b as u32);
    #[cfg(not(feature = "32bit"))]
    let res = (a as u64) < (b as u64);

    env.push(res.into());
    Ok(1)
}

pub fn random(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    let rand = match args {
        0 => {
            // [0,1)
            env.rng.gen_range(0.0..1.0).into()
        }
        1 => {
            // [1,num]
            let arg = env.pop();
            let arg = arg
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
            env.rng.gen_range(1..=arg).into()
        }
        _ => {
            // [m, n]
            env.pop_n(args - 2);
            let (m, n) = env.pop2();

            let m = m
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
            let n = n
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;

            env.rng.gen_range(m..=n).into()
        }
    };
    env.push(rand);
    Ok(1)
}

pub fn randomseed(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    match args {
        0 => {
            env.rng = StdRng::from_entropy();
        }
        1 => {
            let seed = env.pop();
            let seed = seed
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
            env.rng = StdRng::seed_from_u64(seed as u64);
        }
        _ => {
            env.pop_n(args - 2);
            let (seed1, seed2) = env.pop2();
            let seed1 = seed1
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
            let seed2 = seed2
                .try_to_int()
                .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;
            // @TODO this should be 128bit seed
            let seed = (seed1.rotate_left(32) ^ seed2) as u64;
            env.rng = StdRng::seed_from_u64(seed as u64);
        }
    }
    Ok(0)
}

pub fn modf(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args > 1 {
        env.pop_n(args - 1);
    }

    let arg = env.pop();
    let arg = arg
        .try_to_number()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    match arg {
        LuaNumber::Int(i) => {
            env.push2(i.into(), 0.0.into());
            Ok(2)
        }
        LuaNumber::Float(f) => {
            let fract = f.fract();
            let int = f - fract;
            env.push2(int.into(), fract.into());
            Ok(2)
        }
    }
}

pub fn fmod(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::new_empty_argument(1, "number"));
    } else if args == 1 {
        env.pop();
        return Err(RuntimeError::new_empty_argument(2, "number"));
    } else if args > 2 {
        env.pop_n(args - 2);
    }
    let (x, y) = env.pop2();
    let x = x
        .try_to_number()
        .map_err(|e| RuntimeError::BadArgument(1, Box::new(e)))?;
    let y = y
        .try_to_number()
        .map_err(|e| RuntimeError::BadArgument(2, Box::new(e)))?;
    env.push((x % y).into());
    Ok(1)
}
