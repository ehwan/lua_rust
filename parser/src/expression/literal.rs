use crate::IntOrFloat;
use crate::Span;
use crate::Token;
use lua_tokenizer::TokenType;

#[derive(Clone, Copy, Debug)]
pub struct ExprNil {
    pub span: Span,
}
impl ExprNil {
    pub fn new(span: Span) -> Self {
        Self { span }
    }
    /// get the span of the nil expression
    pub fn span(&self) -> Span {
        self.span
    }
}
impl From<Token> for ExprNil {
    fn from(t: Token) -> Self {
        Self::new(t.span)
    }
}

/// lua numeric literal value
#[derive(Clone, Copy, Debug)]
pub struct ExprNumeric {
    pub value: IntOrFloat,
    pub span: Span,
}
impl ExprNumeric {
    pub fn new(value: IntOrFloat, span: Span) -> Self {
        Self { value, span }
    }
    /// get the span of the numeric literal
    pub fn span(&self) -> Span {
        self.span
    }
}
impl From<Token> for ExprNumeric {
    fn from(t: Token) -> Self {
        match t.token_type {
            TokenType::Numeric(value) => Self::new(value.into(), t.span),
            _ => unreachable!(),
        }
    }
}

/// lua string literal value
#[derive(Clone, Debug)]
pub struct ExprString {
    pub value: Vec<u8>,
    pub span: Span,
}
impl ExprString {
    pub fn new(value: Vec<u8>, span: Span) -> Self {
        Self { value, span }
    }
    /// get the span of the string literal
    pub fn span(&self) -> Span {
        self.span
    }
}
impl From<Token> for ExprString {
    fn from(t: Token) -> Self {
        match t.token_type {
            TokenType::String(s) => Self::new(s, t.span),
            TokenType::Ident(s) => Self::new(s.into_bytes(), t.span),
            _ => unreachable!(),
        }
    }
}

/// lua boolean literal value
#[derive(Clone, Copy, Debug)]
pub struct ExprBool {
    pub value: bool,
    pub span: Span,
}
impl ExprBool {
    pub fn new(value: bool, span: Span) -> Self {
        Self { value, span }
    }
    /// get the span of the boolean literal
    pub fn span(&self) -> Span {
        self.span
    }
}
impl From<Token> for ExprBool {
    fn from(t: Token) -> Self {
        match t.token_type {
            TokenType::Bool(value) => Self::new(value, t.span),
            _ => unreachable!(),
        }
    }
}

/// `...`
#[derive(Clone, Copy, Debug)]
pub struct ExprVariadic {
    pub span: Span,
}
impl ExprVariadic {
    pub fn new(span: Span) -> Self {
        Self { span }
    }
    /// get the span of the variadic expression
    pub fn span(&self) -> Span {
        self.span
    }
}
impl From<Token> for ExprVariadic {
    fn from(t: Token) -> Self {
        Self { span: t.span }
    }
}
