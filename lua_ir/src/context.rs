use std::collections::HashMap;

use lua_semantics::Block;
use lua_semantics::ExprLocalVariable;
use lua_semantics::Expression;
use lua_semantics::FunctionDefinition;
use lua_semantics::Scope;
use lua_semantics::Statement;

use crate::vm::Program;
use crate::Instruction;

#[derive(Debug)]
pub struct Context {
    pub instructions: Vec<Instruction>,

    pub label_map: HashMap<String, usize>,
    pub function_label: Vec<String>,
    pub functions: Vec<FunctionDefinition>,
    pub label_counter: usize,

    pub loop_stack: Vec<String>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            instructions: Vec::new(),
            label_counter: 0,
            label_map: HashMap::new(),
            loop_stack: Vec::new(),
            function_label: Vec::new(),
            functions: Vec::new(),
        }
    }

    /// generate unique label id
    fn generate_label(&mut self) -> String {
        let label = format!(".__L{}", self.label_counter);
        self.label_counter += 1;
        label
    }
    /// set label to the current instruction index
    fn set_label(&mut self, label: String) {
        let index = self.instructions.len();
        if let Some(_) = self.label_map.insert(label.clone(), index) {}
    }

    pub fn emit(mut self, mut block: Block, ctx: lua_semantics::Context) -> Program {
        if block.return_statement.is_none() {
            block.return_statement = Some(lua_semantics::ReturnStatement::new(Vec::new()));
        }
        self.emit_block(block);
        for func in ctx.functions {
            self.functions.push(func.clone());
            let label = self.generate_label();
            self.set_label(label.clone());
            self.function_label.push(label);
            self.emit_function_definition(func);
        }

        let stack_size = match &ctx.scopes[0] {
            Scope::Block(block) => block.max_variables,
            _ => unreachable!("main scope must be block"),
        };
        let program = Program {
            function_map: self.function_label,
            instructions: self.instructions,
            functions: self.functions,
            label_map: self.label_map,
            stack_size,
        };
        program
    }

    fn emit_function_definition(&mut self, mut func: FunctionDefinition) {
        if func.body.return_statement.is_none() {
            func.body.return_statement = Some(lua_semantics::ReturnStatement::new(Vec::new()));
        }
        self.emit_block(func.body);
    }

    fn emit_block(&mut self, block: Block) {
        for stmt in block.statements {
            self.emit_statement(stmt);
        }
        if let Some(ret) = block.return_statement {
            self.instructions.push(Instruction::Sp);
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
        debug_assert!(expected.is_none() || expected.unwrap() > 0);
        match expression {
            lua_semantics::Expression::G => self.emit_expression_g(expected),
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
                ExprLocalVariable::Stack(offset) => {
                    self.instructions.push(Instruction::SetStack(offset));
                }
                ExprLocalVariable::Upvalue(index) => {
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
    fn emit_expression_g(&mut self, expected: Option<usize>) {
        self.instructions.push(Instruction::GetGlobal);
        if let Some(expected) = expected {
            self.emit_expression_nil(Some(expected - 1));
        }
    }
    fn emit_expression_env(&mut self, expected: Option<usize>) {
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
        self.instructions.push(Instruction::Numeric(value));
        if let Some(expected) = expected {
            self.emit_expression_nil(Some(expected - 1));
        }
    }
    fn emit_expression_string(&mut self, value: String, expected: Option<usize>) {
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
            self.emit_expression_nil(Some(expected - 1));
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
                self.instructions.push(Instruction::UnaryBitwiseNot);
            }
        }
        if let Some(expected) = expected {
            self.emit_expression_nil(Some(expected - 1));
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
                self.instructions.push(Instruction::BinaryNotEqual);
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
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryGreaterThan);
            }
            lua_semantics::ExprBinary::GreaterEqual(expr) => {
                self.emit_expression(*expr.lhs, Some(1));
                self.emit_expression(*expr.rhs, Some(1));
                self.instructions.push(Instruction::BinaryGreaterEqual);
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
                self.instructions
                    .push(Instruction::JumpFalse(lhs_false_label.clone()));
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
                self.instructions
                    .push(Instruction::JumpTrue(lhs_true_label.clone()));
                self.emit_expression(*expr.rhs, Some(1));
                self.set_label(lhs_true_label);
            }
        }
        if let Some(expected) = expected {
            self.emit_expression_nil(Some(expected - 1));
        }
    }
    fn emit_expression_localvariable(
        &mut self,
        expr: lua_semantics::ExprLocalVariable,
        expected: Option<usize>,
    ) {
        match expr {
            lua_semantics::ExprLocalVariable::Stack(offset) => {
                self.instructions.push(Instruction::GetStack(offset));
            }
            lua_semantics::ExprLocalVariable::Upvalue(index) => {
                self.instructions.push(Instruction::FunctionUpvalue(index));
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
            self.emit_expression_nil(Some(expected - 1));
        }
    }
}

impl Context {
    fn emit_statement_for(&mut self, stmt: lua_semantics::StmtFor) {
        let break_label = self.generate_label();
        self.loop_stack.push(break_label.clone());

        // @TODO

        self.emit_block(stmt.block);
        self.set_label(break_label);
        self.loop_stack.pop();
    }
    fn emit_statement_forgeneric(&mut self, stmt: lua_semantics::StmtForGeneric) {
        unimplemented!("for-generic");
        let break_label = self.generate_label();
        self.loop_stack.push(break_label.clone());

        // @TODO

        self.set_label(break_label);
        self.loop_stack.pop();
    }

    fn emit_expression_function_object(
        &mut self,
        expr: lua_semantics::ExprFunctionObject,
        expected: Option<usize>,
    ) {
        // add dummy return statement if there is no return statement

        let upvalues_len = expr.upvalues_source.len();
        self.instructions
            .push(Instruction::FunctionInit(expr.function_id, upvalues_len));
        for upvalue in expr.upvalues_source {
            match upvalue {
                ExprLocalVariable::Stack(offset) => {
                    self.instructions
                        .push(Instruction::FunctionUpvaluePushWithStack(offset));
                }
                ExprLocalVariable::Upvalue(index) => {
                    self.instructions
                        .push(Instruction::FunctionUpvaluePushWithUpvalue(index));
                }
            }
        }
        if let Some(expected) = expected {
            self.emit_expression_nil(Some(expected - 1));
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
            self.instructions.push(Instruction::Clone);
            self.instructions.push(Instruction::String(method));
            self.instructions.push(Instruction::TableIndex);
            self.instructions.push(Instruction::Swap);
        } else {
            self.emit_expression(*expr.prefix, Some(1));
        }
        let len = expr.args.len();
        for (idx, arg) in expr.args.into_iter().enumerate() {
            if idx == len - 1 {
                self.emit_expression(arg, None);
            } else {
                self.emit_expression(arg, Some(1));
            }
        }
        self.instructions.push(Instruction::FunctionCall(expected));
    }

    fn emit_statement_if(&mut self, stmt: lua_semantics::StmtIf) {
        let end_label = self.generate_label();

        {
            let false_label = self.generate_label();
            self.emit_expression(stmt.condition, Some(1));
            self.instructions
                .push(Instruction::JumpFalse(false_label.clone()));
            self.emit_block(stmt.block);
            self.instructions.push(Instruction::Jump(end_label.clone()));
            self.set_label(false_label);
        }
        for (cond, blk) in stmt.else_ifs {
            let false_label = self.generate_label();
            self.emit_expression(cond, Some(1));
            self.instructions
                .push(Instruction::JumpFalse(false_label.clone()));
            self.emit_block(blk);
            self.instructions.push(Instruction::Jump(end_label.clone()));
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
            self.instructions.push(Instruction::Sp);
            let rhs_len = expr.len();
            for (idx, rhs) in expr.into_iter().enumerate() {
                if idx == rhs_len - 1 {
                    self.emit_expression(rhs, None);
                } else {
                    self.emit_expression(rhs, Some(1));
                }
            }
            self.instructions
                .push(Instruction::AdjustMultire(stmt.decls.len()));
        } else {
            self.emit_expression_nil(Some(stmt.decls.len()));
        }
        for (lhs_info, attrib) in stmt.decls.into_iter().rev() {
            let offset = lhs_info.borrow().offset;
            self.instructions.push(Instruction::SetStack(offset));
            if lhs_info.borrow().is_reference {
                self.instructions.push(Instruction::InitRef(offset));
            }
        }
    }
    fn emit_statement_while(&mut self, stmt: lua_semantics::StmtWhile) {
        let continue_label = self.generate_label();
        let break_label = self.generate_label();
        self.loop_stack.push(break_label.clone());

        self.set_label(continue_label.clone());
        self.emit_expression(stmt.condition, Some(1));
        self.instructions
            .push(Instruction::JumpFalse(break_label.clone()));
        self.emit_block(stmt.block);
        self.instructions
            .push(Instruction::Jump(continue_label.clone()));
        self.set_label(break_label);

        self.loop_stack.pop();
    }
    fn emit_statement_repeat(&mut self, stmt: lua_semantics::StmtRepeat) {
        let continue_label = self.generate_label();
        let break_label = self.generate_label();
        self.loop_stack.push(break_label.clone());

        self.set_label(continue_label.clone());
        self.emit_block(stmt.block);
        self.emit_expression(stmt.condition, Some(1));
        self.instructions
            .push(Instruction::JumpTrue(continue_label.clone()));
        self.set_label(break_label);

        self.loop_stack.pop();
    }
    fn emit_statement_assignment(&mut self, stmt: lua_semantics::StmtAssignment) {
        let lhs_len = stmt.lhs.len();

        self.instructions.push(Instruction::Sp);
        let rhs_len = stmt.rhs.len();
        for (idx, rhs) in stmt.rhs.into_iter().enumerate() {
            if idx == rhs_len - 1 {
                self.emit_expression(rhs, None);
            } else {
                self.emit_expression(rhs, Some(1));
            }
        }
        // adjust rhs count
        self.instructions.push(Instruction::AdjustMultire(lhs_len));

        for lhs in stmt.lhs.into_iter().rev() {
            self.emit_expression_set(lhs);
        }
    }
    fn emit_statement_break(&mut self) {
        let break_label = match self.loop_stack.last() {
            Some(label) => label.clone(),
            None => {
                unreachable!("break outside loop");
            }
        };
        self.instructions.push(Instruction::Jump(break_label));
    }
    fn emit_statement_goto(&mut self, stmt: lua_semantics::StmtGoto) {
        self.instructions
            .push(Instruction::Jump(stmt.label.borrow().name.clone()));
    }
    fn emit_statement_label(&mut self, stmt: lua_semantics::StmtLabel) {
        self.set_label(stmt.label.borrow().name.clone());
    }
}
