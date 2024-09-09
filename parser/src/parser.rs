// This file must be explicitly converted to a `parser_expanded.rs` by `rustylr`

use lua_tokenizer::IntOrFloat;
use lua_tokenizer::Token;
use lua_tokenizer::TokenType;

use crate::expression;
use crate::Expression;
use crate::statement;
use crate::Statement;
use crate::IntType;
use crate::Span;
use crate::SpannedString;
use crate::ParseError;

// @TODO Block span

%%

%glr;
%lalr;
%tokentype Token;
%err ParseError;

%shift lparen;

%token ident Token::new_type(TokenType::Ident("".to_string()));

%token string_literal Token::new_type(TokenType::String("".to_string()));
%token numeric_literal Token::new_type(TokenType::Numeric(IntOrFloat::Int(0)));
%token nil Token::new_type(TokenType::Nil);
%token bool_ Token::new_type(TokenType::Bool(false));

%token plus Token::new_type(TokenType::Plus);
%token minus Token::new_type(TokenType::Minus);
%token asterisk Token::new_type(TokenType::Asterisk);
%token slash Token::new_type(TokenType::Slash);
%token percent Token::new_type(TokenType::Percent);
%token caret Token::new_type(TokenType::Caret);
%token hash Token::new_type(TokenType::Hash);
%token ampersand Token::new_type(TokenType::Ampersand);
%token tilde Token::new_type(TokenType::Tilde);
%token pipe Token::new_type(TokenType::Pipe);
%token lessless Token::new_type(TokenType::LessLess);
%token greatergreater Token::new_type(TokenType::GreaterGreater);
%token slashslash Token::new_type(TokenType::SlashSlash);
%token equalequal Token::new_type(TokenType::EqualEqual);
%token tildeequal Token::new_type(TokenType::TildeEqual);
%token lessequal Token::new_type(TokenType::LessEqual);
%token greaterequal Token::new_type(TokenType::GreaterEqual);
%token less Token::new_type(TokenType::Less);
%token greater Token::new_type(TokenType::Greater);
%token equal Token::new_type(TokenType::Equal);
%token lparen Token::new_type(TokenType::LParen);
%token rparen Token::new_type(TokenType::RParen);
%token lbrace Token::new_type(TokenType::LBrace);
%token rbrace Token::new_type(TokenType::RBrace);
%token lbracket Token::new_type(TokenType::LBracket);
%token rbracket Token::new_type(TokenType::RBracket);
%token coloncolon Token::new_type(TokenType::ColonColon);
%token semicolon Token::new_type(TokenType::Semicolon);
%token colon Token::new_type(TokenType::Colon);
%token comma Token::new_type(TokenType::Comma);
%token dot Token::new_type(TokenType::Dot);
%token dotdot Token::new_type(TokenType::DotDot);
%token dotdotdot Token::new_type(TokenType::DotDotDot);

%token and_ Token::new_type(TokenType::And);
%token break_ Token::new_type(TokenType::Break);
%token do_ Token::new_type(TokenType::Do);
%token else_ Token::new_type(TokenType::Else);
%token elseif_ Token::new_type(TokenType::Elseif);
%token end_ Token::new_type(TokenType::End);
%token for_ Token::new_type(TokenType::For);
%token function_ Token::new_type(TokenType::Function);
%token goto_ Token::new_type(TokenType::Goto);
%token if_ Token::new_type(TokenType::If);
%token in_ Token::new_type(TokenType::In);
%token local_ Token::new_type(TokenType::Local);
%token not_ Token::new_type(TokenType::Not);
%token or_ Token::new_type(TokenType::Or);
%token repeat_ Token::new_type(TokenType::Repeat);
%token return_ Token::new_type(TokenType::Return);
%token then_ Token::new_type(TokenType::Then);
%token until_ Token::new_type(TokenType::Until);
%token while_ Token::new_type(TokenType::While);
%eof Token::new_type(TokenType::Eof);

%start Chunk;

Chunk(statement::Block)
    : Block
    ;

Block(statement::Block)
    : Statement* ReturnStatement? {
        let span0 = if let Some(first) = Statement.first() {
            first.span()
        } else {
            Span::new_none()
        };
        if let Some(ret) = ReturnStatement {
            let span1 = ret.span();
            let span = span0.merge_ordered(&span1);
            statement::Block::new( Statement, Some(ret), span )
        } else {
            let span1 = if let Some(last) = Statement.last() {
                last.span()
            } else {
                Span::new_none()
            };
            let span = span0.merge_ordered(&span1);
            statement::Block::new( Statement, None, span )
        }
    }
    ;

Statement(Statement)
    : semicolon { Statement::None( statement::StmtNone::new(semicolon.span()) ) }
    | VarList equal ExpList1 {
        let span = VarList.first().unwrap().span().merge_ordered(&ExpList1.last().unwrap().span());
        let span_eq = equal.span();
        Statement::Assignment( statement::StmtAssignment::new(VarList, ExpList1, span, span_eq) )
    }
    | FunctionCall {
        Statement::FunctionCall(
            FunctionCall
        )
    }
    | c1=coloncolon ident c2=coloncolon {
        let span = c1.span().merge_ordered(&c2.span());
        Statement::Label( statement::StmtLabel::new(
            ident.into(),
            span
        ))
    }
    | break_ {
        Statement::Break( statement::StmtBreak::new( break_.span() ) )
    }
    | goto_ ident {
        let span = goto_.span().merge_ordered( &ident.span() );
        Statement::Goto( statement::StmtGoto::new(ident.into(), span) )
    }
    | do_ Block end_ {
        let span = do_.span().merge_ordered( &end_.span() );
        Statement::Do( statement::StmtDo::new(Block, span) )
    }
    | while_ Exp do_! Block end_ {
        let span = while_.span().merge_ordered( &end_.span() );
        Statement::While( statement::StmtWhile::new(Exp, Block, span) )
    }
    | repeat_ Block until_! Exp {
        let span = repeat_.span().merge_ordered( &Exp.span() );
        Statement::Repeat( statement::StmtRepeat::new(Block, Exp, span) )
    }
    | if_ Exp then_! Block elseifs=ElseIf* else_=(else_! Block)? end_ {
        let span = if_.span().merge_ordered( &end_.span() );
        Statement::If(
            statement::StmtIf::new(
                Exp,
                Block,
                elseifs,
                else_,
                span
            )
        )
    }
    | for_ ident equal! start=Exp comma! end=Exp step=(comma! Exp)? do_! Block end_ {
        let span = for_.span().merge_ordered( &end_.span() );
        Statement::For(
            statement::StmtFor::new(
                ident.token_type.into_ident().unwrap(),
                start,
                end,
                step.unwrap_or_else(|| {
                    // @TODO no none span
                    Expression::Numeric(expression::ExprNumeric::new(1.into(), Span::new_none()))
                }),
                Block,
                span,
            )
        )
    }
    | for_ NameList in_! ExpList1 do_! Block end_ {
        let span = for_.span().merge_ordered( &end_.span() );
        Statement::ForGeneric( statement::StmtForGeneric::new(NameList, ExpList1, Block, span) )
    }
    | function_ FuncName FuncBody {
        let span = function_.span().merge_ordered( &FuncBody.span() );
        Statement::FunctionDefinition(
            statement::StmtFunctionDefinition::new(FuncName, FuncBody, span)
        )
    }
    | local_ function_! ident FuncBody {
        let span = local_.span().merge_ordered( &FuncBody.span() );
        Statement::FunctionDefinitionLocal(
            statement::StmtFunctionDefinitionLocal::new(ident.into(), FuncBody, span)
        )
    }
    | local_ AttNameList rhs_list=(equal! ExpList1)? {
        let span0 = local_.span();
        if let Some(rhs) = rhs_list {
            let span = span0.merge_ordered( &rhs.last().unwrap().span() );
            Statement::LocalDeclaration( statement::StmtLocalDeclaration::new(AttNameList, Some(rhs), span) )
        } else {
            let span = AttNameList.last().unwrap().span();
            Statement::LocalDeclaration( statement::StmtLocalDeclaration::new(AttNameList, None, span) )
        }
    }
    ;

ElseIf(statement::StmtElseIf)
    : elseif_ Exp then_ Block {
        let span = if Block.span().is_none() {
            elseif_.span().merge_ordered( &then_.span() )
        } else {
            elseif_.span().merge_ordered( &Block.span() )
        };
        statement::StmtElseIf::new(Exp, Block, span)
    }
    ;

ReturnStatement(statement::ReturnStatement)
    : return_ ExpList0 semicolon? {
        let span0 = return_.span();
        let span = if let Some(last) = semicolon {
            span0.merge_ordered(&last.span())
        } else {
            if let Some(last) = ExpList0.last() {
                span0.merge_ordered(&last.span())
            } else {
                span0
            }
        };
        statement::ReturnStatement::new(ExpList0, span)
    }
    ;

Var(Expression)
    : ident {
        Expression::Ident( ident.into() )
    }
    | PrefixExp lbracket! Exp rbracket {
        let span = PrefixExp.span().merge_ordered(&rbracket.span());
        Expression::TableIndex( expression::ExprTableIndex::new(PrefixExp, Exp, span) )
    }
    | PrefixExp dot! ident {
        // a.b => a["b"]

        let span = PrefixExp.span().merge_ordered(&ident.span());
        let member = expression::ExprString::from(ident);

        Expression::TableIndex(
            expression::ExprTableIndex::new(
                PrefixExp,
                Expression::String(member),
                span
            )
        )
    }
    ;

PrefixExp(Expression)
    : Var
    | FunctionCall {
        Expression::FunctionCall(
            FunctionCall
        )
    }
    | lparen! Exp rparen!
    ;


FunctionCall(expression::ExprFunctionCall)
    : PrefixExp Args {
        let span = PrefixExp.span().merge_ordered(&Args.span());
        expression::ExprFunctionCall::new( PrefixExp, None, Args, span )
    }
    | PrefixExp colon! ident Args {
        let span = PrefixExp.span().merge_ordered(&Args.span());
        expression::ExprFunctionCall::new( PrefixExp, Some(ident.into()), Args, span )
    }
    ;

Args(expression::FunctionCallArguments)
    : lparen ExpList0 rparen {
        let span = lparen.span().merge_ordered(&rparen.span());
        expression::FunctionCallArguments::new( ExpList0, span )
    }
    | TableConstructor {
        let span = TableConstructor.span();
        let table_expr = Expression::Table( TableConstructor );
        let exprs = vec![table_expr];
        expression::FunctionCallArguments::new( exprs, span )
    }
    | string_literal {
        let span = string_literal.span();
        let exprs = vec![Expression::String(
            string_literal.into()
        )];
        expression::FunctionCallArguments::new( exprs, span )
    }
    ;

// one or more comma-separated Vars
VarList(Vec<Expression>)
    : VarList comma Var {
        VarList.push(Var);
        VarList
    }
    | Var { vec![Var] }
    ;

// one or more comma-separated expressions
ExpList1(Vec<Expression>)
    : ExpList1 comma Exp {
        ExpList1.push(Exp);
        ExpList1
    }
    | Exp {
        vec![Exp]
    }
    ;
// zero or more comma-separated expressions
ExpList0(Vec<Expression>)
    : ExpList1 {
        ExpList1
    }
    | {
        vec![]
    }
    ;

// one or more comma-separated names
NameList(Vec<SpannedString>)
    : NameList comma! ident {
        NameList.push(ident.into());
        NameList
    }
    | ident {
        vec![ident.into()]
    }
    ;

AttName(statement::AttName)
    : ident Attrib {
        let span = ident.span();
        statement::AttName::new( ident.into(), Attrib, span )
    }
    ;
AttNameList(Vec<statement::AttName>)
    : AttNameList comma! AttName {
        AttNameList.push( AttName );
        AttNameList
    }
    | AttName {
        vec![ AttName ]
    }
    ;

Attrib(Option<statement::Attrib>)
    : less! ident greater! {
        let s:SpannedString = ident.into();
        match s.as_str() {
            "const" => Some(statement::Attrib::Const),
            "close" => Some(statement::Attrib::Close),
            _ => {
                return Err( ParseError::UnknownAttribute(s) );
            }
        }
    }
    | { None }
    ;



Exp(Expression)
    : Exp12
    ;

Exp0(Expression)
    : numeric_literal {
        Expression::Numeric(
            numeric_literal.into()
        )
    }
    | nil {
        Expression::Nil( nil.into() )
    }
    | string_literal {
        Expression::String(
            string_literal.into()
        )
    }
    | bool_ {
        Expression::Bool(
            bool_.into()
        )
    }
    | dotdotdot {
        Expression::Variadic( dotdotdot.into() )
    }
    | FunctionDef {
        Expression::Function( FunctionDef )
    }
    | PrefixExp
    | TableConstructor {
        Expression::Table( TableConstructor )
    }
    ;



Exp1(Expression)
    : Exp0 caret Exp1 {
        let span = Exp0.span().merge_ordered(&Exp1.span());
        let span_op = caret.span();
        let binary_data = expression::ExprBinaryData::new(Exp0, Exp1, span, span_op);

        Expression::Binary(
            expression::ExprBinary::Pow(
                binary_data
            )
        )
    }
    | Exp0
    ;

Exp2(Expression)
    : not_ Exp2 {
        let span = not_.span().merge_ordered(&Exp2.span());
        let span_op = not_.span();
        let unary_data = expression::ExprUnaryData::new(Exp2, span, span_op);
        Expression::Unary(
            expression::ExprUnary::LogicalNot(
                unary_data
            )
        )
    }
    | hash Exp2 {
        let span = hash.span().merge_ordered(&Exp2.span());
        let span_op = hash.span();
        let unary_data = expression::ExprUnaryData::new(Exp2, span, span_op);
        Expression::Unary(
            expression::ExprUnary::Length(
                unary_data
            )
        )
    }
    | minus Exp2 {
        let span = minus.span().merge_ordered(&Exp2.span());
        let span_op = minus.span();
        let unary_data = expression::ExprUnaryData::new(Exp2, span, span_op);
        Expression::Unary(
            expression::ExprUnary::Minus(
                unary_data
            )
        )
    }
    | plus Exp2 {
        let span = plus.span().merge_ordered(&Exp2.span());
        let span_op = plus.span();
        let unary_data = expression::ExprUnaryData::new(Exp2, span, span_op);
        Expression::Unary(
            expression::ExprUnary::Plus(
                unary_data
            )
        )
    }
    | tilde Exp2 {
        let span = tilde.span().merge_ordered(&Exp2.span());
        let span_op = tilde.span();
        let unary_data = expression::ExprUnaryData::new(Exp2, span, span_op);
        Expression::Unary(
            expression::ExprUnary::BitwiseNot(
                unary_data
            )
        )
    }
    | Exp1
    ;

Exp3(Expression)
    : Exp3 asterisk Exp2 {
        let span = Exp3.span().merge_ordered(&Exp2.span());
        let span_op = asterisk.span();
        let binary_data = expression::ExprBinaryData::new(Exp3, Exp2, span, span_op);
        Expression::Binary(
            expression::ExprBinary::Mul(
                binary_data
            )
        )
    }
    | Exp3 slash Exp2 {
        let span = Exp3.span().merge_ordered(&Exp2.span());
        let span_op = slash.span();
        let binary_data = expression::ExprBinaryData::new(Exp3, Exp2, span, span_op);
        Expression::Binary(
            expression::ExprBinary::Div(
                binary_data
            )
        )
    }
    | Exp3 slashslash Exp2 {
        let span = Exp3.span().merge_ordered(&Exp2.span());
        let span_op = slashslash.span();
        let binary_data = expression::ExprBinaryData::new(Exp3, Exp2, span, span_op);
        Expression::Binary(
            expression::ExprBinary::FloorDiv(
                binary_data
            )
        )
    }
    | Exp3 percent Exp2 {
        let span = Exp3.span().merge_ordered(&Exp2.span());
        let span_op = percent.span();
        let binary_data = expression::ExprBinaryData::new(Exp3, Exp2, span, span_op);
        Expression::Binary(
            expression::ExprBinary::Mod(
                binary_data
            )
        )
    }
    | Exp2
    ;

Exp4(Expression)
    : Exp4 plus Exp3 {
        let span = Exp4.span().merge_ordered(&Exp3.span());
        let span_op = plus.span();
        let binary_data = expression::ExprBinaryData::new(Exp4, Exp3, span, span_op);
        Expression::Binary(
            expression::ExprBinary::Add(
                binary_data
            )
        )
    }
    | Exp4 minus Exp3 {
        let span = Exp4.span().merge_ordered(&Exp3.span());
        let span_op = minus.span();
        let binary_data = expression::ExprBinaryData::new(Exp4, Exp3, span, span_op);
        Expression::Binary(
            expression::ExprBinary::Sub(
                binary_data
            )
        )
    }
    | Exp3
    ;

Exp5(Expression)
    // right associative for concat '..'
    : Exp4 dotdot Exp5 {
        let span = Exp4.span().merge_ordered(&Exp5.span());
        let span_op = dotdot.span();
        let binary_data = expression::ExprBinaryData::new(Exp4, Exp5, span, span_op);
        Expression::Binary(
            expression::ExprBinary::Concat(
                binary_data
            )
        )
    }
    | Exp4
    ;

Exp6(Expression)
    : Exp6 lessless Exp5 {
        let span = Exp6.span().merge_ordered(&Exp5.span());
        let span_op = lessless.span();
        let binary_data = expression::ExprBinaryData::new(Exp6, Exp5, span, span_op);
        Expression::Binary(
            expression::ExprBinary::ShiftLeft(
                binary_data
            )
        )
    }
    | Exp6 greatergreater Exp5 {
        let span = Exp6.span().merge_ordered(&Exp5.span());
        let span_op = greatergreater.span();
        let binary_data = expression::ExprBinaryData::new(Exp6, Exp5, span, span_op);
        Expression::Binary(
            expression::ExprBinary::ShiftRight(
                binary_data
            )
        )
    }
    | Exp5
    ;

Exp7(Expression)
    : Exp7 ampersand Exp6 {
        let span = Exp7.span().merge_ordered(&Exp6.span());
        let span_op = ampersand.span();
        let binary_data = expression::ExprBinaryData::new(Exp7, Exp6, span, span_op);
        Expression::Binary(
            expression::ExprBinary::BitwiseAnd(
                binary_data
            )
        )
    }
    | Exp6
    ;

Exp8(Expression)
    : Exp8 tilde Exp7 {
        let span = Exp8.span().merge_ordered(&Exp7.span());
        let span_op = tilde.span();
        let binary_data = expression::ExprBinaryData::new(Exp8, Exp7, span, span_op);
        Expression::Binary(
            expression::ExprBinary::BitwiseXor(
                binary_data
            )
        )
    }
    | Exp7
    ;

Exp9(Expression)
    : Exp9 pipe Exp8 {
        let span = Exp9.span().merge_ordered(&Exp8.span());
        let span_op = pipe.span();
        let binary_data = expression::ExprBinaryData::new(Exp9, Exp8, span, span_op);
        Expression::Binary(
            expression::ExprBinary::BitwiseOr(
                binary_data
            )
        )
    }
    | Exp8
    ;

Exp10(Expression)
    : Exp10 less Exp9 {
        let span = Exp10.span().merge_ordered(&Exp9.span());
        let span_op = less.span();
        let binary_data = expression::ExprBinaryData::new(Exp10, Exp9, span, span_op);
        Expression::Binary(
            expression::ExprBinary::LessThan(
                binary_data
            )
        )
    }
    | Exp10 lessequal Exp9 {
        let span = Exp10.span().merge_ordered(&Exp9.span());
        let span_op = lessequal.span();
        let binary_data = expression::ExprBinaryData::new(Exp10, Exp9, span, span_op);
        Expression::Binary(
            expression::ExprBinary::LessEqual(
                binary_data
            )
        )
    }
    | Exp10 greater Exp9 {
        let span = Exp10.span().merge_ordered(&Exp9.span());
        let span_op = greater.span();
        let binary_data = expression::ExprBinaryData::new(Exp10, Exp9, span, span_op);
        Expression::Binary(
            expression::ExprBinary::GreaterThan(
                binary_data
            )
        )
    }
    | Exp10 greaterequal Exp9 {
        let span = Exp10.span().merge_ordered(&Exp9.span());
        let span_op = greaterequal.span();
        let binary_data = expression::ExprBinaryData::new(Exp10, Exp9, span, span_op);
        Expression::Binary(
            expression::ExprBinary::GreaterEqual(
                binary_data
            )
        )
    }
    | Exp10 tildeequal Exp9 {
        let span = Exp10.span().merge_ordered(&Exp9.span());
        let span_op = tildeequal.span();
        let binary_data = expression::ExprBinaryData::new(Exp10, Exp9, span, span_op);
        Expression::Binary(
            expression::ExprBinary::NotEqual(
                binary_data
            )
        )
    }
    | Exp10 equalequal Exp9 {
        let span = Exp10.span().merge_ordered(&Exp9.span());
        let span_op = equalequal.span();
        let binary_data = expression::ExprBinaryData::new(Exp10, Exp9, span, span_op);
        Expression::Binary(
            expression::ExprBinary::Equal(
                binary_data
            )
        )
    }
    | Exp9
    ;

Exp11(Expression)
    : Exp11 and_ Exp10 {
        let span = Exp11.span().merge_ordered(&Exp10.span());
        let span_op = and_.span();
        let binary_data = expression::ExprBinaryData::new(Exp11, Exp10, span, span_op);
        Expression::Binary(
            expression::ExprBinary::LogicalAnd(
                binary_data
            )
        )
    }
    | Exp10
    ;

Exp12(Expression)
    : Exp12 or_ Exp11 {
        let span = Exp12.span().merge_ordered(&Exp11.span());
        let span_op = or_.span();
        let binary_data = expression::ExprBinaryData::new(Exp12, Exp11, span, span_op);
        Expression::Binary(
            expression::ExprBinary::LogicalOr(
                binary_data
            )
        )
    }
    | Exp11
    ;

TableConstructor(expression::ExprTable)
    : lbrace FieldList rbrace {
        let span = lbrace.span().merge_ordered(&rbrace.span());
        let mut table = expression::ExprTable::new(span);
        // for no-key value in FieldList
        let mut consecutive:IntType = 1;
        for field in FieldList.into_iter() {
            match field {
                // [k] = v
                expression::TableConstructorFieldBuilder::KeyValue(k, v) => {
                    let span = k.span().merge_ordered(&v.span());
                    table.fields.push(
                        expression::TableField::new(k, v, span)
                    );
                }
                // 'k' = v
                expression::TableConstructorFieldBuilder::NameValue(name, v) => {
                    let span = name.span().merge_ordered(&v.span());
                    table.fields.push(expression::TableField::new(
                        Expression::String(name.into()),
                        v,
                        span,
                    ));
                }
                // v
                expression::TableConstructorFieldBuilder::Value(v) => {
                    let idx = consecutive;
                    consecutive += 1;
                    let span = v.span();
                    table.fields.push(expression::TableField::new(
                        Expression::Numeric(expression::ExprNumeric::new(
                            idx.into(),
                            // @TODO no none span
                            Span::new_none(),
                        )),
                        v,
                        span,
                    ));
                }
            }
        }
        table
    }
    ;


// one or more separated Fields
FieldList1(Vec<expression::TableConstructorFieldBuilder>)
    : FieldList1 FieldSep Field {
        FieldList1.push(Field);
        FieldList1
    }
    | Field {
        vec![Field]
    }
    ;

// zero or more separated Fields, with optional trailing separator
FieldList(Vec<expression::TableConstructorFieldBuilder>)
    : FieldList1 FieldSep? {
        FieldList1
    }
    | {
        vec![]
    }
    ;

Field(expression::TableConstructorFieldBuilder)
    : lbracket! k=Exp rbracket! equal! v=Exp {
        expression::TableConstructorFieldBuilder::KeyValue(k, v)
    }
    | ident equal! Exp {
        expression::TableConstructorFieldBuilder::NameValue(ident.into(), Exp)
    }
    | Exp {
        expression::TableConstructorFieldBuilder::Value(Exp)
    }
    ;

FieldSep: comma | semicolon ;


FunctionDef(expression::ExprFunction)
    : function_ FuncBody {
        let span = function_.span().merge_ordered(&FuncBody.span());
        FuncBody.span = span;
        FuncBody
    }
    ;

FuncBody(expression::ExprFunction)
    : lparen ParList? rparen! Block end_ {
        let span = lparen.span().merge_ordered(&end_.span());
        expression::ExprFunction::new(ParList, Block, span)
    }
    ;

// dot chained ident
FuncName1(Vec<SpannedString>)
    : FuncName1 dot ident {
        FuncName1.push( ident.into() );
        FuncName1
    }
    | ident {
        vec![ident.into()]
    }
    ;

FuncName(statement::FunctionName)
    : FuncName1 colon! ident {
        let span = FuncName1.first().unwrap().span().merge_ordered(&ident.span());
        statement::FunctionName::new( FuncName1, Some(ident.into()), span )
    }
    | FuncName1 {
        let span = FuncName1.first().unwrap().span().merge_ordered(
            &FuncName1.last().unwrap().span()
        );
        statement::FunctionName::new( FuncName1, None, span )
    }
    ;

ParList(expression::ParameterList)
    : NameList var=(comma! dotdotdot)? {
        if let Some(var) = var {
            let span = NameList.first().unwrap().span().merge_ordered(&var.span());
            expression::ParameterList::new( NameList, true, span )
        } else {
            let span = NameList.first().unwrap().span();
            expression::ParameterList::new( NameList, false, span )
        }
    }
    | dotdotdot {
        expression::ParameterList::new( Vec::new(), true, dotdotdot.span() )
    }
    ;