/// This file must be explicitly converted to a `parser_expanded.rs` by `rustylr`

use lua_tokenizer::IntOrFloat;
use lua_tokenizer::Token;
use lua_tokenizer::TokenType;

use crate::expression;
use crate::Expression;
use crate::statement;
use crate::Statement;
use crate::IntType;

%%

%glr;
%lalr;
%tokentype Token;

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
        statement::Block::new( Statement, ReturnStatement )
    }
    ;

Statement(Statement)
    : semicolon! { Statement::None }
    | VarList equal! ExpList1 {
        Statement::Assignment( statement::StmtAssignment::new(VarList, ExpList1) )
    }
    | FunctionCall {
        Statement::FunctionCall(
            statement::StmtFunctionCall::new(
                FunctionCall.0,
                FunctionCall.1
            )
        )
    }
    | coloncolon! ident coloncolon! {
        Statement::Label( statement::StmtLabel::new(ident.token_type.into_ident().unwrap()) )
    }
    | break_! {
        Statement::Break( statement::StmtBreak{} )
    }
    | goto_! ident {
        Statement::Goto( statement::StmtGoto::new(ident.token_type.into_ident().unwrap()) )
    }
    | do_! Block end_! {
        Statement::Do( statement::StmtDo::new(Block) )
    }
    | while_! Exp do_! Block end_! {
        Statement::While( statement::StmtWhile::new(Exp, Block) )
    }
    | repeat_! Block until_! Exp {
        Statement::Repeat( statement::StmtRepeat::new(Block, Exp) )
    }
    | if_! Exp then_! Block elseifs=(elseif_! Exp then_! Block)* else_=(else_! Block)? end_! {
        Statement::If(
            statement::StmtIf::new(
                Exp,
                Block,
                elseifs,
                else_
            )
        )
    }
    | for_! ident equal! start=Exp comma! end=Exp step=(comma! Exp)? do_! Block end_! {
        Statement::For(
            statement::StmtFor::new(
                ident.token_type.into_ident().unwrap(),
                start,
                end,
                step.unwrap_or_else(|| Expression::from(1)),
                Block
            )
        )
    }
    | for_! NameList in_! ExpList1 do_! Block end_! {
        Statement::ForGeneric( statement::StmtForGeneric::new(NameList, ExpList1, Block) )
    }
    | function_! FuncName FuncBody {
        Statement::FunctionDefinition(
            statement::StmtFunctionDefinition::new(FuncName, FuncBody)
        )
    }
    | local_! function_! ident FuncBody {
        Statement::FunctionDefinitionLocal(
            statement::StmtFunctionDefinitionLocal::new(ident.token_type.into_ident().unwrap(), FuncBody)
        )
    }
    | local_! AttNameList rhs_list=(equal! ExpList1)? {
        Statement::LocalDeclaration( statement::StmtLocalDeclaration::new(AttNameList, rhs_list) )
    }
    ;

ReturnStatement(statement::ReturnStatement)
    : return_! ExpList0 semicolon? {
        statement::ReturnStatement::new(ExpList0)
    }
    ;

Var(Expression)
    : ident {
        Expression::new_ident(ident.token_type.into_ident().unwrap())
    }
    | PrefixExp lbracket! Exp rbracket! {
        Expression::TableIndex( expression::ExprTableIndex{ table:Box::new(PrefixExp), index:Box::new(Exp) } )
    }
    | PrefixExp dot! ident {
        let member = ident.token_type.into_ident().unwrap();
        // a.b => a["b"]

        Expression::TableIndex(
            expression::ExprTableIndex{
                table:Box::new(PrefixExp),
                index:Box::new(
                    Expression::from(member)
                )
            }
        )
    }
    ;

PrefixExp(Expression)
    : Var
    | FunctionCall {
        Expression::FunctionCall(
            expression::ExprFunctionCall::new( FunctionCall.0, FunctionCall.1 )
        )
    }
    | lparen! Exp rparen!
    ;


FunctionCall((Expression, Vec<Expression>))
    : PrefixExp Args {
        (PrefixExp, Args)
    }
    | PrefixExp colon! ident Args {
        // v:name(args) => v.name(v,args)
        // copy PrefixExp for arg0
        let arg0 = PrefixExp.clone();
        let mut args = Vec::with_capacity( Args.len() + 1 );
        args.push( arg0 );
        args.extend( Args );

        // get v.name
        let member = Expression::TableIndex(
            expression::ExprTableIndex::new( PrefixExp, Expression::from(ident.token_type.into_ident().unwrap()) )
        );

        (member, args)
    }
    ;

Args(Vec<Expression>)
    : lparen! ExpList0 rparen!
    | TableConstructor {
        let table_expr = Expression::Table( TableConstructor );
        vec![table_expr]
    }
    | string_literal {
        vec![Expression::from(
            string_literal.token_type.into_string().unwrap()
        )]
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
NameList(Vec<String>)
    : NameList comma! ident {
        NameList.push(ident.token_type.into_ident().unwrap());
        NameList
    }
    | ident {
        vec![ident.token_type.into_ident().unwrap()]
    }
    ;

AttName(statement::AttName)
    : ident Attrib {
        statement::AttName::new( ident.token_type.into_ident().unwrap(), Attrib )
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
        let s = ident.token_type.into_ident().unwrap();
        match s.as_str() {
            "const" => Some(statement::Attrib::Const),
            "close" => Some(statement::Attrib::Close),
            _ => {
                // @TODO error
                Some(statement::Attrib::Const)
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
        Expression::from(
            numeric_literal.token_type.into_numeric().unwrap()
        )
    }
    | nil {
        Expression::Nil( expression::ExprNil )
    }
    | string_literal {
        Expression::from(
            string_literal.token_type.into_string().unwrap()
        )
    }
    | bool_ {
        Expression::from(
            bool_.token_type.into_bool().unwrap()
        )
    }
    | dotdotdot {
        Expression::Variadic
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
        Expression::Binary(
            expression::ExprBinary::Pow(
                Box::new(Exp0),
                Box::new(Exp1)
            )
        )
    }
    | Exp0
    ;

Exp2(Expression)
    : not_ Exp2 {
        Expression::Unary(
            expression::ExprUnary::LogicalNot(
                Box::new(Exp2)
            )
        )
    }
    | hash Exp2 {
        Expression::Unary(
            expression::ExprUnary::Length(
                Box::new(Exp2)
            )
        )
    }
    | minus Exp2 {
        Expression::Unary(
            expression::ExprUnary::Minus(
                Box::new(Exp2)
            )
        )
    }
    | tilde Exp2 {
        Expression::Unary(
            expression::ExprUnary::BitwiseNot(
                Box::new(Exp2)
            )
        )
    }
    | Exp1
    ;

Exp3(Expression)
    : Exp3 asterisk Exp2 {
        Expression::Binary(
            expression::ExprBinary::Mul(
                Box::new(Exp3),
                Box::new(Exp2)
            )
        )
    }
    | Exp3 slash Exp2 {
        Expression::Binary(
            expression::ExprBinary::Div(
                Box::new(Exp3),
                Box::new(Exp2)
            )
        )
    }
    | Exp3 slashslash Exp2 {
        Expression::Binary(
            expression::ExprBinary::FloorDiv(
                Box::new(Exp3),
                Box::new(Exp2)
            )
        )
    }
    | Exp3 percent Exp2 {
        Expression::Binary(
            expression::ExprBinary::Mod(
                Box::new(Exp3),
                Box::new(Exp2)
            )
        )
    }
    | Exp2
    ;

Exp4(Expression)
    : Exp4 plus Exp3 {
        Expression::Binary(
            expression::ExprBinary::Add(
                Box::new(Exp4),
                Box::new(Exp3)
            )
        )
    }
    | Exp4 minus Exp3 {
        Expression::Binary(
            expression::ExprBinary::Sub(
                Box::new(Exp4),
                Box::new(Exp3)
            )
        )
    }
    | Exp3
    ;

Exp5(Expression)
    // right associative for concat '..'
    : Exp4 dotdot Exp5 {
        Expression::Binary(
            expression::ExprBinary::Concat(
                Box::new(Exp4),
                Box::new(Exp5)
            )
        )
    }
    | Exp4
    ;

Exp6(Expression)
    : Exp6 lessless Exp5 {
        Expression::Binary(
            expression::ExprBinary::ShiftLeft(
                Box::new(Exp6),
                Box::new(Exp5)
            )
        )
    }
    | Exp6 greatergreater Exp5 {
        Expression::Binary(
            expression::ExprBinary::ShiftRight(
                Box::new(Exp6),
                Box::new(Exp5)
            )
        )
    }
    | Exp5
    ;

Exp7(Expression)
    : Exp7 ampersand Exp6 {
        Expression::Binary(
            expression::ExprBinary::BitwiseAnd(
                Box::new(Exp7),
                Box::new(Exp6)
            )
        )
    }
    | Exp6
    ;

Exp8(Expression)
    : Exp8 tilde Exp7 {
        Expression::Binary(
            expression::ExprBinary::BitwiseXor(
                Box::new(Exp8),
                Box::new(Exp7)
            )
        )
    }
    | Exp7
    ;

Exp9(Expression)
    : Exp9 pipe Exp8 {
        Expression::Binary(
            expression::ExprBinary::BitwiseOr(
                Box::new(Exp9),
                Box::new(Exp8)
            )
        )
    }
    | Exp8
    ;

Exp10(Expression)
    : Exp10 less Exp9 {
        Expression::Binary(
            expression::ExprBinary::LessThan(
                Box::new(Exp10),
                Box::new(Exp9)
            )
        )
    }
    | Exp10 lessequal Exp9 {
        Expression::Binary(
            expression::ExprBinary::LessEqual(
                Box::new(Exp10),
                Box::new(Exp9)
            )
        )
    }
    | Exp10 greater Exp9 {
        Expression::Binary(
            expression::ExprBinary::GreaterThan(
                Box::new(Exp10),
                Box::new(Exp9)
            )
        )
    }
    | Exp10 greaterequal Exp9 {
        Expression::Binary(
            expression::ExprBinary::GreaterEqual(
                Box::new(Exp10),
                Box::new(Exp9)
            )
        )
    }
    | Exp10 tildeequal Exp9 {
        Expression::Binary(
            expression::ExprBinary::NotEqual(
                Box::new(Exp10),
                Box::new(Exp9)
            )
        )
    }
    | Exp10 equalequal Exp9 {
        Expression::Binary(
            expression::ExprBinary::Equal(
                Box::new(Exp10),
                Box::new(Exp9)
            )
        )
    }
    | Exp9
    ;

Exp11(Expression)
    : Exp11 and_ Exp10 {
        Expression::Binary(
            expression::ExprBinary::LogicalAnd(
                Box::new(Exp11),
                Box::new(Exp10)
            )
        )
    }
    | Exp10
    ;

Exp12(Expression)
    : Exp12 or_ Exp11 {
        Expression::Binary(
            expression::ExprBinary::LogicalOr(
                Box::new(Exp12),
                Box::new(Exp11)
            )
        )
    }
    | Exp11
    ;

TableConstructor(expression::ExprTable)
    : lbrace! FieldList rbrace! {
        let mut table = expression::ExprTable::new();
        // for no-key value in FieldList
        let mut consecutive:IntType = 1;
        for field in FieldList.into_iter() {
            match field {
                // [k] = v
                expression::TableConstructorFieldBuilder::KeyValue(k, v) => {
                    table.fields.push(
                        expression::TableField::new(k, v)
                    );
                }
                // 'k' = v
                expression::TableConstructorFieldBuilder::NameValue(name, v) => {
                    table.fields.push(
                        expression::TableField::new(name.into(), v)
                    );
                }
                // v
                expression::TableConstructorFieldBuilder::Value(v) => {
                    let idx = consecutive;
                    consecutive += 1;
                    table.fields.push(
                        expression::TableField::new(idx.into(), v)
                    );
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
        let name = ident.token_type.into_ident().unwrap();
        expression::TableConstructorFieldBuilder::NameValue(name, Exp)
    }
    | Exp {
        expression::TableConstructorFieldBuilder::Value(Exp)
    }
    ;

FieldSep: comma | semicolon ;


FunctionDef(expression::ExprFunction)
    : function_! FuncBody
    ;

FuncBody(expression::ExprFunction)
    : lparen! ParList? rparen! Block end_! {
        expression::ExprFunction::new(ParList, Block)
    }
    ;

// dot chained ident
FuncName1(Vec<String>)
    : FuncName1 dot ident {
        FuncName1.push( ident.token_type.into_ident().unwrap() );
        FuncName1
    }
    | ident {
        vec![ident.token_type.into_ident().unwrap()]
    }
    ;

FuncName(statement::FunctionName)
    : FuncName1 colon! ident {
        statement::FunctionName::new( FuncName1, Some(ident.token_type.into_ident().unwrap()) )
    }
    | FuncName1 {
        statement::FunctionName::new( FuncName1, None )
    }
    ;

ParList(expression::ParameterList)
    : NameList var=(comma! dotdotdot)? {
        expression::ParameterList::new( NameList, var.is_some() )
    }
    | dotdotdot {
        expression::ParameterList::new( Vec::new(), true )
    }
    ;