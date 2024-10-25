use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use rand::SeedableRng;

use crate::builtin;
use crate::luaval::RefOrValue;
use crate::IntType;
use crate::LuaFunction;
use crate::LuaFunctionLua;
use crate::LuaString;
use crate::LuaTable;
use crate::LuaValue;

use crate::Instruction;
use crate::RuntimeError;

/// Main entry for Lua runtime.
/// It contains global environment, random number generator, coroutine stack, etc.
pub struct LuaEnv {
    /// _env
    pub(crate) env: LuaValue,
    /// random number generator
    pub(crate) rng: rand::rngs::StdRng,

    /// coroutine stack
    pub(crate) coroutines: Vec<Rc<RefCell<LuaThread>>>,

    /// last operation (for error message)
    pub(crate) last_op: String,

    pub(crate) parser: lua_parser::Parser,
    pub(crate) parser_context: Option<lua_parser::Context>,
    pub(crate) semantic_context: lua_semantics::Context,
}

impl LuaEnv {
    pub fn new() -> LuaEnv {
        let env = Rc::new(RefCell::new(builtin::init_env().unwrap()));
        env.borrow_mut()
            .insert("_G".into(), LuaValue::Table(Rc::clone(&env)));
        // let main_thread = Rc::new(RefCell::new(LuaThread::new_main(&chunk)));
        let mut semantic_context = lua_semantics::Context::new();
        semantic_context.begin_scope(false);
        LuaEnv {
            env: LuaValue::Table(env),
            rng: rand::rngs::StdRng::from_entropy(),

            coroutines: vec![],
            last_op: "last_op".to_string(),

            parser: lua_parser::Parser::new(),
            parser_context: None,
            semantic_context,
        }
    }

    pub(crate) fn main_thread(&self) -> &Rc<RefCell<LuaThread>> {
        self.coroutines.first().unwrap()
    }
    pub(crate) fn running_thread(&self) -> &Rc<RefCell<LuaThread>> {
        self.coroutines.last().unwrap()
    }

    /// Returns if there is uncompleted interpreter line, waiting for more input
    pub fn is_feed_pending(&self) -> bool {
        self.parser_context.is_some()
    }
    /// Clears uncompleted line
    pub fn clear_feed_pending(&mut self) {
        self.parser_context = None;
    }
    /// Feed lua source code for REPL interpreter.
    ///
    /// If `source` is not a complete line, nothing will be evaluated, and waiting for next `feed_line`.
    /// If `source` can be evaluated as a `Expression`, it will be evaluated and `print`ed.
    pub fn feed_line(&mut self, source: &[u8]) -> Result<(), RuntimeError> {
        if self.parser_context.is_none() {
            self.parser_context = Some(lua_parser::Context::new());
        }
        for token in lua_tokenizer::Tokenizer::from_bytes(source) {
            match token {
                Ok(token) => {
                    match self
                        .parser_context
                        .as_mut()
                        .unwrap()
                        .feed(&self.parser, token, &mut ())
                    {
                        Ok(_) => {}
                        Err(err) => {
                            self.clear_feed_pending();
                            return Err(RuntimeError::Custom(err.to_string().into()));
                        }
                    }
                }
                Err(err) => {
                    self.clear_feed_pending();
                    return Err(RuntimeError::TokenizeError(err));
                }
            }
        }

        if !self.parser_context.as_ref().unwrap().can_feed(
            &self.parser,
            &lua_tokenizer::Token::new_type(lua_tokenizer::TokenType::Eof),
        ) {
            return Ok(());
        }

        self.parser_context
            .as_mut()
            .unwrap()
            .feed(
                &self.parser,
                lua_tokenizer::Token::new_type(lua_tokenizer::TokenType::Eof),
                &mut (),
            )
            .ok();

        let mut matched_stmt = None;
        let mut matched_expr = None;
        for m in std::mem::take(&mut self.parser_context)
            .unwrap()
            .accept_all()
        {
            match m {
                lua_parser::ChunkOrExpressions::Chunk(chunk) => {
                    if matched_stmt.is_some() {
                        return Err(RuntimeError::Custom("ambiguous statement".into()));
                    }
                    matched_stmt = Some(chunk);
                }
                lua_parser::ChunkOrExpressions::Expressions(exprs) => {
                    if matched_expr.is_some() {
                        return Err(RuntimeError::Custom("ambiguous expression".into()));
                    }
                    matched_expr = Some(exprs);
                }
            }
        }
        // prefer expression to statement, if both are matched.
        // if expression matched, pass it to print() builtin function
        if let Some(matched_expr) = matched_expr {
            let args = lua_parser::FunctionCallArguments {
                args: matched_expr,
                span: lua_parser::Span::new_none(),
            };
            let print_name = lua_parser::ExprIdent::new(lua_parser::SpannedString::new(
                "print".to_string(),
                lua_parser::Span::new_none(),
            ));
            let function_call = lua_parser::StmtFunctionCall::new(
                lua_parser::Expression::Ident(print_name),
                None,
                args,
                lua_parser::Span::new_none(),
            );
            let function_call = lua_parser::Statement::FunctionCall(function_call);
            let blk =
                lua_parser::Block::new(vec![function_call], None, lua_parser::Span::new_none());
            matched_stmt = Some(blk);
        }
        if let Some(matched_stmt) = matched_stmt {
            let processed_block =
                match self
                    .semantic_context
                    .process_block(matched_stmt, true, false)
                {
                    Ok(res) => res,
                    Err(err) => {
                        return Err(RuntimeError::Custom(err.to_string().into()));
                    }
                };

            let ir_context = crate::Context::new();
            let chunk = ir_context.emit(processed_block);
            let thread = LuaThread::new_main(chunk);
            self.coroutines.push(Rc::new(RefCell::new(thread)));

            while !self.coroutines.is_empty() {
                match self.cycle() {
                    Ok(_) => {}
                    Err(err) => {
                        self.coroutines.clear();
                        return Err(err);
                    }
                }
            }
        }

        Ok(())
    }

    /// parse lua chunk from `source` and evaluate it.
    pub fn eval_chunk(&mut self, source: &[u8]) -> Result<(), RuntimeError> {
        self.parser_context = Some(lua_parser::Context::new());

        for token in lua_tokenizer::Tokenizer::from_bytes(source).chain(std::iter::once(Ok(
            lua_tokenizer::Token::new_type(lua_tokenizer::TokenType::Eof),
        ))) {
            match token {
                Ok(token) => {
                    match self
                        .parser_context
                        .as_mut()
                        .unwrap()
                        .feed(&self.parser, token, &mut ())
                    {
                        Ok(_) => {}
                        Err(err) => {
                            self.clear_feed_pending();
                            return Err(RuntimeError::Custom(err.to_string().into()));
                        }
                    }
                }
                Err(err) => {
                    self.clear_feed_pending();
                    return Err(RuntimeError::TokenizeError(err));
                }
            }
        }

        let mut matched_stmt = None;
        for m in std::mem::take(&mut self.parser_context)
            .unwrap()
            .accept_all()
        {
            match m {
                lua_parser::ChunkOrExpressions::Chunk(chunk) => {
                    if matched_stmt.is_some() {
                        return Err(RuntimeError::Custom("ambiguous statement".into()));
                    }
                    matched_stmt = Some(chunk);
                }
                lua_parser::ChunkOrExpressions::Expressions(_) => {}
            }
        }
        if let Some(matched_stmt) = matched_stmt {
            let processed_block =
                match self
                    .semantic_context
                    .process_block(matched_stmt, true, false)
                {
                    Ok(res) => res,
                    Err(err) => {
                        return Err(RuntimeError::Custom(err.to_string().into()));
                    }
                };

            let ir_context = crate::Context::new();
            let chunk = ir_context.emit(processed_block);
            let thread = LuaThread::new_main(chunk);
            self.coroutines.push(Rc::new(RefCell::new(thread)));

            while !self.coroutines.is_empty() {
                match self.cycle() {
                    Ok(_) => {}
                    Err(err) => {
                        self.coroutines.clear();
                        return Err(err);
                    }
                }
            }
        } else {
            return Err(RuntimeError::Custom("no statement found".into()));
        }

        Ok(())
    }

    /// Get global variable name `name`.
    pub fn get_global(&self, name: &str) -> LuaValue {
        match &self.env {
            LuaValue::Table(env) => {
                let name = LuaValue::String(LuaString::from_str(name));
                env.borrow().get(&name).cloned().unwrap_or(LuaValue::Nil)
            }
            _ => LuaValue::Nil,
        }
    }
    /// Set global variable name `name` to `value`
    /// Returns the old value of the variable, or `nil` if it doesn't exist.
    /// Settting a variable to `nil` is equivalent to deleting it.
    pub fn set_global(&mut self, name: &str, value: LuaValue) -> LuaValue {
        let key = LuaValue::String(LuaString::from_str(name));
        match &self.env {
            LuaValue::Table(env) => {
                if value == LuaValue::Nil {
                    env.borrow_mut().remove(&key).unwrap_or(LuaValue::Nil)
                } else {
                    env.borrow_mut().insert(key, value).unwrap_or(LuaValue::Nil)
                }
            }
            _ => LuaValue::Nil,
        }
    }

    pub fn push(&self, value: LuaValue) {
        self.running_thread().borrow_mut().data_stack.push(value);
    }
    pub(crate) fn push2(&self, value1: LuaValue, value2: LuaValue) {
        let mut thread = self.running_thread().borrow_mut();
        thread.data_stack.push(value1);
        thread.data_stack.push(value2);
    }
    pub(crate) fn push3(&self, value1: LuaValue, value2: LuaValue, value3: LuaValue) {
        let mut thread = self.running_thread().borrow_mut();
        thread.data_stack.push(value1);
        thread.data_stack.push(value2);
        thread.data_stack.push(value3);
    }
    pub fn pop(&self) -> LuaValue {
        self.running_thread().borrow_mut().data_stack.pop().unwrap()
    }
    pub(crate) fn pop2(&self) -> (LuaValue, LuaValue) {
        let mut thread = self.running_thread().borrow_mut();
        let value2 = thread.data_stack.pop().unwrap();
        let value1 = thread.data_stack.pop().unwrap();
        (value1, value2)
    }
    pub(crate) fn pop3(&self) -> (LuaValue, LuaValue, LuaValue) {
        let mut thread = self.running_thread().borrow_mut();
        let value3 = thread.data_stack.pop().unwrap();
        let value2 = thread.data_stack.pop().unwrap();
        let value1 = thread.data_stack.pop().unwrap();
        (value1, value2, value3)
    }
    pub(crate) fn pop4(&self) -> (LuaValue, LuaValue, LuaValue, LuaValue) {
        let mut thread = self.running_thread().borrow_mut();
        let value4 = thread.data_stack.pop().unwrap();
        let value3 = thread.data_stack.pop().unwrap();
        let value2 = thread.data_stack.pop().unwrap();
        let value1 = thread.data_stack.pop().unwrap();
        (value1, value2, value3, value4)
    }
    pub(crate) fn pop_n(&self, n: usize) {
        let mut thread_mut = self.running_thread().borrow_mut();
        let len = thread_mut.data_stack.len();
        thread_mut.data_stack.truncate(len - n);
    }
    /// get i'th value from top of the stack
    pub(crate) fn top_i(&self, i: usize) -> LuaValue {
        let thread = self.running_thread().borrow();
        let idx = thread.data_stack.len() - i - 1;
        thread.data_stack.get(idx).unwrap().clone()
    }
    pub(crate) fn borrow_running_thread(&self) -> std::cell::Ref<LuaThread> {
        self.running_thread().borrow()
    }
    pub(crate) fn borrow_running_thread_mut(&self) -> std::cell::RefMut<LuaThread> {
        self.running_thread().borrow_mut()
    }

    pub fn get_metavalue(&self, value: &LuaValue, key: &'static str) -> Option<LuaValue> {
        match value {
            // @TODO: link `string` module here
            LuaValue::String(_s) => {
                None
                // let s = String::from_utf8_lossy(s);
                // match key {
                //     "__name" => Some(LuaValue::String(s.as_bytes().to_vec())),
                //     _ => None,
                // }
            }
            LuaValue::Table(table) => table.borrow().get_metavalue(key),
            _ => None,
        }
    }

    /// Try to call binary metamethod f(lhs, rhs).
    /// It tries to search metamethod on lhs first, then rhs.
    fn try_call_metamethod(
        &mut self,
        lhs: LuaValue,
        rhs: LuaValue,
        meta_name: &'static str,
        error_wrapper: impl FnOnce(&'static str) -> RuntimeError,
    ) -> Result<(), RuntimeError> {
        match self.get_metavalue(&lhs, meta_name) {
            Some(meta) => {
                self.push2(lhs, rhs);
                self.function_call(2, meta, Some(1))?;
                Ok(())
            }
            None => match self.get_metavalue(&rhs, meta_name) {
                Some(meta) => {
                    self.push2(lhs, rhs);
                    self.function_call(2, meta, Some(1))?;
                    Ok(())
                }
                None => Err(error_wrapper(lhs.type_str())),
            },
        }
    }
    /// string-fy a value
    pub fn tostring(&mut self) -> Result<(), RuntimeError> {
        let top = self.pop();
        let meta = self.get_metavalue(&top, "__tostring");
        match meta {
            Some(meta) => {
                self.push(top);
                self.function_call(1, meta, Some(1))?;
                self.tostring()
            }
            _ => {
                let name = self.get_metavalue(&top, "__name");
                let s = match name {
                    Some(name) => match name {
                        LuaValue::String(name) => LuaValue::String(name),
                        _ => name.to_string().into(),
                    },
                    None => match top {
                        LuaValue::String(s) => LuaValue::String(s),
                        top => top.to_string().into(),
                    },
                };
                self.push(s);
                Ok(())
            }
        }
    }

    /// add operation with __add metamethod
    pub fn add(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            // if both are numbers, add them
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs + rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => {
                self.try_call_metamethod(lhs, rhs, "__add", RuntimeError::AttemptToArithmeticOn)
            }
        }
    }

    /// sub operation with __sub metamethod
    pub fn sub(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs - rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => {
                self.try_call_metamethod(lhs, rhs, "__sub", RuntimeError::AttemptToArithmeticOn)
            }
        }
    }
    /// mul operation with __mul metamethod
    pub fn mul(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs * rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => {
                self.try_call_metamethod(lhs, rhs, "__mul", RuntimeError::AttemptToArithmeticOn)
            }
        }
    }
    /// div operation with __div metamethod
    pub fn div(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs / rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => {
                self.try_call_metamethod(lhs, rhs, "__div", RuntimeError::AttemptToArithmeticOn)
            }
        }
    }
    /// mod operation with __mod metamethod
    pub fn mod_(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs % rhs).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => {
                self.try_call_metamethod(lhs, rhs, "__mod", RuntimeError::AttemptToArithmeticOn)
            }
        }
    }
    /// pow operation with __pow metamethod
    pub fn pow(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs.pow(rhs)).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => {
                self.try_call_metamethod(lhs, rhs, "__pow", RuntimeError::AttemptToArithmeticOn)
            }
        }
    }
    /// unary minus operation with __unm metamethod
    pub fn unm(&mut self) -> Result<(), RuntimeError> {
        let lhs = self.pop();
        match lhs {
            LuaValue::Number(num) => {
                self.push((-num).into());
                Ok(())
            }
            lhs => match self.get_metavalue(&lhs, "__unm") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    self.push2(lhs.clone(), lhs);
                    self.function_call(2, meta, Some(1))?;
                    Ok(())
                }
                _ => Err(RuntimeError::AttemptToArithmeticOn(lhs.type_str())),
            },
        }
    }
    /// floor division operation with __idiv metamethod
    pub fn idiv(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push((lhs.floor_div(rhs)).into());
                Ok(())
            }
            // else, try to call metamethod, search on left first
            (lhs, rhs) => {
                self.try_call_metamethod(lhs, rhs, "__idiv", RuntimeError::AttemptToArithmeticOn)
            }
        }
    }
    /// bitwise and operation with __band metamethod
    pub fn band(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Ok(lhs), Ok(rhs)) => {
                        self.push((lhs & rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(
                        lhs,
                        rhs,
                        "__band",
                        RuntimeError::AttemptToBitwiseOn,
                    ),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(lhs, rhs, "__band", RuntimeError::AttemptToBitwiseOn),
        }
    }
    /// bitwise or operation with __bor metamethod
    pub fn bor(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Ok(lhs), Ok(rhs)) => {
                        self.push((lhs | rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(
                        lhs,
                        rhs,
                        "__bor",
                        RuntimeError::AttemptToBitwiseOn,
                    ),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(lhs, rhs, "__bor", RuntimeError::AttemptToBitwiseOn),
        }
    }
    /// bitwise xor operation with __bxor metamethod
    pub fn bxor(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Ok(lhs), Ok(rhs)) => {
                        self.push((lhs ^ rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(
                        lhs,
                        rhs,
                        "__bxor",
                        RuntimeError::AttemptToBitwiseOn,
                    ),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(lhs, rhs, "__bxor", RuntimeError::AttemptToBitwiseOn),
        }
    }
    /// bitwise shift left operation with __shl metamethod
    pub fn shl(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Ok(lhs), Ok(rhs)) => {
                        self.push((lhs << rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(
                        lhs,
                        rhs,
                        "__shl",
                        RuntimeError::AttemptToBitwiseOn,
                    ),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(lhs, rhs, "__shl", RuntimeError::AttemptToBitwiseOn),
        }
    }
    /// bitwise shift right operation with __shr metamethod
    pub fn shr(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match (&lhs, &rhs) {
            (LuaValue::Number(lhs_num), LuaValue::Number(rhs_num)) => {
                match (lhs_num.try_to_int(), rhs_num.try_to_int()) {
                    (Ok(lhs), Ok(rhs)) => {
                        self.push((lhs >> rhs).into());
                        Ok(())
                    }
                    _ => self.try_call_metamethod(
                        lhs,
                        rhs,
                        "__shr",
                        RuntimeError::AttemptToBitwiseOn,
                    ),
                }
            }
            // else, try to call metamethod, search on left first
            _ => self.try_call_metamethod(lhs, rhs, "__shr", RuntimeError::AttemptToBitwiseOn),
        }
    }
    /// bitwise not operation with __bnot metamethod
    pub fn bnot(&mut self) -> Result<(), RuntimeError> {
        let lhs = self.pop();
        match &lhs {
            LuaValue::Number(lhs_num) => match lhs_num.try_to_int() {
                Ok(i) => {
                    self.push((!i).into());
                    Ok(())
                }
                _ => match self.get_metavalue(&lhs, "__bnot") {
                    Some(meta) => {
                        // For the unary operators (negation, length, and bitwise NOT),
                        // the metamethod is computed and called with a dummy second operand
                        // equal to the first one.
                        self.push2(lhs.clone(), lhs);
                        self.function_call(2, meta, Some(1))?;
                        Ok(())
                    }
                    _ => Err(RuntimeError::AttemptToBitwiseOn(lhs.type_str())),
                },
            },
            _ => match self.get_metavalue(&lhs, "__bnot") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    self.push2(lhs.clone(), lhs);
                    self.function_call(2, meta, Some(1))?;
                    Ok(())
                }
                _ => Err(RuntimeError::AttemptToBitwiseOn(lhs.type_str())),
            },
        }
    }
    /// concat operation with __concat metamethod
    pub fn concat(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();
        match lhs {
            LuaValue::Number(lhs_num) => match rhs {
                LuaValue::Number(rhs_num) => {
                    let mut concated = lhs_num.to_string().into_bytes();
                    concated.append(&mut rhs_num.to_string().into_bytes());
                    self.push(LuaString::from_vec(concated).into());
                    Ok(())
                }
                LuaValue::String(rhs) => {
                    let mut lhs = lhs_num.to_string().into_bytes();
                    lhs.extend_from_slice(rhs.as_bytes());
                    self.push(LuaString::from_vec(lhs).into());
                    Ok(())
                }
                _ => self.try_call_metamethod(
                    lhs,
                    rhs,
                    "__concat",
                    RuntimeError::AttemptToConcatenate,
                ),
            },

            LuaValue::String(lhs_str) => match rhs {
                LuaValue::Number(rhs_num) => {
                    let mut concated = lhs_str.into_vec();
                    concated.append(&mut rhs_num.to_string().into_bytes());
                    self.push(LuaString::from_vec(concated).into());
                    Ok(())
                }
                LuaValue::String(rhs) => {
                    let mut lhs = lhs_str.into_vec();
                    lhs.extend_from_slice(rhs.as_bytes());
                    self.push(LuaString::from_vec(lhs).into());
                    Ok(())
                }
                _ => self.try_call_metamethod(
                    LuaValue::String(lhs_str),
                    rhs,
                    "__concat",
                    RuntimeError::AttemptToConcatenate,
                ),
            },

            _ => self.try_call_metamethod(lhs, rhs, "__concat", RuntimeError::AttemptToConcatenate),
        }
    }
    /// `#` length operation with __len metamethod
    pub fn len(&mut self) -> Result<(), RuntimeError> {
        let lhs = self.pop();
        match lhs {
            LuaValue::String(s) => {
                self.push((s.len() as IntType).into());
                Ok(())
            }
            LuaValue::Table(table) => {
                let meta = table.borrow().get_metavalue("__len");
                match meta {
                    Some(meta) => {
                        // For the unary operators (negation, length, and bitwise NOT),
                        // the metamethod is computed and called with a dummy second operand
                        // equal to the first one.
                        self.push2(LuaValue::Table(Rc::clone(&table)), LuaValue::Table(table));
                        self.function_call(2, meta, Some(1))?;
                        Ok(())
                    }
                    _ => {
                        self.push((table.borrow().len() as IntType).into());
                        Ok(())
                    }
                }
            }
            lhs => match self.get_metavalue(&lhs, "__len") {
                Some(meta) => {
                    // For the unary operators (negation, length, and bitwise NOT),
                    // the metamethod is computed and called with a dummy second operand
                    // equal to the first one.
                    self.push2(lhs.clone(), lhs);
                    self.function_call(2, meta, Some(1))?;
                    Ok(())
                }
                _ => Err(RuntimeError::AttemptToGetLengthOf(lhs.type_str())),
            },
        }
    }
    /// table index get operation with __index metamethod
    pub fn index(&mut self) -> Result<(), RuntimeError> {
        let (table, key) = self.pop2();
        match table {
            LuaValue::Table(table) => {
                let get = table.borrow().get(&key).cloned();
                if let Some(get) = get {
                    self.push(get);
                    Ok(())
                } else {
                    let meta = table.borrow().get_metavalue("__index");
                    match meta {
                        Some(LuaValue::Function(meta_func)) => {
                            self.push2(LuaValue::Table(table), key);
                            self.function_call(2, LuaValue::Function(meta_func), Some(1))?;
                            Ok(())
                        }
                        Some(LuaValue::Table(meta_table)) => {
                            self.push2(LuaValue::Table(meta_table), key);
                            self.index()
                        }
                        _ => {
                            self.push(LuaValue::Nil);
                            Ok(())
                        }
                    }
                }
            }
            table => {
                let meta = self.get_metavalue(&table, "__index");
                match meta {
                    Some(LuaValue::Function(meta_func)) => {
                        self.push2(table, key);
                        self.function_call(2, LuaValue::Function(meta_func), Some(1))?;
                        Ok(())
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        self.push2(LuaValue::Table(meta_table), key);
                        self.index()
                    }
                    _ =>
                    // @TODO : error message
                    {
                        Err(RuntimeError::Custom("__index metamethod not found".into()))
                    }
                }
            }
        }
    }
    /// table index set operation with __newindex metamethod
    pub fn newindex(&mut self) -> Result<(), RuntimeError> {
        let (value, table, key) = self.pop3();

        match table {
            LuaValue::Table(table) => {
                {
                    let mut table_mut = table.borrow_mut();
                    if let Some(val) = table_mut.get_mut(&key) {
                        // if rhs is nil, remove the key
                        if value.is_nil() {
                            table_mut.remove(&key);
                        } else {
                            *val = value;
                        }
                        return Ok(());
                    }
                }
                let meta = table.borrow().get_metavalue("__newindex");
                match meta {
                    Some(LuaValue::Function(meta_func)) => {
                        self.push3(LuaValue::Table(table), key, value);
                        self.function_call(3, LuaValue::Function(meta_func), Some(0))?;
                        Ok(())
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        self.push3(value, LuaValue::Table(meta_table), key);
                        self.newindex()
                    }
                    _ => {
                        table.borrow_mut().insert(key, value);
                        Ok(())
                    }
                }
            }
            table => {
                let meta = self.get_metavalue(&table, "__newindex");
                match meta {
                    Some(LuaValue::Function(meta_func)) => {
                        self.push3(table, key, value);
                        self.function_call(3, LuaValue::Function(meta_func), Some(0))?;
                        Ok(())
                    }
                    Some(LuaValue::Table(meta_table)) => {
                        self.push3(value, LuaValue::Table(meta_table), key);
                        self.newindex()
                    }
                    _ => Err(RuntimeError::Custom(
                        "__newindex metamethod not found".into(),
                    )),
                }
            }
        }
    }
    /// equality operation with __eq metamethod
    pub fn eq(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();

        match (lhs, rhs) {
            (LuaValue::Table(lhs), LuaValue::Table(rhs)) => {
                if Rc::ptr_eq(&lhs, &rhs) {
                    self.push(LuaValue::Boolean(true));
                    return Ok(());
                } else {
                    // @TODO
                    self.try_call_metamethod(
                        LuaValue::Table(lhs),
                        LuaValue::Table(rhs),
                        "__eq",
                        RuntimeError::AttemptToArithmeticOn,
                    )
                }
            }
            (lhs, rhs) => {
                self.push(LuaValue::Boolean(lhs == rhs));
                Ok(())
            }
        }
    }
    /// less than operation with __lt metamethod
    pub fn lt(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();

        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push(LuaValue::Boolean(lhs < rhs));
                Ok(())
            }
            (LuaValue::String(lhs), LuaValue::String(rhs)) => {
                self.push(LuaValue::Boolean(lhs < rhs));
                Ok(())
            }
            // @TODO error type
            (lhs, rhs) => {
                self.try_call_metamethod(lhs, rhs, "__lt", RuntimeError::AttemptToArithmeticOn)
            }
        }
    }

    pub fn le(&mut self) -> Result<(), RuntimeError> {
        let (lhs, rhs) = self.pop2();

        match (lhs, rhs) {
            (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
                self.push(LuaValue::Boolean(lhs <= rhs));
                Ok(())
            }
            (LuaValue::String(lhs), LuaValue::String(rhs)) => {
                self.push(LuaValue::Boolean(lhs <= rhs));
                Ok(())
            }
            // @TODO error type
            (lhs, rhs) => {
                self.try_call_metamethod(lhs, rhs, "__le", RuntimeError::AttemptToArithmeticOn)
            }
        }
    }

    /// Function call with given function object.
    /// This could search for `__call` metamethod if the function object is not a function.
    pub fn function_call(
        &mut self,
        // number of arguments actually passed
        args_num: usize,
        // function object
        func: LuaValue,
        // number of return values expected; returned values will be adjusted to this number
        expected_ret: Option<usize>,
    ) -> Result<(), RuntimeError> {
        match func {
            LuaValue::Function(func) => {
                let func_borrow = func.borrow();
                match &*func_borrow {
                    LuaFunction::LuaFunc(lua_internal) => {
                        let mut thread_mut_ = self.running_thread().borrow_mut();
                        let thread_mut = &mut *thread_mut_;

                        // adjust function arguments
                        // extract variadic arguments if needed
                        if lua_internal.is_variadic {
                            let (variadic, rest_args_num) = if args_num <= lua_internal.args {
                                (Vec::new(), args_num)
                            } else {
                                (
                                    thread_mut
                                        .data_stack
                                        .drain(
                                            thread_mut.data_stack.len() - args_num
                                                + lua_internal.args..,
                                        )
                                        .collect(),
                                    lua_internal.args,
                                )
                            };

                            // push call stack frame
                            thread_mut.call_stack.push(CallStackFrame {
                                function: Rc::clone(&func),
                                usize_stack: thread_mut.usize_stack.len(),
                                return_expected: expected_ret,
                                variadic: variadic,
                                bp: thread_mut.bp,
                                counter: 0,
                                data_stack: thread_mut.data_stack.len() - rest_args_num,
                                local_variables: thread_mut.local_variables.len(),
                            });

                            // set base pointer to new stack frame
                            thread_mut.bp = thread_mut.local_variables.len();
                            // reserve stack space for local variables
                            thread_mut.local_variables.resize_with(
                                thread_mut.local_variables.len() + lua_internal.chunk.stack_size,
                                Default::default,
                            );

                            // copy arguments to local variables
                            for (idx, arg) in thread_mut
                                .data_stack
                                .drain(thread_mut.data_stack.len() - rest_args_num..)
                                .enumerate()
                            {
                                thread_mut.local_variables[thread_mut.bp + idx] =
                                    RefOrValue::Value(arg);
                            }
                        } else {
                            // push call stack frame
                            thread_mut.call_stack.push(CallStackFrame {
                                function: Rc::clone(&func),
                                usize_stack: thread_mut.usize_stack.len(),
                                return_expected: expected_ret,
                                variadic: Vec::new(),
                                bp: thread_mut.bp,
                                counter: 0,
                                data_stack: thread_mut.data_stack.len() - args_num,
                                local_variables: thread_mut.local_variables.len(),
                            });

                            // set base pointer to new stack frame
                            thread_mut.bp = thread_mut.local_variables.len();
                            // reserve stack space for local variables
                            thread_mut.local_variables.resize_with(
                                thread_mut.local_variables.len() + lua_internal.chunk.stack_size,
                                Default::default,
                            );

                            // copy arguments to local variables
                            for (idx, arg) in thread_mut
                                .data_stack
                                .drain(thread_mut.data_stack.len() - args_num..)
                                .take(lua_internal.args)
                                .enumerate()
                            {
                                thread_mut.local_variables[thread_mut.bp + idx] =
                                    RefOrValue::Value(arg);
                            }
                        };
                        drop(thread_mut_);
                        drop(func_borrow);
                        drop(func);

                        let coroutine_len = self.coroutines.len();
                        let call_stack_len = self.running_thread().borrow().call_stack.len();
                        loop {
                            if self.coroutines.len() < coroutine_len {
                                break;
                            } else if self.coroutines.len() == coroutine_len
                                && self.coroutines[coroutine_len - 1].borrow().call_stack.len()
                                    < call_stack_len
                            {
                                break;
                            }

                            self.cycle()?;
                        }
                        Ok(())
                    }
                    LuaFunction::RustFunc(rust_internal) => {
                        // self.running_thread()
                        //     .borrow_mut()
                        //     .call_stack
                        //     .push(CallStackFrame {
                        //         bp: 0,
                        //         counter: 0,
                        //         data_stack: 0,
                        //         function: Rc::clone(&func),
                        //         local_variables: 0,
                        //         return_expected: expected_ret,
                        //         usize_stack: 0,
                        //         variadic: Vec::new(),
                        //     });
                        rust_internal(self, args_num, expected_ret)?;
                        // self.running_thread().borrow_mut().call_stack.pop();
                        Ok(())
                    }
                }
            }
            other => {
                let func = self.get_metavalue(&other, "__call");
                if let Some(meta) = func {
                    {
                        // push `self` as first argument
                        let front_arg_pos =
                            self.running_thread().borrow().data_stack.len() - args_num;
                        self.running_thread()
                            .borrow_mut()
                            .data_stack
                            .insert(front_arg_pos, other);
                    }
                    self.function_call(args_num + 1, meta, expected_ret)
                } else {
                    // @TODO : error message
                    let msg = format!("__call metamethod not found for {}", other);
                    Err(RuntimeError::Custom(msg.into()))
                }
            }
        }
    }

    /// execute single instruction
    pub fn run_instruction(&mut self, instruction: Instruction) -> Result<(), RuntimeError> {
        debug_assert!(self.coroutines.is_empty() == false);
        match instruction {
            Instruction::Clone => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let top = thread_mut.data_stack.last().unwrap().clone();
                thread_mut.data_stack.push(top);
            }
            Instruction::Sp => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let len = thread_mut.data_stack.len();
                thread_mut.usize_stack.push(len);
            }
            Instruction::Pop => {
                self.pop();
            }
            Instruction::Deref => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let sp = *thread_mut.usize_stack.last().unwrap();
                let top = thread_mut.data_stack[sp].clone();
                thread_mut.data_stack.push(top);
            }
            Instruction::Jump(label) => {
                let mut thread = self.running_thread().borrow_mut();
                let frame = thread.call_stack.last_mut().unwrap();
                let func = frame.function.borrow();
                let next_pc = match &*func {
                    LuaFunction::LuaFunc(f) => f.chunk.label_map.get(label).unwrap(),
                    _ => unreachable!("jump from non-lua function"),
                };
                frame.counter = *next_pc;
            }
            Instruction::JumpTrue(label) => {
                let mut thread = self.running_thread().borrow_mut();
                let top = thread.data_stack.pop().unwrap().to_bool();
                if top {
                    let frame = thread.call_stack.last_mut().unwrap();
                    let func = frame.function.borrow();
                    let next_pc = match &*func {
                        LuaFunction::LuaFunc(f) => f.chunk.label_map.get(label).unwrap(),
                        _ => unreachable!("jump from non-lua function"),
                    };
                    frame.counter = *next_pc;
                }
            }
            Instruction::JumpFalse(label) => {
                let mut thread = self.running_thread().borrow_mut();
                let top = thread.data_stack.pop().unwrap().to_bool();
                if !top {
                    let frame = thread.call_stack.last_mut().unwrap();
                    let func = frame.function.borrow();
                    let next_pc = match &*func {
                        LuaFunction::LuaFunc(f) => f.chunk.label_map.get(label).unwrap(),
                        _ => unreachable!("jump from non-lua function"),
                    };
                    frame.counter = *next_pc;
                }
            }
            Instruction::GetLocalVariable(local_id, name) => {
                self.last_op = name;
                let mut thread_mut = self.running_thread().borrow_mut();
                let local_idx = local_id + thread_mut.bp;
                let val = match thread_mut.local_variables.get(local_idx).unwrap() {
                    RefOrValue::Ref(val) => val.borrow().clone(),
                    RefOrValue::Value(val) => val.clone(),
                };
                thread_mut.data_stack.push(val);
            }
            Instruction::SetLocalVariable(local_id) => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let top = thread_mut.data_stack.pop().unwrap();
                let local_idx = local_id + thread_mut.bp;
                match thread_mut.local_variables.get_mut(local_idx).unwrap() {
                    RefOrValue::Ref(val) => {
                        val.replace(top);
                    }
                    RefOrValue::Value(val) => {
                        *val = top;
                    }
                }
            }
            Instruction::InitLocalVariable(local_id) => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let top = thread_mut.data_stack.pop().unwrap();
                let local_idx = local_id + thread_mut.bp;
                if thread_mut.local_variables.len() <= local_idx {
                    let gap = local_idx - thread_mut.local_variables.len() + 1;
                    let new_len = thread_mut.local_variables.len() + 2 * gap;
                    thread_mut
                        .local_variables
                        .resize_with(new_len, Default::default);
                }
                *thread_mut.local_variables.get_mut(local_idx).unwrap() = RefOrValue::Value(top);
            }
            Instruction::IsNil => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let top = thread_mut.data_stack.pop().unwrap();
                thread_mut.data_stack.push(LuaValue::Boolean(top.is_nil()));
            }

            Instruction::Nil => {
                self.push(LuaValue::Nil);
            }
            Instruction::Boolean(b) => {
                self.push(LuaValue::Boolean(b));
            }
            Instruction::Numeric(n) => {
                self.push(LuaValue::Number(n));
            }
            Instruction::String(s) => {
                self.last_op = String::from_utf8_lossy(&s).to_string();
                self.push(LuaString::from_vec(s).into());
            }
            Instruction::GetEnv => {
                let env = self.env.clone();
                self.push(env);
            }
            Instruction::TableInit(cap) => {
                let table = LuaTable::with_capacity(cap);
                self.push(table.into());
            }
            Instruction::TableIndexInit => {
                let (table, index, value) = self.pop3();
                if let LuaValue::Table(table) = table {
                    table.borrow_mut().insert(index, value);
                    self.push(LuaValue::Table(table));
                } else {
                    unreachable!("table must be on top of stack");
                }
            }
            Instruction::TableInitLast(start_key) => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let sp = thread_mut.usize_stack.pop().unwrap();
                let mut values: BTreeMap<_, _> = thread_mut
                    .data_stack
                    .drain(sp..)
                    .enumerate()
                    .map(|(idx, value)| {
                        let index = idx as IntType + start_key;
                        (index.into(), value)
                    })
                    .collect();
                if let LuaValue::Table(table) = thread_mut.data_stack.last().unwrap() {
                    table.borrow_mut().arr.append(&mut values);
                } else {
                    unreachable!("table must be on top of stack");
                }
            }

            Instruction::TableIndex => {
                self.index()?;
            }
            Instruction::TableIndexSet => {
                self.newindex()?;
            }

            Instruction::FunctionInit(func) => {
                self.push(LuaFunction::LuaFunc(*func).into());
            }
            Instruction::FunctionInitUpvalueFromLocalVar(src_local_id) => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let local_idx = src_local_id + thread_mut.bp;
                let local_var = thread_mut.local_variables.get_mut(local_idx).unwrap();
                // upvalue must be reference.
                let local_var = match local_var {
                    RefOrValue::Ref(r) => Rc::clone(r),
                    RefOrValue::Value(v) => {
                        let reffed_var = Rc::new(RefCell::new(v.clone()));
                        *local_var = RefOrValue::Ref(Rc::clone(&reffed_var));
                        reffed_var
                    }
                };
                match thread_mut.data_stack.last().unwrap() {
                    LuaValue::Function(func) => match &mut *func.borrow_mut() {
                        LuaFunction::LuaFunc(f) => {
                            f.upvalues.push(local_var);
                        }
                        _ => unreachable!("stack top must be function"),
                    },
                    _ => unreachable!("stack top must be function"),
                }
            }
            Instruction::FunctionInitUpvalueFromUpvalue(src_upvalue_id) => {
                let thread = self.running_thread().borrow();
                let func = thread.call_stack.last().unwrap().function.borrow();

                let val = match &*func {
                    LuaFunction::LuaFunc(f) => Rc::clone(&f.upvalues[src_upvalue_id]),
                    _ => {
                        unreachable!("function must be LuaFunc");
                    }
                };
                drop(func);
                drop(thread);

                match self.running_thread().borrow().data_stack.last().unwrap() {
                    LuaValue::Function(func) => match &mut *func.borrow_mut() {
                        LuaFunction::LuaFunc(f) => {
                            f.upvalues.push(val);
                        }
                        _ => unreachable!("stack top must be function"),
                    },
                    _ => unreachable!("stack top must be function"),
                }
            }

            Instruction::FunctionUpvalue(upvalue_id, name) => {
                self.last_op = name;
                let thread = self.running_thread().borrow();
                let func = thread.call_stack.last().unwrap().function.borrow();

                let val = match &*func {
                    LuaFunction::LuaFunc(f) => RefCell::borrow(&f.upvalues[upvalue_id]).clone(),
                    _ => {
                        unreachable!("function must be LuaFunc");
                    }
                };
                drop(func);
                drop(thread);
                self.push(val);
            }
            Instruction::FunctionUpvalueSet(upvalue_id) => {
                let top = self.pop();
                let thread = self.running_thread().borrow();
                let mut func = thread.call_stack.last().unwrap().function.borrow_mut();
                match &mut *func {
                    LuaFunction::LuaFunc(f) => {
                        f.upvalues[upvalue_id] = Rc::new(RefCell::new(top));
                    }
                    _ => {
                        unreachable!("function must be LuaFunc");
                    }
                }
            }

            Instruction::BinaryAdd => {
                self.add()?;
            }
            Instruction::BinarySub => {
                self.sub()?;
            }
            Instruction::BinaryMul => {
                self.mul()?;
            }
            Instruction::BinaryDiv => {
                self.div()?;
            }
            Instruction::BinaryFloorDiv => {
                self.idiv()?;
            }
            Instruction::BinaryMod => {
                self.mod_()?;
            }
            Instruction::BinaryPow => {
                self.pow()?;
            }
            Instruction::BinaryConcat => {
                self.concat()?;
            }
            Instruction::BinaryBitwiseAnd => {
                self.band()?;
            }
            Instruction::BinaryBitwiseOr => {
                self.bor()?;
            }
            Instruction::BinaryBitwiseXor => {
                self.bxor()?;
            }
            Instruction::BinaryShiftLeft => {
                self.shl()?;
            }
            Instruction::BinaryShiftRight => {
                self.shr()?;
            }
            Instruction::BinaryEqual => {
                self.eq()?;
            }
            Instruction::BinaryLessThan => {
                self.lt()?;
            }
            Instruction::BinaryLessEqual => {
                self.le()?;
            }

            Instruction::UnaryMinus => {
                self.unm()?;
            }
            Instruction::UnaryBitwiseNot => {
                self.bnot()?;
            }
            Instruction::UnaryLength => {
                self.len()?;
            }
            Instruction::UnaryLogicalNot => {
                let top = self.pop().to_bool();
                self.push((!top).into());
            }

            Instruction::FunctionCall(expected_ret) => {
                let (func, num_args) = {
                    let mut thread_mut = self.running_thread().borrow_mut();
                    let func = thread_mut.data_stack.pop().unwrap();
                    let sp = thread_mut.usize_stack.pop().unwrap();
                    let num_args = thread_mut.data_stack.len() - sp;
                    (func, num_args)
                };
                self.function_call(num_args, func, expected_ret)?;
            }

            Instruction::Return => {
                let mut thread_mut = self.running_thread().borrow_mut();
                let frame = thread_mut.call_stack.pop().unwrap();
                if thread_mut.call_stack.is_empty() {
                    // end this thread
                    thread_mut.set_dead();
                    drop(thread_mut);
                    let yield_thread = self.coroutines.pop().unwrap();

                    if !self.coroutines.is_empty() {
                        let mut yield_thread_mut = yield_thread.borrow_mut();
                        let mut resume_thread_mut = self.running_thread().borrow_mut();

                        let return_args_num = yield_thread_mut.data_stack.len() - frame.data_stack;
                        let resume_expected = match resume_thread_mut.status {
                            ThreadStatus::ResumePending(expected) => expected,
                            _ => unreachable!("coroutine must be in resume pending state"),
                        };
                        resume_thread_mut.status = ThreadStatus::Running;
                        resume_thread_mut.data_stack.push(true.into());
                        resume_thread_mut
                            .data_stack
                            .extend(yield_thread_mut.data_stack.drain(frame.data_stack..));
                        if let Some(resume_expected) = resume_expected {
                            let adjusted = resume_thread_mut.data_stack.len() - return_args_num - 1
                                + resume_expected;
                            resume_thread_mut
                                .data_stack
                                .resize_with(adjusted, Default::default);
                        }
                    }
                } else {
                    // return from function call
                    thread_mut.local_variables.truncate(frame.local_variables);
                    thread_mut.usize_stack.truncate(frame.usize_stack);
                    thread_mut.bp = frame.bp;
                    if let Some(expected) = frame.return_expected {
                        let adjusted = frame.data_stack + expected;
                        thread_mut
                            .data_stack
                            .resize_with(adjusted, Default::default);
                    }
                }
            }

            Instruction::GetVariadic(expected) => {
                if let Some(expected) = expected {
                    let mut variadic = self
                        .running_thread()
                        .borrow()
                        .call_stack
                        .last()
                        .unwrap()
                        .variadic
                        .clone();
                    variadic.resize_with(expected, Default::default);
                    self.running_thread()
                        .borrow_mut()
                        .data_stack
                        .append(&mut variadic);
                } else {
                    let mut variadic = self
                        .running_thread()
                        .borrow()
                        .call_stack
                        .last()
                        .unwrap()
                        .variadic
                        .clone();
                    self.running_thread()
                        .borrow_mut()
                        .data_stack
                        .append(&mut variadic);
                }
            }
        }
        Ok(())
    }

    /// run single instruction
    pub fn cycle(&mut self) -> Result<(), RuntimeError> {
        if self.coroutines.len() == 0 {
            return Ok(());
        }
        let mut thread_mut = self.running_thread().borrow_mut();
        let frame_mut = thread_mut.call_stack.last_mut().unwrap();
        let func = frame_mut.function.borrow();
        match &*func {
            LuaFunction::LuaFunc(f) => {
                if let Some(instruction) = f.chunk.instructions.get(frame_mut.counter).cloned() {
                    frame_mut.counter += 1;
                    drop(func);
                    drop(thread_mut);
                    match self.run_instruction(instruction.clone()) {
                        Ok(_) => return Ok(()),
                        Err(err) => {
                            // if this error was occured in main chunk, just return it
                            if self.coroutines.len() == 1 {
                                return Err(err);
                            } else {
                                // if this error was occured in coroutine, propagate it to parent coroutine
                                let error_object = err.into_lua_value(self);

                                // return 'false' and 'error_object' to parent's 'resume()'
                                self.coroutines.pop().unwrap().borrow_mut().set_dead();
                                let status = self.running_thread().borrow().status;
                                if let ThreadStatus::ResumePending(resume_expected) = status {
                                    match resume_expected {
                                        Some(0) => {}
                                        Some(1) => {
                                            self.running_thread()
                                                .borrow_mut()
                                                .data_stack
                                                .push(false.into());
                                        }
                                        Some(resume_expected) => {
                                            self.push2(false.into(), error_object);
                                            self.running_thread().borrow_mut().data_stack.extend(
                                                std::iter::repeat(LuaValue::Nil)
                                                    .take(resume_expected - 2),
                                            );
                                        }
                                        None => {
                                            self.push2(false.into(), error_object);
                                        }
                                    }
                                } else {
                                    unreachable!("coroutine must be in resume pending state");
                                }
                                self.running_thread().borrow_mut().status = ThreadStatus::Running;
                            }
                        }
                    }
                    return Ok(());
                } else {
                    return Ok(());
                }
            }
            _ => unimplemented!("cycle: function call from non-lua function"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CallStackFrame {
    pub function: Rc<RefCell<LuaFunction>>,
    /// current instruction counter
    pub counter: usize,

    /// number of return values expected.
    pub return_expected: Option<usize>,
    /// variadic arguments
    pub variadic: Vec<LuaValue>,

    /// data_stack.len() to restore when return
    pub data_stack: usize,
    /// bp to restore when return
    pub bp: usize,
    /// local_variables.len() to restore when return
    pub local_variables: usize,
    // usize_stack.len() to restore when return
    pub usize_stack: usize,
}

/// Status for Lua thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreadStatus {
    /// thread is not started yet
    NotStarted,
    /// thread is running.
    /// it is not pending on any operation.
    Running,
    /// thread is running, and it is pending on `resume`.
    /// value is the expected number of return values from `resume`.
    ResumePending(Option<usize>),
    /// thread is not running, and it is pending on `yield`.
    /// that is, the thread is waiting for any parent coroutine to `resume`.
    /// value is the expected number of return values from `yield`.
    YieldPending(Option<usize>),
    /// thread is dead, and it is not running anymore.
    Dead,
}
impl ThreadStatus {
    pub fn is_started(&self) -> bool {
        !matches!(self, ThreadStatus::NotStarted)
    }
    pub fn is_dead(&self) -> bool {
        matches!(self, ThreadStatus::Dead)
    }
    pub fn is_yield_pending(&self) -> bool {
        matches!(self, ThreadStatus::YieldPending(_))
    }
    pub fn is_resume_pending(&self) -> bool {
        matches!(self, ThreadStatus::ResumePending(_))
    }
}

/// for error handling, recovering state
#[derive(Debug, Clone)]
pub struct ThreadState {
    pub local_variables: usize,
    pub data_stack: usize,
    pub usize_stack: usize,
    pub call_stack: usize,
    pub bp: usize,
}

/// Type for Lua thread.
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

    // function object, variadic, return values multire expected count
    pub call_stack: Vec<CallStackFrame>,

    pub status: ThreadStatus,

    /// If this thread is created by `coroutine.create`, this field is Some.
    /// The function object of the coroutine.
    pub function: Option<Rc<RefCell<LuaFunction>>>,
}
impl LuaThread {
    pub fn new_main(chunk: Chunk) -> LuaThread {
        let mut local_variables = Vec::new();
        local_variables.resize_with(chunk.stack_size, Default::default);
        let func = LuaFunctionLua {
            args: 0,
            chunk,
            is_variadic: false,
            upvalues: Vec::new(),
        };
        let func = Rc::new(RefCell::new(LuaFunction::LuaFunc(func)));
        let frame = CallStackFrame {
            bp: 0,
            counter: 0,
            function: func,
            return_expected: None,
            variadic: Vec::new(),
            data_stack: 0,
            local_variables: 0,
            usize_stack: 0,
        };

        LuaThread {
            local_variables,
            data_stack: Vec::new(),
            usize_stack: Vec::new(),
            call_stack: vec![frame],
            bp: 0,
            status: ThreadStatus::Running,
            function: None,
        }
    }
    pub fn new_coroutine(_env: &LuaEnv, func: Rc<RefCell<LuaFunction>>) -> LuaThread {
        LuaThread {
            local_variables: Vec::new(),
            data_stack: Vec::new(),
            usize_stack: Vec::new(),
            call_stack: Vec::new(),
            function: Some(func),
            bp: 0,
            status: ThreadStatus::NotStarted,
        }
    }

    pub(crate) fn to_state(&self) -> ThreadState {
        ThreadState {
            local_variables: self.local_variables.len(),
            data_stack: self.data_stack.len(),
            usize_stack: self.usize_stack.len(),
            call_stack: self.call_stack.len(),
            bp: self.bp,
        }
    }
    pub(crate) fn from_state(&mut self, state: ThreadState) {
        debug_assert!(self.status == ThreadStatus::Running);
        self.local_variables.truncate(state.local_variables);
        self.data_stack.truncate(state.data_stack);
        self.usize_stack.truncate(state.usize_stack);
        self.call_stack.truncate(state.call_stack);
        self.bp = state.bp;
    }

    pub fn drain_last(&mut self, n: usize) -> impl Iterator<Item = LuaValue> + '_ {
        self.data_stack.drain(self.data_stack.len() - n..)
    }

    pub fn set_dead(&mut self) {
        self.status = ThreadStatus::Dead;
    }
    pub fn is_dead(&self) -> bool {
        self.status == ThreadStatus::Dead
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub label_map: Vec<usize>,
    pub stack_size: usize,
}
impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            instructions: Vec::new(),
            label_map: Vec::new(),
            stack_size: 0,
        }
    }
}
