use lua_tokenizer::IntOrFloat;
use lua_tokenizer::Token;
use lua_tokenizer::TokenType;

use crate::expression;
use crate::Expression;
use crate::statement;
use crate::Statement;

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
        Statement::Assignment( statement::Assignment::new(VarList, ExpList1) )
    }
    | FunctionCall {
        Statement::FunctionCall(FunctionCall)
    }
    | coloncolon! ident coloncolon! {
        if let TokenType::Ident(s) = ident.token_type {
            Statement::Label( statement::Label::new(s) )
        } else {
            unreachable!( "m" );
        }
    }
    | break_! {
        Statement::Break
    }
    | goto_! ident {
        if let TokenType::Ident(s) = ident.token_type {
            Statement::Goto( statement::Goto::new(s) )
        } else {
            unreachable!( "n" );
        }
    }
    | do_! Block end_! {
        Statement::Do( statement::Do::new(Block) )
    }
    | while_! Exp do_! Block end_! {
        Statement::While( statement::While::new(Exp, Block) )
    }
    | repeat_! Block until_! Exp {
        Statement::Repeat( statement::Repeat::new(Block, Exp) )
    }
    | if_! Exp then_! Block elseifs=(elseif_! Exp then_! Block)* else_=(else_! Block)? end_! {
        Statement::If(
            statement::If::new(
                Exp,
                Block,
                elseifs,
                else_
            )
        )
    }
    | for_! ident equal! start=Exp comma! end=Exp step=(comma! Exp)? do_! Block end_! {
        if let TokenType::Ident(s) = ident.token_type {
            Statement::For(
                statement::For::new(
                    s,
                    start,
                    end,
                    step,
                    Block
                )
            )
        } else {
            unreachable!( "o" );
        }
    }
    | for_! NameList in_! ExpList1 do_! Block end_! {
        Statement::ForGeneric( statement::ForGeneric::new(NameList, ExpList1, Block) )
    }
    | function_! FuncName FuncBody {
        Statement::FunctionDefinition(
            statement::FunctionDefinition::new(FuncName, FuncBody)
        )
    }
    | local_! function_! ident FuncBody {
        if let TokenType::Ident(s) = ident.token_type {
            Statement::FunctionDefinitionLocal(
                statement::FunctionDefinitionLocal::new(s, FuncBody)
            )
        } else {
            unreachable!( "p" );
        }
    }
    | local_! AttNameList rhs_list=(equal! ExpList1)? {
        Statement::LocalDeclaration( statement::LocalDeclaration::new(AttNameList, rhs_list) )
    }
    ;

ReturnStatement(statement::ReturnStatement)
    : return_! ExpList0 semicolon? {
        statement::ReturnStatement::new(ExpList0)
    }
    ;

Var(Expression)
    : ident {
        if let TokenType::Ident(s) = ident.token_type {
            Expression::Ident( expression::Ident{ name:s } )
        } else {
            unreachable!( "i" );
        }
    }
    | PrefixExp lbracket! Exp rbracket! {
        Expression::TableIndex( expression::TableIndex{ table:Box::new(PrefixExp), index:Box::new(Exp) } )
    }
    | PrefixExp dot! ident {
        let member = if let TokenType::Ident(s) = ident.token_type {
            s
        } else {
            unreachable!( "j" );
        };
        // a.b => a["b"]

        Expression::TableIndex(
            expression::TableIndex{
                table:Box::new(PrefixExp),
                index:Box::new(
                    Expression::String( expression::StringLiteral{ value:member } )
                )
            }
        )
    }
    ;

PrefixExp(Expression)
    : Var
    | FunctionCall {
        Expression::FunctionCall( FunctionCall )
    }
    | lparen! Exp rparen!
    ;


FunctionCall(expression::FunctionCall)
    : PrefixExp Args {
        expression::FunctionCall::new(
            PrefixExp,
            Args
        )
    }
    | PrefixExp colon! ident Args {
        // v:name(args) => v.name(v,args)
        if let TokenType::Ident(s) = ident.token_type {
            // copy PrefixExp for arg0
            let arg0 = PrefixExp.clone();
            let mut args = Vec::with_capacity( Args.len() + 1 );
            args.push( arg0 );
            args.extend( Args );

            // get v.name
            let member = Expression::TableIndex(
                expression::TableIndex::new( PrefixExp, Expression::String( expression::StringLiteral::new(s) ) )
            );

            expression::FunctionCall::new(
                member,
                args
            )
        } else {
            unreachable!( "k" );
        }
    }
    ;

Args(Vec<Expression>)
    : lparen! ExpList0 rparen!
    | TableConstructor {
        let table_expr = Expression::TableConstructor( TableConstructor );
        vec![table_expr]
    }
    | string_literal {
        if let TokenType::String(s) = string_literal.token_type {
            let string_expr = Expression::String( expression::StringLiteral::new(s) );
            vec![string_expr]
        } else {
            unreachable!( "k" );
        }
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
        if let TokenType::Ident(s) = ident.token_type {
            NameList.push(s);
            NameList
        } else {
            unreachable!( "l" );
        }
    }
    | ident {
        if let TokenType::Ident(s) = ident.token_type {
            vec![s]
        } else {
            unreachable!( "l" );
        }
    }
    ;

AttName(statement::AttName)
    : ident Attrib {
        if let TokenType::Ident(s) = ident.token_type {
            statement::AttName::new( s, Attrib )
        } else {
            unreachable!( "m" );
        }
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
        if let TokenType::Ident(s) = ident.token_type {
            match s.as_str() {
                "const" => Some(statement::Attrib::Const),
                "close" => Some(statement::Attrib::Close),
                _ => {
                    // @TODO error
                    Some(statement::Attrib::Const)
                }
            }
        } else {
            unreachable!( "m" );
        }
    }
    | { None }
    ;



Exp(Expression)
    : Exp12
    ;

Exp0(Expression)
    : numeric_literal {
        if let TokenType::Numeric(iorf) = numeric_literal.token_type {
            Expression::Numeric( expression::Numeric{ value:iorf } )
        } else {
            unreachable!( "0" );
        }
    }
    | nil {
        Expression::Nil
    }
    | string_literal {
        if let TokenType::String(s) = string_literal.token_type {
            Expression::String( expression::StringLiteral{ value:s } )
        } else {
            unreachable!( "1" );
        }
    }
    | bool_ {
        if let TokenType::Bool(b) = bool_.token_type {
            Expression::Bool( expression::Bool{ value:b } )
        } else {
            unreachable!( "2" );
        }
    }
    | dotdotdot {
        Expression::Variadic
    }
    | FunctionDef {
        Expression::FunctionDef( FunctionDef )
    }
    | PrefixExp
    | TableConstructor {
        Expression::TableConstructor( TableConstructor )
    }
    ;



Exp1(Expression)
    : Exp0 caret Exp1 {
        Expression::Binary(
            expression::Binary::Pow(
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
            expression::Unary::LogicalNot(
                Box::new(Exp2)
            )
        )
    }
    | hash Exp2 {
        Expression::Unary(
            expression::Unary::Length(
                Box::new(Exp2)
            )
        )
    }
    | minus Exp2 {
        Expression::Unary(
            expression::Unary::Minus(
                Box::new(Exp2)
            )
        )
    }
    | tilde Exp2 {
        Expression::Unary(
            expression::Unary::BitwiseNot(
                Box::new(Exp2)
            )
        )
    }
    | Exp1
    ;

Exp3(Expression)
    : Exp3 asterisk Exp2 {
        Expression::Binary(
            expression::Binary::Mul(
                Box::new(Exp3),
                Box::new(Exp2)
            )
        )
    }
    | Exp3 slash Exp2 {
        Expression::Binary(
            expression::Binary::Div(
                Box::new(Exp3),
                Box::new(Exp2)
            )
        )
    }
    | Exp3 slashslash Exp2 {
        Expression::Binary(
            expression::Binary::FloorDiv(
                Box::new(Exp3),
                Box::new(Exp2)
            )
        )
    }
    | Exp3 percent Exp2 {
        Expression::Binary(
            expression::Binary::Mod(
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
            expression::Binary::Add(
                Box::new(Exp4),
                Box::new(Exp3)
            )
        )
    }
    | Exp4 minus Exp3 {
        Expression::Binary(
            expression::Binary::Sub(
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
            expression::Binary::Concat(
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
            expression::Binary::ShiftLeft(
                Box::new(Exp6),
                Box::new(Exp5)
            )
        )
    }
    | Exp6 greatergreater Exp5 {
        Expression::Binary(
            expression::Binary::ShiftRight(
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
            expression::Binary::BitwiseAnd(
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
            expression::Binary::BitwiseXor(
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
            expression::Binary::BitwiseOr(
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
            expression::Binary::LessThan(
                Box::new(Exp10),
                Box::new(Exp9)
            )
        )
    }
    | Exp10 lessequal Exp9 {
        Expression::Binary(
            expression::Binary::LessEqual(
                Box::new(Exp10),
                Box::new(Exp9)
            )
        )
    }
    | Exp10 greater Exp9 {
        Expression::Binary(
            expression::Binary::GreaterThan(
                Box::new(Exp10),
                Box::new(Exp9)
            )
        )
    }
    | Exp10 greaterequal Exp9 {
        Expression::Binary(
            expression::Binary::GreaterEqual(
                Box::new(Exp10),
                Box::new(Exp9)
            )
        )
    }
    | Exp10 tildeequal Exp9 {
        Expression::Binary(
            expression::Binary::NotEqual(
                Box::new(Exp10),
                Box::new(Exp9)
            )
        )
    }
    | Exp10 equalequal Exp9 {
        Expression::Binary(
            expression::Binary::Equal(
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
            expression::Binary::LogicalAnd(
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
            expression::Binary::LogicalOr(
                Box::new(Exp12),
                Box::new(Exp11)
            )
        )
    }
    | Exp11
    ;

TableConstructor(expression::TableConstructor)
    : lbrace! FieldList rbrace! {
        let mut table = expression::TableConstructor::new();
        for field in FieldList.into_iter() {
            table.insert(field);
        }
        table
    }
    ;


// one or more separated Fields
FieldList1(Vec<expression::TableConstructorField>)
    : FieldList1 FieldSep Field {
        FieldList1.push(Field);
        FieldList1
    }
    | Field {
        vec![Field]
    }
    ;

// zero or more separated Fields, with optional trailing separator
FieldList(Vec<expression::TableConstructorField>)
    : FieldList1 FieldSep? {
        FieldList1
    }
    | {
        vec![]
    }
    ;

Field(expression::TableConstructorField)
    : lbracket! k=Exp rbracket! equal! v=Exp {
        expression::TableConstructorField::KeyValue(k, v)
    }
    | ident equal! Exp {
        let name = if let TokenType::Ident(s) = ident.token_type {
            s
        } else {
            unreachable!( "k" );
        };
        expression::TableConstructorField::NameValue(name, Exp)
    }
    | Exp {
        expression::TableConstructorField::Value(Exp)
    }
    ;

FieldSep: comma | semicolon ;


FunctionDef(expression::FunctionBody)
    : function_! FuncBody
    ;

FuncBody(expression::FunctionBody)
    : lparen! ParList? rparen! Block end_! {
        expression::FunctionBody::new(ParList, Block)
    }
    ;

// dot chained ident
FuncName1(Vec<String>)
    : FuncName1 dot ident {
        if let TokenType::Ident(s) = ident.token_type {
            FuncName1.push( s );
            FuncName1
        } else {
            unreachable!( "l" );
        }
    }
    | ident {
        if let TokenType::Ident(s) = ident.token_type {
            vec![s]
        } else {
            unreachable!( "l" );
        }
    }
    ;

FuncName(statement::FunctionName)
    : FuncName1 colon! ident {
        if let TokenType::Ident(s) = ident.token_type {
            statement::FunctionName::new( FuncName1, Some(s) )
        } else {
            unreachable!( "m" );
        }
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