use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::RwLock;

use rand::SeedableRng;

use crate::builtin;
use crate::luaval::RefOrValue;
use crate::FunctionInfo;
use crate::IntType;
use crate::LuaFunction;
use crate::LuaFunctionLua;
use crate::LuaTable;
use crate::LuaValue;

use crate::Instruction;
use crate::RuntimeError;

pub struct LuaEnv {
    /// _Env
    pub(crate) env: LuaValue,
    /// random number generator
    pub(crate) rng: rand::rngs::StdRng,

    pub main_thread: Arc<RwLock<LuaThread>>,
}

impl LuaEnv {
    pub fn new() -> LuaEnv {
        let env = Arc::new(RwLock::new(builtin::init_env().unwrap()));
        // assign _G to _ENV itself
        env.write()
            .unwrap()
            .insert("_G".into(), LuaValue::Table(Arc::clone(&env)));
        LuaEnv {
            env: LuaValue::Table(env),
            rng: rand::rngs::StdRng::from_entropy(),
            main_thread: Arc::new(RwLock::new(LuaThread::new())),
        }
    }

    /// Try to call binary metamethod f(lhs, rhs).
    /// It tries to search metamethod on lhs first, then rhs.
    fn try_call_metamethod(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
        lhs: LuaValue,
        rhs: LuaValue,
        meta_name: &str,
    ) -> Result<(), RuntimeError> {
        match lhs.get_metavalue(meta_name) {
            Some(meta) => {
                let mut t = thread.write().unwrap();
                t.data_stack.push(lhs);
                t.data_stack.push(rhs);
                drop(t);
                self.function_call(thread, chunk, 2, meta, Some(1))
            }
            None => match rhs.get_metavalue(meta_name) {
                Some(meta) => {
                    let mut t = thread.write().unwrap();
                    t.data_stack.push(lhs);
                    t.data_stack.push(rhs);
                    drop(t);
                    self.function_call(thread, chunk, 2, meta, Some(1))
                }
                None => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// add operation with __add metamethod
    pub fn add(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match (lhs, rhs) {
            // if both are numbers, add them
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                t.data_stack.push((lhs + rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => {
                drop(t);
                self.try_call_metamethod(thread, chunk, lhs, rhs, "__add")
            }
        }
    }

    /// sub operation with __sub metamethod
    pub fn sub(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                t.data_stack.push((lhs - rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => {
                drop(t);
                self.try_call_metamethod(thread, chunk, lhs, rhs, "__sub")
            }
        }
    }
    /// mul operation with __mul metamethod
    pub fn mul(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                t.data_stack.push((lhs * rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => {
                drop(t);
                self.try_call_metamethod(thread, chunk, lhs, rhs, "__mul")
            }
        }
    }
    /// div operation with __div metamethod
    pub fn div(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                t.data_stack.push((lhs / rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => {
                drop(t);
                self.try_call_metamethod(thread, chunk, lhs, rhs, "__div")
            }
        }
    }
    /// mod operation with __mod metamethod
    pub fn mod_(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                t.data_stack.push((lhs % rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => {
                drop(t);
                self.try_call_metamethod(thread, chunk, lhs, rhs, "__mod")
            }
        }
    }
    /// pow operation with __pow metamethod
    pub fn pow(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                t.data_stack.push((lhs.pow(rhs)).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => {
                drop(t);
                self.try_call_metamethod(thread, chunk, lhs, rhs, "__pow")
            }
        }
    }
    /// unary minus operation with __unm metamethod
    pub fn unm(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match lhs {
            LuaValue::Number(num) => {
                t.data_stack.push((-num).into());
                Ok(())
            }
            lhs => match lhs.get_metavalue("__unm") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    t.data_stack.push(lhs.clone());
                    t.data_stack.push(lhs);
                    drop(t);
                    self.function_call(thread, chunk, 2, meta, Some(1))
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// floor division operation with __idiv metamethod
    pub fn idiv(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                t.data_stack.push((lhs.floor_div(rhs)).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => {
                drop(t);
                self.try_call_metamethod(thread, chunk, lhs, rhs, "__idiv")
            }
        }
    }
    /// bitwise and operation with __band metamethod
    pub fn band(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        t.data_stack.push((lhs & rhs).into());
                        Ok(())
                    }
                    _ => {
                        drop(t);
                        self.try_call_metamethod(thread, chunk, lhs, rhs, "__band")
                    }
                }
            }
            // else, try to call metamethod, search on left first
            _ => {
                drop(t);
                self.try_call_metamethod(thread, chunk, lhs, rhs, "__band")
            }
        }
    }
    /// bitwise or operation with __bor metamethod
    pub fn bor(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        t.data_stack.push((lhs | rhs).into());
                        Ok(())
                    }
                    _ => {
                        drop(t);
                        self.try_call_metamethod(thread, chunk, lhs, rhs, "__bor")
                    }
                }
            }
            // else, try to call metamethod, search on left first
            _ => {
                drop(t);
                self.try_call_metamethod(thread, chunk, lhs, rhs, "__bor")
            }
        }
    }
    /// bitwise xor operation with __bxor metamethod
    pub fn bxor(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        t.data_stack.push((lhs ^ rhs).into());
                        Ok(())
                    }
                    _ => {
                        drop(t);
                        self.try_call_metamethod(thread, chunk, lhs, rhs, "__bxor")
                    }
                }
            }
            // else, try to call metamethod, search on left first
            _ => {
                drop(t);
                self.try_call_metamethod(thread, chunk, lhs, rhs, "__bxor")
            }
        }
    }
    /// bitwise shift left operation with __shl metamethod
    pub fn shl(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        t.data_stack.push((lhs << rhs).into());
                        Ok(())
                    }
                    _ => {
                        drop(t);
                        self.try_call_metamethod(thread, chunk, lhs, rhs, "__shl")
                    }
                }
            }
            // else, try to call metamethod, search on left first
            _ => {
                drop(t);
                self.try_call_metamethod(thread, chunk, lhs, rhs, "__shl")
            }
        }
    }
    /// bitwise shift right operation with __shr metamethod
    pub fn shr(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Some(lhs), Some(rhs)) => {
                        t.data_stack.push((lhs >> rhs).into());
                        Ok(())
                    }
                    _ => {
                        drop(t);
                        self.try_call_metamethod(thread, chunk, lhs, rhs, "__shr")
                    }
                }
            }
            // else, try to call metamethod, search on left first
            _ => {
                drop(t);
                self.try_call_metamethod(thread, chunk, lhs, rhs, "__shr")
            }
        }
    }
    /// bitwise not operation with __bnot metamethod
    pub fn bnot(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match &lhs {
            LuaValue::Number(lhs_num) => match lhs_num.try_to_int() {
                Some(i) => {
                    t.data_stack.push((!i).into());
                    Ok(())
                }
                _ => match lhs.get_metavalue("__bnot") {
                    Some(meta) => {
                        // For the unary operators (negation, length, and bitwise NOT),
                        // the metamethod is computed and called with a dummy second operand
                        // equal to the first one.
                        t.data_stack.push(lhs.clone());
                        t.data_stack.push(lhs);
                        drop(t);
                        self.function_call(thread, chunk, 2, meta, Some(1))
                    }
                    _ => Err(RuntimeError::NoMetaMethod),
                },
            },
            _ => match lhs.get_metavalue("__bnot") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    t.data_stack.push(lhs.clone());
                    t.data_stack.push(lhs);
                    drop(t);
                    self.function_call(thread, chunk, 2, meta, Some(1))
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// concat operation with __concat metamethod
    pub fn concat(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match lhs {
            LuaValue::Number(lhs_num) => match rhs {
                LuaValue::Number(rhs_num) => {
                    let mut concated = lhs_num.to_string().into_bytes();
                    concated.append(&mut rhs_num.to_string().into_bytes());
                    t.data_stack.push(LuaValue::String(concated));
                    Ok(())
                }
                LuaValue::String(mut rhs) => {
                    let mut lhs = lhs_num.to_string().into_bytes();
                    lhs.append(&mut rhs);
                    t.data_stack.push(LuaValue::String(lhs));
                    Ok(())
                }
                _ => {
                    drop(t);
                    self.try_call_metamethod(thread, chunk, lhs, rhs, "__concat")
                }
            },

            LuaValue::String(lhs_str) => match rhs {
                LuaValue::Number(rhs_num) => {
                    let mut concated = lhs_str;
                    concated.append(&mut rhs_num.to_string().into_bytes());
                    t.data_stack.push(LuaValue::String(concated));
                    Ok(())
                }
                LuaValue::String(mut rhs) => {
                    let mut lhs = lhs_str;
                    lhs.append(&mut rhs);
                    t.data_stack.push(LuaValue::String(lhs));
                    Ok(())
                }
                _ => {
                    drop(t);
                    self.try_call_metamethod(
                        thread,
                        chunk,
                        LuaValue::String(lhs_str),
                        rhs,
                        "__concat",
                    )
                }
            },

            _ => {
                drop(t);
                self.try_call_metamethod(thread, chunk, lhs, rhs, "__concat")
            }
        }
    }
    /// `#` length operation with __len metamethod
    pub fn len(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let lhs = t.data_stack.pop().unwrap();
        match lhs {
            LuaValue::String(s) => {
                t.data_stack.push((s.len() as IntType).into());
                Ok(())
            }
            LuaValue::Table(table) => {
                let meta = table.read().unwrap().get_metavalue("__len");
                match meta {
                    Some(meta) => {
                        // For the unary operators (negation, length, and bitwise NOT),
                        // the metamethod is computed and called with a dummy second operand
                        // equal to the first one.
                        t.data_stack.push(LuaValue::Table(Arc::clone(&table)));
                        t.data_stack.push(LuaValue::Table(table));
                        drop(t);
                        self.function_call(thread, chunk, 2, meta, Some(1))
                    }
                    _ => {
                        t.data_stack
                            .push((table.read().unwrap().len() as IntType).into());
                        Ok(())
                    }
                }
            }
            lhs => match lhs.get_metavalue("__len") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    t.data_stack.push(lhs.clone());
                    t.data_stack.push(lhs);
                    drop(t);
                    self.function_call(thread, chunk, 2, meta, Some(1))
                }
                _ => Err(RuntimeError::NoMetaMethod),
            },
        }
    }
    /// table index get operation with __index metamethod
    pub fn index(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let key = t.data_stack.pop().unwrap();
        let table = t.data_stack.pop().unwrap();
        match table {
            LuaValue::Table(table) => {
                let get = table.read().unwrap().get(&key).cloned();
                if let Some(get) = get {
                    t.data_stack.push(get);
                    Ok(())
                } else {
                    let meta = table.read().unwrap().get_metavalue("__index");
                    match meta {
                        Some(LuaValue::Function(meta_func)) => {
                            t.data_stack.push(LuaValue::Table(table));
                            t.data_stack.push(key);
                            drop(t);
                            self.function_call(
                                thread,
                                chunk,
                                2,
                                LuaValue::Function(meta_func),
                                Some(1),
                            )
                        }
                        Some(LuaValue::Table(meta_table)) => {
                            t.data_stack.push(LuaValue::Table(meta_table));
                            t.data_stack.push(key);
                            drop(t);
                            self.index(thread, chunk)
                        }
                        _ => {
                            t.data_stack.push(LuaValue::Nil);
                            Ok(())
                        }
                    }
                }
            }
            table => {
                let meta = table.get_metavalue("__index");
                match meta {
                    Some(LuaValue::Function(meta_func)) => {
                        t.data_stack.push(table);
                        t.data_stack.push(key);
                        drop(t);
                        self.function_call(thread, chunk, 2, LuaValue::Function(meta_func), Some(1))
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        t.data_stack.push(LuaValue::Table(meta_table));
                        t.data_stack.push(key);
                        drop(t);
                        self.index(thread, chunk)
                    }
                    _ => Err(RuntimeError::NotTable),
                }
            }
        }
    }
    /// table index set operation with __newindex metamethod
    pub fn newindex(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let key = t.data_stack.pop().unwrap();
        let table = t.data_stack.pop().unwrap();
        let value = t.data_stack.pop().unwrap();

        match table {
            LuaValue::Table(table) => {
                {
                    let mut table = table.write().unwrap();
                    if let Some(val) = table.get_mut(&key) {
                        // if rhs is nil, remove the key
                        if value.is_nil() {
                            table.remove(&key);
                        } else {
                            *val = value;
                        }
                        return Ok(());
                    }
                }
                let meta = table.read().unwrap().get_metavalue("__newindex");
                match meta {
                    Some(LuaValue::Function(meta_func)) => {
                        t.data_stack.push(LuaValue::Table(table));
                        t.data_stack.push(key);
                        t.data_stack.push(value);
                        drop(t);
                        self.function_call(thread, chunk, 3, LuaValue::Function(meta_func), Some(0))
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        t.data_stack.push(value);
                        t.data_stack.push(LuaValue::Table(meta_table));
                        t.data_stack.push(key);
                        drop(t);
                        self.newindex(thread, chunk)
                    }
                    _ => {
                        table.write().unwrap().insert(key, value);
                        Ok(())
                    }
                }
            }
            table => {
                let meta = table.get_metavalue("__newindex");
                match meta {
                    Some(LuaValue::Function(meta_func)) => {
                        t.data_stack.push(table);
                        t.data_stack.push(key);
                        t.data_stack.push(value);
                        drop(t);
                        self.function_call(thread, chunk, 3, LuaValue::Function(meta_func), Some(0))
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        t.data_stack.push(value);
                        t.data_stack.push(LuaValue::Table(meta_table));
                        t.data_stack.push(key);
                        drop(t);
                        self.newindex(thread, chunk)
                    }
                    _ => Err(RuntimeError::NotTable),
                }
            }
        }
    }
    /// equality operation with __eq metamethod
    pub fn eq(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();

        match (lhs, rhs) {
            (LuaValue::Table(lhs), LuaValue::Table(rhs)) => {
                if Arc::ptr_eq(&lhs, &rhs) {
                    t.data_stack.push(LuaValue::Boolean(true));
                    return Ok(());
                } else {
                    drop(t);
                    self.try_call_metamethod(
                        thread,
                        chunk,
                        LuaValue::Table(lhs),
                        LuaValue::Table(rhs),
                        "__eq",
                    )
                }
            }
            (lhs, rhs) => {
                t.data_stack.push(LuaValue::Boolean(lhs == rhs));
                Ok(())
            }
        }
    }
    /// less than operation with __lt metamethod
    pub fn lt(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();

        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                t.data_stack.push(LuaValue::Boolean(lhs < rhs));
                Ok(())
            }
            (LuaValue::String(lhs), LuaValue::String(rhs)) => {
                t.data_stack.push(LuaValue::Boolean(lhs < rhs));
                Ok(())
            }
            (lhs, rhs) => {
                drop(t);
                self.try_call_metamethod(thread, chunk, lhs, rhs, "__lt")
            }
        }
    }

    /// less than or equal operation with __le metamethod
    pub fn le(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        let mut t = thread.write().unwrap();
        let rhs = t.data_stack.pop().unwrap();
        let lhs = t.data_stack.pop().unwrap();

        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                t.data_stack.push(LuaValue::Boolean(lhs <= rhs));
                Ok(())
            }
            (LuaValue::String(lhs), LuaValue::String(rhs)) => {
                t.data_stack.push(LuaValue::Boolean(lhs <= rhs));
                Ok(())
            }
            (lhs, rhs) => {
                drop(t);
                self.try_call_metamethod(thread, chunk, lhs, rhs, "__le")
            }
        }
    }

    /// function call with __call metamethod.
    /// this does not return until the function call is finished.
    pub fn function_call(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
        // number of arguments actually passed
        args_num: usize,
        // function object
        func: LuaValue,
        // number of return values expected; returned values will be adjusted to this number
        expected_ret: Option<usize>,
    ) -> Result<(), RuntimeError> {
        match func {
            LuaValue::Function(func) => {
                let func = func.read().unwrap();
                match &*func {
                    LuaFunction::LuaFunc(lua_internal_func_ref) => {
                        let lua_internal = lua_internal_func_ref.clone();
                        drop(func);

                        let func_id = lua_internal.function_id;
                        let func_info = &chunk.functions[func_id];

                        let usize_len0 = thread.read().unwrap().usize_stack.len();

                        {
                            let mut t = thread.write().unwrap();
                            let t = &mut *t;

                            t.usize_stack.push(t.counter);
                            t.usize_stack.push(t.bp);
                            t.usize_stack.push(t.data_stack.len() - args_num);

                            t.bp = t.local_variables.len();
                            t.local_variables
                                .resize_with(t.bp + func_info.args, Default::default);
                        }
                        let variadic = if args_num > func_info.args {
                            let mut t = thread.write().unwrap();
                            let t = &mut *t;
                            let variadic = if func_info.is_variadic {
                                let variadic_num = args_num - func_info.args;
                                t.data_stack
                                    .drain(t.data_stack.len() - variadic_num..)
                                    .collect()
                            } else {
                                t.data_stack.resize_with(
                                    t.data_stack.len() - args_num + func_info.args,
                                    Default::default,
                                );
                                Vec::new()
                            };
                            for (idx, arg) in t
                                .data_stack
                                .drain(t.data_stack.len() - func_info.args..)
                                .enumerate()
                            {
                                *t.local_variables.get_mut(t.bp + idx).unwrap() =
                                    RefOrValue::Value(arg);
                            }

                            variadic
                        } else {
                            let mut t = thread.write().unwrap();
                            let t = &mut *t;
                            for (idx, arg) in t
                                .data_stack
                                .drain(t.data_stack.len() - args_num..)
                                .enumerate()
                            {
                                *t.local_variables.get_mut(t.bp + idx).unwrap() =
                                    RefOrValue::Value(arg);
                            }
                            Vec::new()
                        };

                        let func_stack = FunctionCallStackElem {
                            function_object: lua_internal,
                            return_expected: expected_ret,
                            variadic,
                        };

                        {
                            let mut t = thread.write().unwrap();
                            t.function_stack.push(func_stack);
                            t.counter = func_info.address;
                        }

                        loop {
                            let t = thread.read().unwrap();
                            if t.usize_stack.len() <= usize_len0 {
                                break;
                            }
                            let instruction = chunk.instructions.get(t.counter).unwrap();
                            drop(t);
                            self.cycle(thread, chunk, instruction)?;
                        }
                        Ok(())
                    }
                    LuaFunction::RustFunc(rust_internal) => {
                        let ret_num = rust_internal(self, thread, chunk, args_num)?;
                        drop(func);
                        if let Some(expected) = expected_ret {
                            let mut t = thread.write().unwrap();
                            let adjusted = t.data_stack.len() - ret_num + expected;
                            t.data_stack.resize_with(adjusted, Default::default);
                        }
                        Ok(())
                    }
                }
            }
            other => {
                let func = other.get_metavalue("__call");
                if let Some(meta) = func {
                    {
                        let mut t = thread.write().unwrap();
                        let t = &mut *t;
                        t.data_stack.insert(t.data_stack.len() - args_num, other);
                    }
                    self.function_call(thread, chunk, args_num + 1, meta, expected_ret)
                } else {
                    Err(RuntimeError::NotFunction)
                }
            }
        }
    }
    /// execute single instruction
    pub fn cycle(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
        instruction: &Instruction,
    ) -> Result<(), RuntimeError> {
        match instruction {
            Instruction::Clone => {
                let mut t = thread.write().unwrap();
                let top = t.data_stack.last().unwrap().clone();
                t.data_stack.push(top);
            }
            Instruction::Sp => {
                let mut t = thread.write().unwrap();
                let l = t.data_stack.len();
                t.usize_stack.push(l);
            }
            Instruction::Pop => {
                thread.write().unwrap().data_stack.pop();
            }
            Instruction::Deref => {
                let mut t = thread.write().unwrap();
                let sp = *t.usize_stack.last().unwrap();
                let top = t.data_stack[sp].clone();
                t.data_stack.push(top);
            }
            Instruction::Jump(label) => {
                let pc = *chunk.label_map.get(*label).unwrap();
                let mut t = thread.write().unwrap();
                t.counter = pc;
                return Ok(());
            }
            Instruction::JumpTrue(label) => {
                let mut t = thread.write().unwrap();
                if t.data_stack.pop().unwrap().to_bool() {
                    let pc = *chunk.label_map.get(*label).unwrap();
                    t.counter = pc;
                    return Ok(());
                }
            }
            Instruction::JumpFalse(label) => {
                let mut t = thread.write().unwrap();
                if !t.data_stack.pop().unwrap().to_bool() {
                    let pc = *chunk.label_map.get(*label).unwrap();
                    t.counter = pc;
                    return Ok(());
                }
            }
            Instruction::GetLocalVariable(local_id) => {
                let mut t = thread.write().unwrap();
                let val = t.local_variables.get(*local_id + t.bp).unwrap();
                let val = match val {
                    RefOrValue::Ref(val) => val.read().unwrap().clone(),
                    RefOrValue::Value(val) => val.clone(),
                };
                t.data_stack.push(val);
            }
            Instruction::SetLocalVariable(local_id) => {
                let mut t = thread.write().unwrap();
                let top = t.data_stack.pop().unwrap();
                let localvar_idx = *local_id + t.bp;
                let val = t.local_variables.get_mut(localvar_idx).unwrap();
                match val {
                    RefOrValue::Ref(val) => {
                        *val.write().unwrap() = top;
                    }
                    RefOrValue::Value(val) => {
                        *val = top;
                    }
                }
            }
            Instruction::InitLocalVariable(local_id) => {
                let mut t = thread.write().unwrap();
                let top = t.data_stack.pop().unwrap();
                let localvar_idx = *local_id + t.bp;
                if t.local_variables.len() <= localvar_idx {
                    t.local_variables
                        .resize_with(localvar_idx + 1, Default::default);
                }
                *t.local_variables.get_mut(localvar_idx).unwrap() = RefOrValue::Value(top);
            }
            Instruction::IsNil => {
                let mut t = thread.write().unwrap();
                let top = t.data_stack.pop().unwrap();
                t.data_stack.push(LuaValue::Boolean(top.is_nil()));
            }

            Instruction::Nil => {
                let mut t = thread.write().unwrap();
                t.data_stack.push(LuaValue::Nil);
            }
            Instruction::Boolean(b) => {
                let mut t = thread.write().unwrap();
                t.data_stack.push(LuaValue::Boolean(*b));
            }
            Instruction::Numeric(n) => {
                let mut t = thread.write().unwrap();
                t.data_stack.push(LuaValue::Number(*n));
            }
            Instruction::String(s) => {
                let mut t = thread.write().unwrap();
                t.data_stack.push(LuaValue::String(s.clone()));
            }
            Instruction::GetEnv => {
                // @TODO replace this instruction with GetEnvIndex
                let mut t = thread.write().unwrap();
                t.data_stack.push(self.env.clone());
            }
            Instruction::TableInit(cap) => {
                let table = LuaTable::with_capacity(*cap);
                let mut t = thread.write().unwrap();
                t.data_stack.push(table.into());
            }
            Instruction::TableIndexInit => {
                let mut t = thread.write().unwrap();
                let value = t.data_stack.pop().unwrap();
                let index = t.data_stack.pop().unwrap();
                let table = t.data_stack.last().unwrap();
                if let LuaValue::Table(table) = table {
                    table.write().unwrap().insert(index, value);
                } else {
                    unreachable!("table must be on top of stack");
                }
            }
            Instruction::TableInitLast(i) => {
                let mut t = thread.write().unwrap();
                let sp = t.usize_stack.pop().unwrap();
                let mut values: BTreeMap<_, _> = t
                    .data_stack
                    .drain(sp..)
                    .enumerate()
                    .map(|(idx, value)| {
                        let index = idx as IntType + *i;
                        (index.into(), value)
                    })
                    .collect();
                let table = t.data_stack.last().unwrap();
                if let LuaValue::Table(table) = table {
                    table.write().unwrap().arr.append(&mut values);
                } else {
                    unreachable!("table must be on top of stack");
                }
            }

            Instruction::TableIndex => {
                self.index(thread, chunk)?;
            }
            Instruction::TableIndexSet => {
                self.newindex(thread, chunk)?;
            }

            Instruction::FunctionInit(func_id, num_upvalues) => {
                let func = LuaFunctionLua {
                    function_id: *func_id,
                    upvalues: Vec::with_capacity(*num_upvalues),
                };
                let func = LuaFunction::LuaFunc(func);
                let mut t = thread.write().unwrap();
                t.data_stack.push(func.into());
            }
            Instruction::FunctionInitUpvalueFromLocalVar(src_local_id) => {
                let mut t = thread.write().unwrap();
                let local_var = {
                    let localvar_idx = *src_local_id + t.bp;
                    let local_var = t.local_variables.get_mut(localvar_idx).unwrap();
                    // upvalue must be reference.
                    match local_var {
                        RefOrValue::Ref(r) => r.clone(),
                        RefOrValue::Value(v) => {
                            let reffed_var = Arc::new(RwLock::new(v.clone()));
                            *local_var = RefOrValue::Ref(Arc::clone(&reffed_var));
                            reffed_var
                        }
                    }
                };

                match t.data_stack.last().unwrap() {
                    LuaValue::Function(f) => match &mut *f.write().unwrap() {
                        LuaFunction::LuaFunc(f) => {
                            f.upvalues.push(local_var);
                        }
                        _ => unreachable!("stack top must be function"),
                    },
                    _ => unreachable!("stack top must be function"),
                }
            }
            Instruction::FunctionInitUpvalueFromUpvalue(src_upvalue_id) => {
                let t = thread.read().unwrap();
                let value = Arc::clone(
                    &t.function_stack.last().unwrap().function_object.upvalues[*src_upvalue_id],
                );

                match t.data_stack.last().unwrap() {
                    LuaValue::Function(f) => match &mut *f.write().unwrap() {
                        LuaFunction::LuaFunc(f) => {
                            f.upvalues.push(value);
                        }
                        _ => unreachable!("stack top must be function"),
                    },
                    _ => unreachable!("stack top must be function"),
                }
            }

            Instruction::FunctionUpvalue(upvalue_id) => {
                let mut t = thread.write().unwrap();
                let func = t.function_stack.last().unwrap();
                let value = func.function_object.upvalues[*upvalue_id]
                    .read()
                    .unwrap()
                    .clone();
                t.data_stack.push(value);
            }
            Instruction::FunctionUpvalueSet(upvalue_id) => {
                let mut t = thread.write().unwrap();
                let rhs = t.data_stack.pop().unwrap();
                let func = t.function_stack.last().unwrap();
                let upvalue = &func.function_object.upvalues[*upvalue_id];
                *upvalue.write().unwrap() = rhs;
            }

            Instruction::BinaryAdd => {
                self.add(thread, chunk)?;
            }
            Instruction::BinarySub => {
                self.sub(thread, chunk)?;
            }
            Instruction::BinaryMul => {
                self.mul(thread, chunk)?;
            }
            Instruction::BinaryDiv => {
                self.div(thread, chunk)?;
            }
            Instruction::BinaryFloorDiv => {
                self.idiv(thread, chunk)?;
            }
            Instruction::BinaryMod => {
                self.mod_(thread, chunk)?;
            }
            Instruction::BinaryPow => {
                self.pow(thread, chunk)?;
            }
            Instruction::BinaryConcat => {
                self.concat(thread, chunk)?;
            }
            Instruction::BinaryBitwiseAnd => {
                self.band(thread, chunk)?;
            }
            Instruction::BinaryBitwiseOr => {
                self.bor(thread, chunk)?;
            }
            Instruction::BinaryBitwiseXor => {
                self.bxor(thread, chunk)?;
            }
            Instruction::BinaryShiftLeft => {
                self.shl(thread, chunk)?;
            }
            Instruction::BinaryShiftRight => {
                self.shr(thread, chunk)?;
            }
            Instruction::BinaryEqual => {
                self.eq(thread, chunk)?;
            }
            Instruction::BinaryLessThan => {
                self.lt(thread, chunk)?;
            }
            Instruction::BinaryLessEqual => {
                self.le(thread, chunk)?;
            }

            Instruction::UnaryMinus => {
                self.unm(thread, chunk)?;
            }
            Instruction::UnaryBitwiseNot => {
                self.bnot(thread, chunk)?;
            }
            Instruction::UnaryLength => {
                self.len(thread, chunk)?;
            }
            Instruction::UnaryLogicalNot => {
                let mut t = thread.write().unwrap();
                let top = t.data_stack.pop().unwrap().to_bool();
                t.data_stack.push((!top).into());
            }

            Instruction::FunctionCall(expected_ret) => {
                let mut t = thread.write().unwrap();
                let func = t.data_stack.pop().unwrap();
                let sp = t.usize_stack.pop().unwrap();
                let num_args = t.data_stack.len() - sp;
                drop(t);
                self.function_call(thread, chunk, num_args, func, *expected_ret)?;
            }

            // sp -> top
            Instruction::Return => {
                let mut t = thread.write().unwrap();
                if let Some(func) = t.function_stack.pop() {
                    // return from function call
                    let old_stacklen = t.usize_stack.pop().unwrap();
                    let old_bp = t.usize_stack.pop().unwrap();
                    let old_pc = t.usize_stack.pop().unwrap();
                    let old_localvar_len = t.bp;
                    t.local_variables.truncate(old_localvar_len);
                    t.bp = old_bp;
                    t.counter = old_pc;

                    if let Some(expected) = func.return_expected {
                        let adjusted = old_stacklen + expected;
                        t.data_stack.resize_with(adjusted, Default::default);
                    }
                    return Ok(());
                } else {
                    // main chunk
                    t.counter = chunk.instructions.len();
                    return Ok(());
                }
            }

            Instruction::GetVariadic(expected) => {
                let mut t = thread.write().unwrap();
                if let Some(expected) = expected {
                    let mut variadic: Vec<_> = t
                        .function_stack
                        .last()
                        .unwrap()
                        .variadic
                        .iter()
                        .take(*expected)
                        .cloned()
                        .collect();
                    let len = variadic.len();

                    t.data_stack.append(&mut variadic);
                    if len < *expected {
                        t.data_stack.extend(
                            std::iter::repeat(LuaValue::Nil).take(*expected - variadic.len()),
                        );
                    }
                } else {
                    let mut variadic = t.function_stack.last().unwrap().variadic.clone();
                    t.data_stack.append(&mut variadic);
                }
            }
        }
        thread.write().unwrap().counter += 1;
        Ok(())
    }
    /// run the whole chunk
    pub fn run(
        &mut self,
        thread: &Arc<RwLock<LuaThread>>,
        chunk: &Chunk,
    ) -> Result<(), RuntimeError> {
        loop {
            let t = thread.read().unwrap();
            if let Some(instruction) = chunk.instructions.get(t.counter) {
                drop(t);
                self.cycle(thread, chunk, instruction)?;
            } else {
                break;
            }
        }
        Ok(())
    }
}

/// for information about function call stack
#[derive(Debug, Clone)]
pub struct FunctionCallStackElem {
    /// function object
    pub function_object: LuaFunctionLua,
    /// number of return values expected.
    pub return_expected: Option<usize>,
    /// variadic arguments
    pub variadic: Vec<LuaValue>,
}

/// stack for a single thread
#[derive(Debug, Clone)]
pub struct LuaThread {
    /// local variable stack
    pub local_variables: Vec<RefOrValue>,
    /// offset of local variables for current scope
    pub bp: usize,

    /// normal stack, for temporary values
    pub data_stack: Vec<LuaValue>,

    /// stack for storing usize values
    pub usize_stack: Vec<usize>,

    /// for information about function call stack
    pub function_stack: Vec<FunctionCallStackElem>,

    /// current instruction counter
    pub counter: usize,
}
impl LuaThread {
    pub fn new() -> LuaThread {
        LuaThread {
            local_variables: Vec::new(),
            data_stack: Vec::new(),
            usize_stack: Vec::new(),
            function_stack: Vec::new(),
            counter: 0,
            bp: 0,
        }
    }

    pub fn top(&self) -> &LuaValue {
        self.data_stack.last().unwrap()
    }
    pub fn top_mut(&mut self) -> &mut LuaValue {
        self.data_stack.last_mut().unwrap()
    }
    pub fn pop(&mut self) -> Option<LuaValue> {
        self.data_stack.pop()
    }
    pub fn pop_n(&mut self, n: usize) -> impl Iterator<Item = LuaValue> + '_ {
        self.data_stack.drain(self.data_stack.len() - n..)
    }
    pub fn pop1(&mut self, n: usize) -> LuaValue {
        let mut it = self.pop_n(n);
        let v0 = it.next().unwrap();
        v0
    }
    pub fn pop2(&mut self, n: usize) -> (LuaValue, LuaValue) {
        let mut it = self.pop_n(n);
        let v0 = it.next().unwrap();
        let v1 = it.next().unwrap();
        (v0, v1)
    }
    pub fn pop3(&mut self, n: usize) -> (LuaValue, LuaValue, LuaValue) {
        let mut it = self.pop_n(n);
        let v0 = it.next().unwrap();
        let v1 = it.next().unwrap();
        let v2 = it.next().unwrap();
        (v0, v1, v2)
    }
    pub fn pop4(&mut self, n: usize) -> (LuaValue, LuaValue, LuaValue, LuaValue) {
        let mut it = self.pop_n(n);
        let v0 = it.next().unwrap();
        let v1 = it.next().unwrap();
        let v2 = it.next().unwrap();
        let v3 = it.next().unwrap();
        (v0, v1, v2, v3)
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub label_map: Vec<usize>,
    pub functions: Vec<FunctionInfo>,
    pub stack_size: usize,
}
impl Chunk {}
