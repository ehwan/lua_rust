use crate::scope::UpvalueInfo;
use crate::ExprLocalVariable;
use crate::LabelInfo;
use crate::ProcessError;
use crate::Scope;
use crate::ScopeBlock;
use crate::ScopeFunction;
use crate::VariableInfo;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Context {
    pub scopes: Vec<Scope>,
    pub scope_counter: usize,

    pub labels: HashMap<String, Rc<RefCell<LabelInfo>>>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            scopes: Vec::new(),
            scope_counter: 0,
            labels: Default::default(),
        }
    }

    /// return index on local stack for new variable
    fn new_offset(&self) -> usize {
        for parent in self.scopes.iter().rev() {
            match parent {
                Scope::Block(blk) => return blk.offset + blk.variables.len(),
                Scope::Function(_) => return 0,
            }
        }
        0
    }
    pub fn begin_scope(&mut self, is_loop: bool) {
        let offset = self.new_offset();
        self.scope_counter += 1;
        self.scopes.push(Scope::Block(ScopeBlock {
            id: self.scope_counter,
            max_variables: 0,
            offset,
            variables: Vec::new(),
            is_loop,
            labels: Vec::new(),
        }));
    }
    /// close all local variable scopes, count up variables and re calculate stack size.
    fn end_scope(&mut self) -> Scope {
        let scope = self.scopes.pop().unwrap();

        if let Scope::Block(blk) = &scope {
            for label in blk.labels.iter() {
                self.labels.remove(label);
            }
            match self.scopes.last_mut() {
                Some(Scope::Block(parent)) => {
                    let vars = parent.variables.len() + blk.max_variables;
                    parent.max_variables = parent.max_variables.max(vars);
                }
                Some(Scope::Function(parent)) => {
                    parent.max_variables = parent.max_variables.max(blk.max_variables);
                }
                _ => {}
            }
        } else {
            unreachable!("end_scope - block scope not opened?");
        }

        scope
    }
    /// return local stack offset
    fn begin_variable_scope(&mut self, name: String) -> Rc<RefCell<VariableInfo>> {
        if let Some(Scope::Block(blk)) = self.scopes.last_mut() {
            let offset = blk.offset + blk.variables.len();
            let varinfo = Rc::new(RefCell::new(VariableInfo {
                name,
                is_reference: false,
                offset,
            }));
            blk.variables.push(Rc::clone(&varinfo));
            blk.max_variables = blk.max_variables.max(blk.variables.len());
            varinfo
        } else {
            unreachable!("begin_variable_scope - block scope not opened?");
        }
    }
    /// search for local variable name `name`
    fn search_local_variable(&mut self, name: &str) -> Option<ExprLocalVariable> {
        let mut function_scopes = Vec::new();
        let mut found = None;
        'a: for scope in self.scopes.iter_mut().rev() {
            match scope {
                Scope::Block(blk) => {
                    for var in blk.variables.iter().rev() {
                        if var.borrow().name == name {
                            found = Some(ExprLocalVariable::Stack(
                                var.borrow().offset,
                                name.to_string(),
                            ));
                            break 'a;
                        }
                    }
                }
                Scope::Function(func) => {
                    for (upvalue_idx, upvalue) in func.upvalues.iter().enumerate() {
                        if upvalue.name.as_str() == name {
                            found = Some(ExprLocalVariable::Upvalue(upvalue_idx, name.to_string()));
                            break 'a;
                        }
                    }
                    function_scopes.push(func);
                }
            }
        }
        if let Some(mut found) = found {
            for scope in function_scopes.into_iter().rev() {
                let upvalue_idx = scope.upvalues.len();
                scope.upvalues.push(UpvalueInfo {
                    name: name.to_string(),
                    from: found,
                });
                found = ExprLocalVariable::Upvalue(upvalue_idx, name.to_string());
            }
            Some(found)
        } else {
            None
        }
    }
    fn begin_function_scope(&mut self, variadic: bool) {
        self.scope_counter += 1;
        self.scopes.push(Scope::Function(ScopeFunction {
            id: self.scope_counter,
            max_variables: 0,
            upvalues: Vec::new(),
            variadic,
        }));
    }
    fn end_function_scope(&mut self) -> ScopeFunction {
        if let Some(Scope::Function(scope)) = self.scopes.pop() {
            scope
        } else {
            unreachable!("end_function_scope - function scope not opened? - 2");
        }
    }
    fn nearest_function_scope(&mut self) -> Option<&mut ScopeFunction> {
        for scope in self.scopes.iter_mut().rev() {
            match scope {
                Scope::Function(func) => return Some(func),
                _ => {}
            }
        }
        None
    }

    fn scope_tree(&self) -> Vec<usize> {
        let mut tree = Vec::new();
        for scope in self.scopes.iter().rev() {
            match scope {
                Scope::Block(blk) => tree.push(blk.id),
                Scope::Function(_) => break,
            }
        }
        tree.reverse();
        tree
    }

    // pub fn process(&mut self, block: lua_parser::Block) -> Result<crate::Block, ProcessError> {
    //     self.begin_scope(false);
    //     self.process_block(block, false, false)
    // }

    pub fn process_block(
        &mut self,
        block: lua_parser::Block,
        make_scope: bool,
        is_loop: bool,
    ) -> Result<crate::Block, ProcessError> {
        if make_scope {
            self.begin_scope(is_loop);
        }

        let mut blk = crate::Block::new(Vec::with_capacity(block.statements.len()), None, None);
        for stmt in block.statements.into_iter() {
            self.process_statement(stmt, &mut blk)?;
        }
        if let Some(ret) = block.return_statement {
            self.process_return_statement(ret, &mut blk)?;
        }

        if make_scope {
            let scope = self.end_scope();
            match scope {
                Scope::Block(scope) => {
                    blk.stack_size = Some(scope.max_variables);
                }
                Scope::Function(_) => {
                    unreachable!("process_block - function scope not closed?");
                }
            }
        }
        Ok(blk)
    }
    fn process_statement(
        &mut self,
        stmt: lua_parser::Statement,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        match stmt {
            lua_parser::Statement::None(_) => {
                // do nothing
            }
            lua_parser::Statement::Assignment(v) => self.process_statement_assignment(v, blk)?,
            lua_parser::Statement::Label(v) => self.process_statement_label(v, blk)?,
            lua_parser::Statement::Break(v) => self.process_statement_break(v, blk)?,
            lua_parser::Statement::Goto(v) => self.process_statement_goto(v, blk)?,
            lua_parser::Statement::Do(v) => self.process_statement_do(v, blk)?,
            lua_parser::Statement::While(v) => self.process_statement_while(v, blk)?,
            lua_parser::Statement::Repeat(v) => self.process_statement_repeat(v, blk)?,
            lua_parser::Statement::If(v) => self.process_statement_if(v, blk)?,
            lua_parser::Statement::For(v) => self.process_statement_for(v, blk)?,
            lua_parser::Statement::ForGeneric(v) => self.process_statement_for_generic(v, blk)?,
            lua_parser::Statement::LocalDeclaration(v) => {
                self.process_statement_local_decl(v, blk)?
            }
            lua_parser::Statement::FunctionDefinition(v) => {
                self.process_statement_function_def(v, blk)?
            }
            lua_parser::Statement::FunctionDefinitionLocal(v) => {
                self.process_statement_function_def_local(v, blk)?
            }
            lua_parser::Statement::FunctionCall(v) => {
                self.process_statement_function_call(v, blk)?
            }

            _ => {
                let message = format!("{:?}", stmt);
                unimplemented!("{}", message);
            }
        }
        Ok(())
    }
    fn process_expression(
        &mut self,
        expr: lua_parser::Expression,
    ) -> Result<crate::Expression, ProcessError> {
        match expr {
            lua_parser::Expression::Nil(v) => self.process_expression_nil(v),
            lua_parser::Expression::Numeric(v) => self.process_expression_numeric(v),
            lua_parser::Expression::String(v) => self.process_expression_string(v),
            lua_parser::Expression::Bool(v) => self.process_expression_bool(v),
            lua_parser::Expression::Variadic(v) => self.process_expression_variadic(v),
            lua_parser::Expression::Ident(v) => self.process_expression_ident(v),
            lua_parser::Expression::Table(v) => self.process_expression_table(v),
            lua_parser::Expression::Function(v) => self.process_expression_function(v),
            lua_parser::Expression::FunctionCall(v) => self.process_expression_function_call(v),
            lua_parser::Expression::TableIndex(v) => self.process_expression_table_index(v),
            lua_parser::Expression::Binary(v) => self.process_expression_binary(v),
            lua_parser::Expression::Unary(v) => self.process_expression_unary(v),

            _ => {
                let message = format!("{:?}", expr);
                unimplemented!("{}", message);
            }
        }
    }
}

impl Context {
    fn process_expression_binary(
        &mut self,
        expr: lua_parser::ExprBinary,
    ) -> Result<crate::Expression, ProcessError> {
        match expr {
            lua_parser::ExprBinary::Add(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::Add(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::Sub(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::Sub(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::Mul(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::Mul(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::Div(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::Div(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::FloorDiv(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::FloorDiv(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::Mod(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::Mod(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::Pow(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::Pow(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::Concat(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::Concat(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::LessThan(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::LessThan(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::LessEqual(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::LessEqual(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::GreaterThan(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::GreaterThan(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::GreaterEqual(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::GreaterEqual(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::Equal(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::Equal(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::NotEqual(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::NotEqual(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::LogicalAnd(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::LogicalAnd(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::LogicalOr(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::LogicalOr(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::BitwiseAnd(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::BitwiseAnd(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::BitwiseOr(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::BitwiseOr(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::BitwiseXor(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::BitwiseXor(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::ShiftLeft(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::ShiftLeft(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
            lua_parser::ExprBinary::ShiftRight(v) => {
                let lhs = self.process_expression(*v.lhs)?;
                let rhs = self.process_expression(*v.rhs)?;
                Ok(crate::Expression::Binary(crate::ExprBinary::ShiftRight(
                    crate::ExprBinaryData::new(lhs, rhs),
                )))
            }
        }
    }
    fn process_expression_unary(
        &mut self,
        expr: lua_parser::ExprUnary,
    ) -> Result<crate::Expression, ProcessError> {
        match expr {
            lua_parser::ExprUnary::Minus(v) => {
                let expr = self.process_expression(*v.value)?;
                Ok(crate::Expression::Unary(crate::ExprUnary::Minus(
                    crate::ExprUnaryData::new(expr),
                )))
            }
            lua_parser::ExprUnary::Plus(v) => {
                // @TODO
                // only for numeric
                let expr = self.process_expression(*v.value)?;
                Ok(expr)
            }
            lua_parser::ExprUnary::Length(v) => {
                let expr = self.process_expression(*v.value)?;
                Ok(crate::Expression::Unary(crate::ExprUnary::Length(
                    crate::ExprUnaryData::new(expr),
                )))
            }
            lua_parser::ExprUnary::BitwiseNot(v) => {
                let expr = self.process_expression(*v.value)?;
                Ok(crate::Expression::Unary(crate::ExprUnary::BitwiseNot(
                    crate::ExprUnaryData::new(expr),
                )))
            }
            lua_parser::ExprUnary::LogicalNot(v) => {
                let expr = self.process_expression(*v.value)?;
                Ok(crate::Expression::Unary(crate::ExprUnary::LogicalNot(
                    crate::ExprUnaryData::new(expr),
                )))
            }
        }
    }

    fn process_expression_function_call(
        &mut self,
        expr: lua_parser::ExprFunctionCall,
    ) -> Result<crate::Expression, ProcessError> {
        let prefix = self.process_expression(*expr.prefix)?;
        let mut args = Vec::with_capacity(expr.args.args.len());
        for arg in expr.args.args.into_iter() {
            args.push(self.process_expression(arg)?);
        }
        let method = expr.method.map(|s| s.string);
        Ok(crate::Expression::FunctionCall(
            crate::ExprFunctionCall::new(prefix, method, args),
        ))
    }
    fn process_expression_table_index(
        &mut self,
        expr: lua_parser::ExprTableIndex,
    ) -> Result<crate::Expression, ProcessError> {
        let table = self.process_expression(*expr.table)?;
        let index = self.process_expression(*expr.index)?;
        Ok(crate::Expression::TableIndex(crate::ExprTableIndex::new(
            table, index,
        )))
    }
    fn process_expression_table(
        &mut self,
        expr: lua_parser::ExprTable,
    ) -> Result<crate::Expression, ProcessError> {
        let field_len = expr.fields.len();
        let mut fields = Vec::with_capacity(field_len);
        let mut last_value = None;
        let mut consecutive: crate::IntType = 0;
        for (idx, field) in expr.fields.into_iter().enumerate() {
            match field {
                lua_parser::TableField::KeyValue(keyval) => {
                    let key = self.process_expression(keyval.key)?;
                    let value = self.process_expression(keyval.value)?;
                    fields.push((key, value));
                }
                lua_parser::TableField::NameValue(nameval) => {
                    let key: crate::Expression = nameval.name.string.into();
                    let value = self.process_expression(nameval.value)?;
                    fields.push((key, value));
                }
                lua_parser::TableField::Value(value) => {
                    consecutive += 1;
                    let value = self.process_expression(value.value)?;
                    if idx == field_len - 1 {
                        last_value = Some((consecutive, Box::new(value)));
                    } else {
                        let key: crate::Expression = consecutive.into();
                        fields.push((key, value));
                    }
                }
            }
        }
        Ok(crate::Expression::TableConstructor(
            crate::ExprTableConstructor::new(fields, last_value),
        ))
    }
    fn process_expression_ident(
        &mut self,
        expr: lua_parser::ExprIdent,
    ) -> Result<crate::Expression, ProcessError> {
        if let Some(local_var) = self.search_local_variable(&expr.name) {
            Ok(crate::Expression::LocalVariable(local_var))
        } else {
            // it is global variable
            let key: crate::Expression = expr.name.string.into();
            let table = crate::Expression::Env;

            Ok(crate::Expression::TableIndex(crate::ExprTableIndex::new(
                table, key,
            )))
        }
    }
    fn process_expression_nil(
        &mut self,
        _: lua_parser::ExprNil,
    ) -> Result<crate::Expression, ProcessError> {
        Ok(().into())
    }
    fn process_expression_numeric(
        &mut self,
        expr: lua_parser::ExprNumeric,
    ) -> Result<crate::Expression, ProcessError> {
        Ok(expr.value.into())
    }
    fn process_expression_string(
        &mut self,
        expr: lua_parser::ExprString,
    ) -> Result<crate::Expression, ProcessError> {
        Ok(expr.value.into())
    }
    fn process_expression_bool(
        &mut self,
        expr: lua_parser::ExprBool,
    ) -> Result<crate::Expression, ProcessError> {
        Ok(expr.value.into())
    }
    fn process_statement_function_call(
        &mut self,
        stmt: lua_parser::StmtFunctionCall,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        let expression = self.process_expression_function_call(stmt)?;
        if let crate::Expression::FunctionCall(expr) = expression {
            blk.statements.push(crate::Statement::FunctionCall(expr));
        } else {
            unreachable!("StmtFunctionCall must be match with ExprFunctionCall");
        }
        Ok(())
    }
    fn process_statement_function_def_local(
        &mut self,
        stmt: lua_parser::StmtFunctionDefinitionLocal,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        /*
        local function_name;
        function_name = function_obj;
        */

        let varinfo = self.begin_variable_scope(stmt.name.to_string());
        let func_expr = self.process_expression_function(stmt.body)?;

        let var_expr = crate::Expression::LocalVariable(crate::ExprLocalVariable::Stack(
            varinfo.borrow().offset,
            stmt.name.to_string(),
        ));
        let assign_stmt = crate::Statement::Assignment(crate::StmtAssignment::new(
            vec![var_expr],
            vec![func_expr],
        ));
        blk.statements.push(assign_stmt);
        Ok(())
    }
    fn process_statement_assignment(
        &mut self,
        stmt: lua_parser::StmtAssignment,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        // eval rhs first
        let mut rhs = Vec::with_capacity(stmt.rhs.len());
        for expr in stmt.rhs.into_iter() {
            rhs.push(self.process_expression(expr)?);
        }

        let mut lhs = Vec::with_capacity(stmt.lhs.len());
        for expr in stmt.lhs.into_iter() {
            lhs.push(self.process_expression(expr)?);
        }

        let assign_stmt = crate::Statement::Assignment(crate::StmtAssignment::new(lhs, rhs));
        blk.statements.push(assign_stmt);
        Ok(())
    }
    fn process_statement_do(
        &mut self,
        stmt: lua_parser::StmtDo,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        let block = self.process_block(stmt.block, true, false)?;
        let do_stmt = crate::Statement::Do(block);
        blk.statements.push(do_stmt);
        Ok(())
    }
    fn process_statement_while(
        &mut self,
        stmt: lua_parser::StmtWhile,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        let condition = self.process_expression(stmt.condition)?;
        let block = self.process_block(stmt.block, true, true)?;
        let while_stmt = crate::Statement::While(crate::StmtWhile::new(condition, block));
        blk.statements.push(while_stmt);
        Ok(())
    }
    fn process_statement_break(
        &mut self,
        stmt: lua_parser::StmtBreak,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        // check loop scope
        for scope in self.scopes.iter().rev() {
            match scope {
                Scope::Block(blk_) => {
                    if blk_.is_loop {
                        blk.statements.push(crate::Statement::Break);
                        return Ok(());
                    }
                }
                Scope::Function(_) => {
                    return Err(ProcessError::BreakOutsideLoop(stmt.span()));
                }
            }
        }
        Err(ProcessError::BreakOutsideLoop(stmt.span()))
    }
    fn process_statement_if(
        &mut self,
        stmt: lua_parser::StmtIf,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        let condition = self.process_expression(stmt.condition)?;
        let block = self.process_block(stmt.block, true, false)?;
        let mut else_ifs = Vec::with_capacity(stmt.else_ifs.len());
        for else_if in stmt.else_ifs.into_iter() {
            let condition = self.process_expression(else_if.condition)?;
            let block = self.process_block(else_if.block, true, false)?;
            else_ifs.push((condition, block));
        }
        let else_ = if let Some(else_block) = stmt.else_block {
            Some(self.process_block(else_block, true, false)?)
        } else {
            None
        };
        let if_stmt = crate::Statement::If(crate::StmtIf::new(condition, block, else_ifs, else_));
        blk.statements.push(if_stmt);
        Ok(())
    }
    fn process_statement_repeat(
        &mut self,
        stmt: lua_parser::StmtRepeat,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        self.begin_scope(true);

        let block = self.process_block(stmt.block, false, true)?;
        let condition = self.process_expression(stmt.condition)?;
        let repeat_stmt = crate::Statement::Repeat(crate::StmtRepeat::new(block, condition));
        blk.statements.push(repeat_stmt);

        self.end_scope();
        Ok(())
    }
    fn process_statement_for(
        &mut self,
        stmt: lua_parser::StmtFor,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        self.begin_scope(true);

        let start = self.process_expression(stmt.start)?;
        let end = self.process_expression(stmt.end)?;
        let step = self.process_expression(stmt.step)?;

        let name = stmt.name;
        let control_var = self.begin_variable_scope(name);

        let block = self.process_block(stmt.block, false, true)?;
        let for_stmt =
            crate::Statement::For(crate::StmtFor::new(control_var, start, end, step, block));
        blk.statements.push(for_stmt);

        self.end_scope();
        Ok(())
    }
    fn process_statement_for_generic(
        &mut self,
        stmt: lua_parser::StmtForGeneric,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        self.begin_scope(true);

        let iterator = self.begin_variable_scope("@for_iterator".to_string());
        let state = self.begin_variable_scope("@for_state".to_string());
        let closing = self.begin_variable_scope("@for_closing".to_string());

        let mut exprs = Vec::with_capacity(stmt.expressions.len());
        for expr in stmt.expressions.into_iter() {
            exprs.push(self.process_expression(expr)?);
        }

        let mut control_variables = Vec::with_capacity(stmt.names.len());
        for name in stmt.names {
            let name = name.string;
            let offset = self.begin_variable_scope(name);
            control_variables.push(offset);
        }

        let block = self.process_block(stmt.block, false, true)?;

        let for_stmt = crate::Statement::ForGeneric(crate::StmtForGeneric::new(
            control_variables,
            iterator,
            state,
            closing,
            exprs,
            block,
        ));
        blk.statements.push(for_stmt);

        self.end_scope();
        Ok(())
    }
    fn process_return_statement(
        &mut self,
        stmt: lua_parser::ReturnStatement,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        let mut exprs = Vec::with_capacity(stmt.values.len());
        for expr in stmt.values.into_iter() {
            exprs.push(self.process_expression(expr)?);
        }
        let ret_stmt = crate::ReturnStatement::new(exprs);
        blk.return_statement = Some(ret_stmt);
        Ok(())
    }
    fn process_statement_function_def(
        &mut self,
        stmt: lua_parser::StmtFunctionDefinition,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        /*
        func.name.dot.chain = function_obj;

        func.name.dot.chain:colon = function_obj ( self, args ... ) ;
        */
        let mut name = stmt.name;
        let mut body = stmt.body;
        if let Some(colon) = name.colon {
            // a.b:c ( arg0, arg1, ... ) => a.b.c ( self, arg0, arg1, ... )

            // add "self" to the beginning of the parameter list
            body.parameters.names.insert(
                0,
                lua_parser::SpannedString::new("self".to_string(), lua_parser::Span::new_none()),
            );

            // add colon-ed method name to the end of dot-chain
            name.names.push(colon);
        }

        let mut dotted_names = name.names;
        dotted_names.reverse();
        let name0 = dotted_names.pop().unwrap();
        let mut var_expr = self.process_expression_ident(lua_parser::ExprIdent::new(name0))?;

        while let Some(name) = dotted_names.pop() {
            let key: crate::Expression = name.string.into();
            let next_expr =
                crate::Expression::TableIndex(crate::ExprTableIndex::new(var_expr, key));
            var_expr = next_expr;
        }

        // var_expr = function_obj assignment

        let function_expr = self.process_expression_function(body)?;
        let assign_stmt = crate::Statement::Assignment(crate::StmtAssignment::new(
            vec![var_expr],
            vec![function_expr],
        ));
        blk.statements.push(assign_stmt);

        Ok(())
    }
    fn process_statement_local_decl(
        &mut self,
        stmt: lua_parser::StmtLocalDeclaration,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        let rhs = if let Some(values) = stmt.values {
            let mut rhs = Vec::with_capacity(values.len());
            for value in values.into_iter() {
                rhs.push(self.process_expression(value)?);
            }
            Some(rhs)
        } else {
            None
        };

        let mut vars = Vec::with_capacity(stmt.names.len());
        for name in stmt.names.into_iter() {
            let offset = self.begin_variable_scope(name.name.string);
            vars.push((offset, name.attrib));
        }

        let local_decl =
            crate::Statement::LocalDeclaration(crate::StmtLocalDeclaration::new(vars, rhs));
        blk.statements.push(local_decl);
        Ok(())
    }
    fn process_expression_variadic(
        &mut self,
        expr: lua_parser::ExprVariadic,
    ) -> Result<crate::Expression, ProcessError> {
        if let Some(scope) = self.nearest_function_scope() {
            if scope.variadic {
                Ok(crate::Expression::Variadic)
            } else {
                Err(ProcessError::VariadicInNonVariadicFunction(expr.span()))
            }
        } else {
            Err(ProcessError::VariadicOutsideFunction(expr.span()))
        }
    }
    fn process_statement_goto(
        &mut self,
        stmt: lua_parser::StmtGoto,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        let span = stmt.span();
        let name = stmt.name.string;
        let scope_tree = self.scope_tree();
        let label = self
            .labels
            .entry(name)
            .or_insert_with(|| Rc::new(RefCell::new(LabelInfo::default())));
        label.borrow_mut().add_from(scope_tree, span)?;
        let goto_stmt = crate::Statement::Goto(crate::StmtGoto::new(Rc::clone(label)));
        blk.statements.push(goto_stmt);
        Ok(())
    }
    fn process_statement_label(
        &mut self,
        stmt: lua_parser::StmtLabel,
        blk: &mut crate::Block,
    ) -> Result<(), ProcessError> {
        let scope_tree = self.scope_tree();
        let span = stmt.span();
        let spanned_name = stmt.name;
        let name = spanned_name.to_string();
        let label = self
            .labels
            .entry(spanned_name.to_string())
            .or_insert_with(|| Rc::new(RefCell::new(LabelInfo::default())));
        label
            .borrow_mut()
            .set_label(spanned_name, scope_tree, span)?;
        match self.scopes.last_mut().unwrap() {
            Scope::Block(blk) => {
                blk.labels.push(name);
            }
            _ => unreachable!("process_statement_label - block scope not opened?"),
        }
        let label_stmt = crate::Statement::Label(crate::StmtLabel::new(Rc::clone(label)));
        blk.statements.push(label_stmt);

        Ok(())
    }
    fn process_expression_function(
        &mut self,
        mut expr: lua_parser::ExprFunction,
    ) -> Result<crate::Expression, ProcessError> {
        // begin function scope
        self.begin_function_scope(expr.parameters.variadic);
        self.begin_scope(false);

        // define parameters
        let mut param_offsets = Vec::with_capacity(expr.parameters.names.len());
        for var_name in expr.parameters.names.into_iter() {
            let offset = self.begin_variable_scope(var_name.string);
            param_offsets.push(offset);
        }

        // force add dummy return statement
        if expr.block.return_statement.is_none() {
            expr.block.return_statement = Some(lua_parser::ReturnStatement::new(
                Vec::new(),
                lua_parser::Span::new_none(),
            ));
        }
        let mut block = self.process_block(expr.block, false, false)?;

        self.end_scope();
        let function_scope = self.end_function_scope();
        block.stack_size = Some(function_scope.max_variables);
        // end function scope

        // add function definition
        let definition =
            crate::FunctionDefinition::new(param_offsets, expr.parameters.variadic, block);

        let upvalues_source = function_scope
            .upvalues
            .into_iter()
            .map(|upvalue| upvalue.from)
            .collect();

        // emit function object
        Ok(crate::Expression::FunctionObject(
            crate::ExprFunctionObject::new(upvalues_source, definition),
        ))
    }
}
