use std::collections::HashMap;

use lua_semantics::Block;
use lua_semantics::ExprLocalVariable;
use lua_semantics::Expression;
use lua_semantics::Statement;

use crate::vm::Chunk;
use crate::Instruction;
use crate::LabelType;
use crate::LuaFunctionLua;

#[derive(Debug)]
pub struct Context {
    pub instructions: Vec<Instruction>,

    /// this stack holds the break label of the current loop
    pub loop_stack: Vec<LabelType>,

    /// label_type -> instruction index map
    pub label_map: Vec<Option<usize>>,
    /// user defined label -> label_type map
    pub user_defined_label: HashMap<String, LabelType>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            instructions: Vec::new(),
            label_map: Default::default(),
            loop_stack: Vec::new(),
            user_defined_label: HashMap::new(),
        }
    }

    /// generate unique label id
    fn generate_label(&mut self) -> LabelType {
        let label = self.label_map.len();
        self.label_map.push(None);
        label
    }
    /// set label to the current instruction index
    fn set_label(&mut self, label: LabelType) {
        // current instruction index
        let index = self.instructions.len();
        self.label_map[label] = Some(index);
    }

    /// return address of newly added instruction to be executed
    pub fn emit(mut self, mut block: Block) -> Chunk {
        if block.return_statement.is_none() {
            block.return_statement = Some(lua_semantics::ReturnStatement::new(Vec::new()));
        }
        let stack_size = block.stack_size.unwrap();
        self.emit_block(block);

        Chunk {
            instructions: self.instructions,
            label_map: self.label_map.into_iter().map(|x| x.unwrap()).collect(),
            stack_size,
        }
    }

    fn emit_block(&mut self, block: Block) {
        for stmt in block.statements {
            self.emit_statement(stmt);
        }
        if let Some(ret) = block.return_statement {
            // self.instructions.push(Instruction::Sp);
            let rhs_len = ret.values.len();
            for (idx, value) in ret.values.into_iter().enumerate() {
                if idx == rhs_len - 1 {
                    self.emit_expression(value, None);
                } else {
                    self.emit_expression(value, Some(1));
                }
            }
            self.instructions.push(Instruction::Return);
        }
    }

    fn emit_statement(&mut self, statement: Statement) {
        match statement {
            lua_semantics::Statement::Assignment(stmt) => self.emit_statement_assignment(stmt),
            lua_semantics::Statement::Break => self.emit_statement_break(),
            lua_semantics::Statement::Do(blk) => self.emit_block(blk),
            lua_semantics::Statement::While(stmt) => self.emit_statement_while(stmt),
            lua_semantics::Statement::Repeat(stmt) => self.emit_statement_repeat(stmt),
            lua_semantics::Statement::If(stmt) => self.emit_statement_if(stmt),
            lua_semantics::Statement::For(stmt) => self.emit_statement_for(stmt),
            lua_semantics::Statement::ForGeneric(stmt) => self.emit_statement_forgeneric(stmt),
            lua_semantics::Statement::FunctionCall(stmt) => self.emit_statement_functioncall(stmt),
            lua_semantics::Statement::LocalDeclaration(stmt) => {
                self.emit_statement_localdeclaration(stmt)
            }
            lua_semantics::Statement::Goto(stmt) => self.emit_statement_goto(stmt),
            lua_semantics::Statement::Label(stmt) => self.emit_statement_label(stmt),

            _ => {
                unimplemented!("unimplemented statement: {:?}", statement);
            }
        }
    }
    /// emit an instruction that evalulates an expression and store the result in a register AX
    fn emit_expression(&mut self, expression: Expression, expected: Option<usize>) {
        match expression {
            lua_semantics::Expression::Env => self.emit_expression_env(expected),
            lua_semantics::Expression::Variadic => self.emit_expression_variadic(expected),
            lua_semantics::Expression::Nil => self.emit_expression_nil(expected),
            lua_semantics::Expression::Boolean(value) => {
                self.emit_expression_boolean(value, expected)
            }
            lua_semantics::Expression::Numeric(value) => {
                self.emit_expression_numeric(value, expected)
            }
            lua_semantics::Expression::String(value) => {
                self.emit_expression_string(value, expected)
            }
            lua_semantics::Expression::LocalVariable(expr) => {
                self.emit_expression_localvariable(expr, expected)
            }
            lua_semantics::Expression::TableIndex(expr) => {
                self.emit_expression_tableindex(expr, expected)
            }
            lua_semantics::Expression::Binary(expr) => self.emit_expression_binary(expr, expected),
            lua_semantics::Expression::Unary(expr) => self.emit_expression_unary(expr, expected),
            lua_semantics::Expression::TableConstructor(expr) => {
                self.emit_expression_tableconstructor(expr, expected)
            }
            lua_semantics::Expression::FunctionCall(expr) => {
                self.emit_expression_functioncall(expr, expected)
            }
            lua_semantics::Expression::FunctionObject(expr) => {
                self.emit_expression_function_object(expr, expected)
            }
            _ => {
                unimplemented!("unimplemented expression: {:?}", expression);
            }
        }
    }
    fn emit_expression_set(&mut self, entry: lua_semantics::Expression) {
        match entry {
            lua_semantics::Expression::LocalVariable(expr) => match expr {
                ExprLocalVariable::Stack(local_id, _name) => {
                    self.instructions
                        .push(Instruction::SetLocalVariable(local_id));
                }
                ExprLocalVariable::Upvalue(index, _name) => {
                    self.instructions
                        .push(Instruction::FunctionUpvalueSet(index));
                }
            },
            lua_semantics::Expression::TableIndex(expr) => {
                self.emit_expression(*expr.table, Some(1));
                self.emit_expression(*expr.index, Some(1));
                self.instructions.push(Instruction::TableIndexSet);
            }
            _ => {
                unimplemented!("unimplemented expression: {:?}", entry);
            }
        }
    }
}

impl Context {
    fn emit_expression_env(&mut self, expected: Option<usize>) {
        if expected == Some(0) {
            return;
        }
        self.instructions.push(Instruction::GetEnv);
        if let Some(expected) = expected {
            self.emit_expression_nil(Some(expected - 1));
        }
    }
    fn emit_expression_nil(&mut self, expected: Option<usize>) {
        if let Some(expected) = expected {
            for _ in 0..expected {
                self.instructions.push(Instruction::Nil);
            }
        } else {
            self.instructions.push(Instruction::Nil);
        }
    }
    fn emit_expression_boolean(&mut self, value: bool, expected: Option<usize>) {
        if expected == Some(0) {
            return;
        }
        self.instructions.push(Instruction::Boolean(value));
        if let Some(expected) = expected {
            self.emit_expression_nil(Some(expected - 1));
        }
    }
    fn emit_expression_numeric(
        &mut self,
        value: lua_semantics::IntOrFloat,
        expected: Option<usize>,
    ) {
        if expected == Some(0) {
            return;
        }
        match value {
            lua_semantics::IntOrFloat::Int(n) => {
                self.instructions.push(Instruction::Numeric(n.into()));
            }
            lua_semantics::IntOrFloat::Float(n) => {
                self.instructions.push(Instruction::Numeric(n.into()));
            }
        }
        if let Some(expected) = expected {
            self.emit_expression_nil(Some(expected - 1));
        }
    }
    fn emit_expression_string(&mut self, value: Vec<u8>, expected: Option<usize>) {
        if expected == Some(0) {
            return;
        }
        self.instructions.push(Instruction::String(value));
        if let Some(expected) = expected {
            self.emit_expression_nil(Some(expected - 1));
        }
    }
    fn emit_expression_tableindex(
        &mut self,
        expr: lua_semantics::ExprTableIndex,
        expected: Option<usize>,
    ) {
        self.emit_expression(*expr.table, Some(1));
        self.emit_expression(*expr.index, Some(1));
        self.instructions.push(Instruction::TableIndex);
        if let Some(expected) = expected {
            if expected == 0 {
                self.instructions.push(Instruction::Pop);
            } else {
                self.emit_expression_nil(Some(expected - 1));
            }
        }
    }
    fn emit_expression_unary(&mut self, expr: lua_semantics::ExprUnary, expected: Option<usize>) {
        match expr {
            lua_semantics::ExprUnary::Minus(expr) => {
                self.emit_expression(*expr.value, Some(1));
                self.instructions.push(Instruction::UnaryMinus);
            }
            lua_semantics::ExprUnary::BitwiseNot(expr) => {
                self.emit_expression(*expr.value, Some(1));
                self.instructions.push(Instruction::UnaryBitwiseNot);
            }
            lua_semantics::ExprUnary::Length(expr) => {
                self.emit_expression(*expr.value, Some(1));
                self.instructions.push(Instruction::UnaryLength);
            }
            lua_semantics::ExprUnary::LogicalNot(expr) => {
                self.emit_expression(*expr.value, Some(1));
                self.instructions.push(Instruction::UnaryLogicalNot);
            }
        }
        if let Some(expected) = expected {
            if expected == 0 {
                self.instructions.push(Instruction::Pop);
            } else {
                self.emit_expression_nil(Some(expected - 1));
            }
        }
    }
    fn emit_expression_binary(&mut self, expr: lua_semantics::ExprBinary, expected: Option<usize>) {
        match expr {
            lua_semantics::ExprBinary::Add(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryAdd);
            }
            lua_semantics::ExprBinary::Sub(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinarySub);
            }
            lua_semantics::ExprBinary::Mul(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryMul);
            }
            lua_semantics::ExprBinary::Div(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryDiv);
            }
            lua_semantics::ExprBinary::FloorDiv(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryFloorDiv);
            }
            lua_semantics::ExprBinary::Mod(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryMod);
            }
            lua_semantics::ExprBinary::Pow(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryPow);
            }
            lua_semantics::ExprBinary::Concat(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryConcat);
            }
            lua_semantics::ExprBinary::BitwiseAnd(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryBitwiseAnd);
            }
            lua_semantics::ExprBinary::BitwiseOr(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryBitwiseOr);
            }
            lua_semantics::ExprBinary::BitwiseXor(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryBitwiseXor);
            }
            lua_semantics::ExprBinary::ShiftLeft(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryShiftLeft);
            }
            lua_semantics::ExprBinary::ShiftRight(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryShiftRight);
            }
            lua_semantics::ExprBinary::Equal(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryEqual);
            }
            lua_semantics::ExprBinary::NotEqual(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryEqual);
                self.instructions.push(Instruction::UnaryLogicalNot);
            }
            lua_semantics::ExprBinary::LessThan(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryLessThan);
            }
            lua_semantics::ExprBinary::LessEqual(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryLessEqual);
            }
            lua_semantics::ExprBinary::GreaterThan(expr) => {
                self.emit_expression(*expr.rhs, Some(1));
                self.emit_expression(*expr.lhs, Some(1));
                self.instructions.push(Instruction::BinaryLessThan);
            }
            lua_semantics::ExprBinary::GreaterEqual(expr) => {
                self.emit_expression(*expr.rhs, Some(1));
                self.emit_expression(*expr.lhs, Some(1));
                self.instructions.push(Instruction::BinaryLessEqual);
            }
            lua_semantics::ExprBinary::LogicalAnd(expr) => {
                /*
                The conjunction operator and returns its first argument if this value is false or nil;
                otherwise, and returns its second argument.

                AX <- eval(lhs)
                if AX is false, jump to lhs_false_label
                AX <- eval(rhs)
                lhs_false_label:
                */

                let lhs_false_label = self.generate_label();
                self.emit_expression(*expr.lhs, Some(1));
                self.instructions.push(Instruction::Clone);
                self.instructions
                    .push(Instruction::JumpFalse(lhs_false_label));
                self.instructions.push(Instruction::Pop);
                self.emit_expression(*expr.rhs, Some(1));
                self.set_label(lhs_false_label);
            }
            lua_semantics::ExprBinary::LogicalOr(expr) => {
                /*
                The disjunction operator or returns its first argument if this value is different from nil and false;
                otherwise, or returns its second argument.

                AX <- eval(lhs)
                if AX is true, jump to lhs_true_label
                AX <- eval(rhs)
                lhs_true_label:
                */
                let lhs_true_label = self.generate_label();
                self.emit_expression(*expr.lhs, Some(1));
                self.instructions.push(Instruction::Clone);
                self.instructions
                    .push(Instruction::JumpTrue(lhs_true_label));
                self.instructions.push(Instruction::Pop);
                self.emit_expression(*expr.rhs, Some(1));
                self.set_label(lhs_true_label);
            }
        }
        if let Some(expected) = expected {
            if expected == 0 {
                self.instructions.push(Instruction::Pop);
            } else {
                self.emit_expression_nil(Some(expected - 1));
            }
        }
    }
    fn emit_expression_localvariable(
        &mut self,
        expr: lua_semantics::ExprLocalVariable,
        expected: Option<usize>,
    ) {
        if expected == Some(0) {
            return;
        }
        match expr {
            lua_semantics::ExprLocalVariable::Stack(local_id, name) => {
                self.instructions
                    .push(Instruction::GetLocalVariable(local_id, name));
            }
            lua_semantics::ExprLocalVariable::Upvalue(index, name) => {
                self.instructions
                    .push(Instruction::FunctionUpvalue(index, name));
            }
        }
        if let Some(expected) = expected {
            self.emit_expression_nil(Some(expected - 1));
        }
    }

    fn emit_expression_variadic(&mut self, expected: Option<usize>) {
        self.instructions.push(Instruction::GetVariadic(expected));
    }
    fn emit_expression_tableconstructor(
        &mut self,
        expr: lua_semantics::ExprTableConstructor,
        expected: Option<usize>,
    ) {
        self.instructions
            .push(Instruction::TableInit(expr.fields.len()));

        for (key, value) in expr.fields {
            self.emit_expression(key, Some(1));
            self.emit_expression(value, Some(1));
            self.instructions.push(Instruction::TableIndexInit);
        }

        if let Some((last_idx, last_expr)) = expr.last_value_field {
            self.instructions.push(Instruction::Sp);
            self.emit_expression(*last_expr, None);
            self.instructions.push(Instruction::TableInitLast(last_idx));
        }

        if let Some(expected) = expected {
            if expected == 0 {
                self.instructions.push(Instruction::Pop);
            } else {
                self.emit_expression_nil(Some(expected - 1));
            }
        }
    }
}

impl Context {
    fn emit_statement_for(&mut self, stmt: lua_semantics::StmtFor) {
        let break_label = self.generate_label();
        let continue_label = self.generate_label();
        self.loop_stack.push(break_label);
        let control_offset = stmt.control_variable.borrow().offset;

        self.emit_expression(stmt.start, Some(1));

        self.set_label(continue_label);
        self.instructions
            .push(Instruction::InitLocalVariable(control_offset));
        // check range
        self.instructions.push(Instruction::GetLocalVariable(
            control_offset,
            "@control".to_string(),
        ));
        self.emit_expression(stmt.end, Some(1));
        self.instructions.push(Instruction::BinaryLessEqual); // @TODO less, overflow check
        self.instructions.push(Instruction::JumpFalse(break_label));

        self.emit_block(stmt.block);
        self.instructions.push(Instruction::GetLocalVariable(
            control_offset,
            "@control".to_string(),
        ));
        self.emit_expression(stmt.step, Some(1));
        self.instructions.push(Instruction::BinaryAdd);
        self.instructions.push(Instruction::Jump(continue_label));

        self.set_label(break_label);
        self.loop_stack.pop();
    }
    fn emit_statement_forgeneric(&mut self, stmt: lua_semantics::StmtForGeneric) {
        let break_label = self.generate_label();
        let continue_label = self.generate_label();
        self.loop_stack.push(break_label);

        // emit exactly 4 expressions for (iterator, state, initial_value, closing_value)
        {
            let mut emit_count = 0;
            let exp_len = stmt.expressions.len();
            for (idx, exp) in stmt.expressions.into_iter().enumerate() {
                if idx == exp_len - 1 {
                    if emit_count <= 4 {
                        self.emit_expression(exp, Some(4 - emit_count));
                    } else {
                        self.emit_expression(exp, Some(0));
                    }
                } else {
                    if emit_count < 4 {
                        self.emit_expression(exp, Some(1));
                        emit_count += 1;
                    } else {
                        self.emit_expression(exp, Some(0));
                    }
                }
            }
        }

        self.instructions
            .push(Instruction::InitLocalVariable(stmt.closing.borrow().offset));
        self.instructions.push(Instruction::InitLocalVariable(
            stmt.control_variables[0].borrow().offset,
        ));
        self.instructions
            .push(Instruction::InitLocalVariable(stmt.state.borrow().offset));
        self.instructions.push(Instruction::InitLocalVariable(
            stmt.iterator.borrow().offset,
        ));

        // ::continue_label::
        self.set_label(continue_label);

        // iterator function call
        // control_variables* ... = iterator( state, control_variable_0 )
        self.instructions.push(Instruction::Sp);
        self.instructions.push(Instruction::GetLocalVariable(
            stmt.state.borrow().offset,
            "@control".to_string(),
        ));
        self.instructions.push(Instruction::GetLocalVariable(
            stmt.control_variables[0].borrow().offset,
            "@control".to_string(),
        ));
        self.instructions.push(Instruction::GetLocalVariable(
            stmt.iterator.borrow().offset,
            "@control".to_string(),
        ));
        self.instructions.push(Instruction::FunctionCall(Some(
            stmt.control_variables.len(),
        )));
        for control_var in stmt.control_variables.iter().rev() {
            self.instructions
                .push(Instruction::InitLocalVariable(control_var.borrow().offset));
        }
        // get control_variable_0
        self.instructions.push(Instruction::GetLocalVariable(
            stmt.control_variables[0].borrow().offset,
            "@control".to_string(),
        ));
        // check if it is nil
        self.instructions.push(Instruction::IsNil);
        // jump to break_label if it is nil
        self.instructions.push(Instruction::JumpTrue(break_label));

        self.emit_block(stmt.block);
        self.instructions.push(Instruction::Jump(continue_label));

        self.set_label(break_label);
        self.loop_stack.pop();
    }

    fn emit_expression_function_object(
        &mut self,
        expr: lua_semantics::ExprFunctionObject,
        expected: Option<usize>,
    ) {
        let function_context = Self::new();
        let lua_function = LuaFunctionLua {
            upvalues: Vec::with_capacity(expr.upvalues_source.len()),
            args: expr.definition.args.len(),
            is_variadic: expr.definition.variadic,
            chunk: function_context.emit(expr.definition.body),
        };

        self.instructions
            .push(Instruction::FunctionInit(Box::new(lua_function)));

        // initialize upvalues
        for upvalue in expr.upvalues_source {
            match upvalue {
                ExprLocalVariable::Stack(local_id, _name) => {
                    self.instructions
                        .push(Instruction::FunctionInitUpvalueFromLocalVar(local_id));
                }
                ExprLocalVariable::Upvalue(index, _name) => {
                    self.instructions
                        .push(Instruction::FunctionInitUpvalueFromUpvalue(index));
                }
            }
        }
        if let Some(expected) = expected {
            if expected == 0 {
                self.instructions.push(Instruction::Pop);
            } else {
                self.emit_expression_nil(Some(expected - 1));
            }
        }
    }

    fn emit_expression_functioncall(
        &mut self,
        expr: lua_semantics::ExprFunctionCall,
        expected: Option<usize>,
    ) {
        self.instructions.push(Instruction::Sp);
        // prefix:method( args ) -> prefix.method( prefix, args )
        if let Some(method) = expr.method {
            self.emit_expression(*expr.prefix, Some(1));
            let len = expr.args.len();
            for (idx, arg) in expr.args.into_iter().enumerate() {
                if idx == len - 1 {
                    self.emit_expression(arg, None);
                } else {
                    self.emit_expression(arg, Some(1));
                }
            }

            self.instructions.push(Instruction::Deref);
            self.instructions
                .push(Instruction::String(method.into_bytes()));
            self.instructions.push(Instruction::TableIndex);
        } else {
            let len = expr.args.len();
            for (idx, arg) in expr.args.into_iter().enumerate() {
                if idx == len - 1 {
                    self.emit_expression(arg, None);
                } else {
                    self.emit_expression(arg, Some(1));
                }
            }
            self.emit_expression(*expr.prefix, Some(1));
        }
        self.instructions.push(Instruction::FunctionCall(expected));
    }

    fn emit_statement_if(&mut self, stmt: lua_semantics::StmtIf) {
        let end_label = self.generate_label();

        {
            let false_label = self.generate_label();
            self.emit_expression(stmt.condition, Some(1));
            self.instructions.push(Instruction::JumpFalse(false_label));
            self.emit_block(stmt.block);
            self.instructions.push(Instruction::Jump(end_label));
            self.set_label(false_label);
        }
        for (cond, blk) in stmt.else_ifs {
            let false_label = self.generate_label();
            self.emit_expression(cond, Some(1));
            self.instructions.push(Instruction::JumpFalse(false_label));
            self.emit_block(blk);
            self.instructions.push(Instruction::Jump(end_label));
            self.set_label(false_label);
        }
        if let Some(else_block) = stmt.else_ {
            self.emit_block(else_block);
        }
        self.set_label(end_label);
    }
    fn emit_statement_functioncall(&mut self, stmt: lua_semantics::StmtFunctionCall) {
        self.emit_expression_functioncall(stmt, Some(0))
    }
    fn emit_statement_localdeclaration(&mut self, stmt: lua_semantics::StmtLocalDeclaration) {
        if let Some(expr) = stmt.values {
            let lhs_len = stmt.decls.len();
            let rhs_len = expr.len();
            for (idx, rhs) in expr.into_iter().enumerate() {
                if idx == rhs_len - 1 {
                    if idx < lhs_len {
                        self.emit_expression(rhs, Some(lhs_len - idx));
                    } else {
                        self.emit_expression(rhs, Some(0));
                    }
                } else {
                    if idx < lhs_len {
                        self.emit_expression(rhs, Some(1));
                    } else {
                        self.emit_expression(rhs, Some(0));
                    }
                }
            }
        } else {
            self.emit_expression_nil(Some(stmt.decls.len()));
        }
        for (lhs_info, _attrib) in stmt.decls.into_iter().rev() {
            let local_id = lhs_info.borrow().offset;
            self.instructions
                .push(Instruction::InitLocalVariable(local_id));
        }
    }
    fn emit_statement_while(&mut self, stmt: lua_semantics::StmtWhile) {
        let continue_label = self.generate_label();
        let break_label = self.generate_label();
        self.loop_stack.push(break_label);

        self.set_label(continue_label);
        self.emit_expression(stmt.condition, Some(1));
        self.instructions.push(Instruction::JumpFalse(break_label));
        self.emit_block(stmt.block);
        self.instructions.push(Instruction::Jump(continue_label));
        self.set_label(break_label);

        self.loop_stack.pop();
    }
    fn emit_statement_repeat(&mut self, stmt: lua_semantics::StmtRepeat) {
        let continue_label = self.generate_label();
        let break_label = self.generate_label();
        self.loop_stack.push(break_label);

        self.set_label(continue_label);
        self.emit_block(stmt.block);
        self.emit_expression(stmt.condition, Some(1));
        self.instructions
            .push(Instruction::JumpTrue(continue_label));
        self.set_label(break_label);

        self.loop_stack.pop();
    }
    fn emit_statement_assignment(&mut self, stmt: lua_semantics::StmtAssignment) {
        let lhs_len = stmt.lhs.len();
        let rhs_len = stmt.rhs.len();

        for (idx, rhs) in stmt.rhs.into_iter().enumerate() {
            if idx == rhs_len - 1 {
                if idx < lhs_len {
                    self.emit_expression(rhs, Some(lhs_len - idx));
                } else {
                    self.emit_expression(rhs, Some(0));
                }
            } else {
                if idx < lhs_len {
                    self.emit_expression(rhs, Some(1));
                } else {
                    self.emit_expression(rhs, Some(0));
                }
            }
        }

        for lhs in stmt.lhs.into_iter().rev() {
            self.emit_expression_set(lhs);
        }
    }
    fn emit_statement_break(&mut self) {
        let break_label = match self.loop_stack.last() {
            Some(label) => *label,
            None => {
                unreachable!("break outside loop");
            }
        };
        self.instructions.push(Instruction::Jump(break_label));
    }
    fn emit_statement_goto(&mut self, stmt: lua_semantics::StmtGoto) {
        let name = &stmt.label.borrow().name;
        let label = if let Some(label) = self.user_defined_label.get(name) {
            *label
        } else {
            let label = self.generate_label();
            self.user_defined_label.insert(name.clone(), label);
            label
        };
        self.instructions.push(Instruction::Jump(label));
    }
    fn emit_statement_label(&mut self, stmt: lua_semantics::StmtLabel) {
        let name = &stmt.label.borrow().name;
        let label = if let Some(label) = self.user_defined_label.get(name) {
            *label
        } else {
            let label = self.generate_label();
            self.user_defined_label.insert(name.clone(), label);
            label
        };
        self.set_label(label);
    }
}
