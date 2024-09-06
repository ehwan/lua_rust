mod error;
mod iorf;
mod span;
mod token;
mod tokentype;

#[cfg(test)]
mod test;

pub use error::TokenizeError;
pub use iorf::IntOrFloat;
pub use span::Span;
pub use token::Token;
pub use tokentype::TokenType;

/// type alias for lua integer type.
#[cfg(not(feature = "32bit"))]
pub type IntType = i64;
/// type alias for lua float type.
#[cfg(not(feature = "32bit"))]
pub type FloatType = f64;

/// type alias for lua integer type.
#[cfg(feature = "32bit")]
pub type IntType = i32;
/// type alias for lua float type.
#[cfg(feature = "32bit")]
pub type FloatType = f32;

use core::str;
use std::collections::HashMap;

/// lazy tokenize iterator.
#[derive(Clone)]
pub struct Tokenizer<'a> {
    /// source code to tokenize
    pub(crate) source: &'a [u8],
    /// current byte offset in source
    pub(crate) byte_offset: usize,

    pub(crate) keyword_map: HashMap<&'static str, TokenType>,
}

impl<'a> Tokenizer<'a> {
    /// create new tokenizer iterator from source code.
    pub fn new(source: &'a str) -> Self {
        let mut keyword_map = HashMap::with_capacity(25);
        keyword_map.insert("and", TokenType::And);
        keyword_map.insert("break", TokenType::Break);
        keyword_map.insert("do", TokenType::Do);
        keyword_map.insert("else", TokenType::Else);
        keyword_map.insert("elseif", TokenType::Elseif);
        keyword_map.insert("end", TokenType::End);
        keyword_map.insert("false", TokenType::Bool(false));
        keyword_map.insert("for", TokenType::For);
        keyword_map.insert("function", TokenType::Function);
        keyword_map.insert("goto", TokenType::Goto);
        keyword_map.insert("if", TokenType::If);
        keyword_map.insert("in", TokenType::In);
        keyword_map.insert("local", TokenType::Local);
        keyword_map.insert("nil", TokenType::Nil);
        keyword_map.insert("not", TokenType::Not);
        keyword_map.insert("or", TokenType::Or);
        keyword_map.insert("repeat", TokenType::Repeat);
        keyword_map.insert("return", TokenType::Return);
        keyword_map.insert("then", TokenType::Then);
        keyword_map.insert("true", TokenType::Bool(true));
        keyword_map.insert("until", TokenType::Until);
        keyword_map.insert("while", TokenType::While);

        Self {
            source: source.as_bytes(),
            byte_offset: 0,
            keyword_map,
        }
    }
    fn get_cursor(&self) -> usize {
        self.byte_offset
    }
    fn set_cursor(&mut self, cursor: usize) {
        self.byte_offset = cursor;
    }

    fn advance(&mut self) {
        self.byte_offset += 1;
    }
    fn advance_n(&mut self, bytes: usize) {
        self.byte_offset += bytes;
    }

    fn peek(&self) -> Option<u8> {
        self.source.get(self.byte_offset).copied()
    }
    pub(crate) fn ignore_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            match ch {
                b' ' | b'\t' | b'\r' | b'\n' => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    /// parse identifier.
    /// returns `Some` if identifier is successfully parsed.
    pub(crate) fn tokenize_ident(&mut self) -> Option<Token> {
        let i0 = self.byte_offset;
        if let Some(ch) = self.peek() {
            match ch {
                b'_' | b'a'..=b'z' | b'A'..=b'Z' => {
                    self.advance();
                    while let Some(ch) = self.peek() {
                        match ch {
                            b'_' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => {
                                self.advance();
                            }
                            _ => break,
                        }
                    }

                    // checks for keyword
                    let i1 = self.byte_offset;
                    let slice = &self.source[i0..i1];
                    let s = unsafe { str::from_utf8_unchecked(slice) };
                    if let Some(keyword) = self.keyword_map.get(s) {
                        let token = Token {
                            token_type: keyword.clone(),
                            span: Span::new(i0, i1),
                        };
                        Some(token)
                    } else {
                        let token = Token {
                            token_type: TokenType::Ident(s.to_string()),
                            span: Span::new(i0, i1),
                        };
                        Some(token)
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }
    /// parse literal.
    /// returns error if it is definitely literal but it contains invalid characters.
    /// otherwise, Ok(true) if it is literal, Ok(false) if it is not literal.
    pub(crate) fn tokenize_literal(&mut self) -> Result<Option<Token>, TokenizeError> {
        if let Some(token) = self.tokenize_numeric()? {
            Ok(Some(token))
        } else if let Some(token) = self.tokenize_string()? {
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    /// parse single hex
    pub(crate) fn hex(ch: u8) -> Option<u32> {
        match ch {
            b'0'..=b'9' => Some((ch - b'0') as u32),
            b'a'..=b'f' => Some((ch - b'a') as u32 + 10),
            b'A'..=b'F' => Some((ch - b'A') as u32 + 10),
            _ => None,
        }
    }

    pub(crate) fn tokenize_numeric(&mut self) -> Result<Option<Token>, TokenizeError> {
        let i0 = self.byte_offset;
        // check if it is hex
        if self.starts_with_and_advance(b"0x") || self.starts_with_and_advance(b"0X") {
            // hex

            // one or more hexs
            let mut value = IntOrFloat::from(0);
            let mut count = 0;
            while let Some(ch) = self.peek() {
                if let Some(hex) = Self::hex(ch) {
                    self.advance();
                    count += 1;
                    value *= 16 as IntType;
                    value += hex as IntType;
                } else {
                    break;
                }
            }
            if count == 0 {
                return Err(TokenizeError::NumericEmpty {
                    start: i0,
                    pos: self.byte_offset,
                });
            }

            // check fraction
            // dot
            if self.peek() == Some(b'.') {
                self.advance();

                value = value.to_float().into();

                // one or more hexs for fraction
                let base = (1.0 / 16.0) as FloatType;
                let mut exp = base;
                count = 0;
                while let Some(ch) = self.peek() {
                    if let Some(hex) = Self::hex(ch) {
                        self.advance();
                        count += 1;

                        let f = hex as FloatType * exp;
                        value += f;
                        exp *= base;
                    } else {
                        break;
                    }
                }

                if count == 0 {
                    return Err(TokenizeError::NumericEmpty {
                        start: i0,
                        pos: self.byte_offset,
                    });
                }
            }

            // check exponent
            // p or P
            if self.peek() == Some(b'p') || self.peek() == Some(b'P') {
                self.advance();

                // '+' or '-'
                let is_neg = match self.peek() {
                    Some(b'+') => {
                        self.advance();
                        false
                    }
                    Some(b'-') => {
                        self.advance();
                        true
                    }
                    _ => false,
                };

                // one or more digits for exponent
                count = 0;
                let mut binary_exp: u32 = 0;
                while let Some(ch) = self.peek() {
                    if ch >= b'0' && ch <= b'9' {
                        self.advance();
                        count += 1;
                        let d = (ch - b'0') as u32;
                        binary_exp = binary_exp.wrapping_mul(10).wrapping_add(d);
                    } else {
                        break;
                    }
                }
                if count == 0 {
                    return Err(TokenizeError::NumericEmpty {
                        start: i0,
                        pos: self.byte_offset,
                    });
                }

                if is_neg {
                    for _ in 0..binary_exp {
                        value *= 0.5 as FloatType;
                    }
                } else {
                    for _ in 0..binary_exp {
                        value *= 2 as IntType;
                    }
                }
            }

            let token = Token {
                token_type: TokenType::Numeric(value),
                span: Span::new(i0, self.byte_offset),
            };
            Ok(Some(token))
        } else {
            // decimals
            if let Some(b'0'..=b'9') = self.peek() {
            } else {
                return Ok(None);
            }

            // one or more digits
            let mut value = IntOrFloat::from(0);
            let mut count = 0;
            while let Some(ch) = self.peek() {
                if ch >= b'0' && ch <= b'9' {
                    self.advance();
                    count += 1;
                    value *= 10 as IntType;
                    value += (ch - b'0') as IntType;
                } else {
                    break;
                }
            }
            if count == 0 {
                return Err(TokenizeError::NumericEmpty {
                    start: i0,
                    pos: self.byte_offset,
                });
            }

            // check fraction
            // dot
            if self.peek() == Some(b'.') {
                self.advance();

                value = value.to_float().into();

                // one or more hexs for fraction
                let base = (1.0 / 10.0) as FloatType;
                let mut exp = base;
                count = 0;
                while let Some(ch) = self.peek() {
                    if ch >= b'0' && ch <= b'9' {
                        self.advance();
                        count += 1;

                        let f = (ch - b'0') as FloatType * exp;
                        value += f;
                        exp *= base;
                    } else {
                        break;
                    }
                }

                if count == 0 {
                    return Err(TokenizeError::NumericEmpty {
                        start: i0,
                        pos: self.byte_offset,
                    });
                }
            }

            // check exponent
            // e or E
            if self.peek() == Some(b'e') || self.peek() == Some(b'E') {
                self.advance();

                // '+' or '-'
                let is_neg = match self.peek() {
                    Some(b'+') => {
                        self.advance();
                        false
                    }
                    Some(b'-') => {
                        self.advance();
                        true
                    }
                    _ => false,
                };

                // one or more digits for exponent
                count = 0;
                let mut base10_exp: u32 = 0;
                while let Some(ch) = self.peek() {
                    if ch >= b'0' && ch <= b'9' {
                        self.advance();
                        count += 1;
                        let d = (ch - b'0') as u32;
                        base10_exp = base10_exp.wrapping_mul(10).wrapping_add(d);
                    } else {
                        break;
                    }
                }
                if count == 0 {
                    return Err(TokenizeError::NumericEmpty {
                        start: i0,
                        pos: self.byte_offset,
                    });
                }

                if is_neg {
                    for _ in 0..base10_exp {
                        value *= 0.1 as FloatType;
                    }
                } else {
                    for _ in 0..base10_exp {
                        value *= 10 as IntType;
                    }
                }
            }

            let token = Token {
                token_type: TokenType::Numeric(value),
                span: Span::new(i0, self.byte_offset),
            };
            Ok(Some(token))
        }
    }
    pub(crate) fn short_string_literal(
        &mut self,
        delim: u8,
        start: usize,
    ) -> Result<String, TokenizeError> {
        let mut s = Vec::<u8>::new();
        while let Some(ch) = self.peek() {
            if ch == delim {
                self.advance();
                match String::from_utf8(s) {
                    Ok(s) => return Ok(s),
                    Err(e) => {
                        return Err(TokenizeError::InvalidUtf8 {
                            start,
                            end: self.byte_offset,
                            error: e,
                        });
                    }
                }
            }
            match ch {
                b'\\' => {
                    let escape_start = self.byte_offset;
                    // escape
                    // consume '\\'
                    self.advance();
                    match self.peek() {
                        Some(b'z') => {
                            self.advance();
                            self.ignore_whitespace();
                        }
                        Some(b'a') => {
                            s.push(b'\x07');
                            self.advance();
                        }
                        Some(b'b') => {
                            s.push(b'\x08');
                            self.advance();
                        }
                        Some(b'f') => {
                            s.push(b'\x0c');
                            self.advance();
                        }
                        Some(b'n') | Some(b'\n') => {
                            s.push(b'\n');
                            self.advance();
                        }
                        Some(b'r') => {
                            s.push(b'\r');
                            self.advance();
                        }
                        Some(b't') => {
                            s.push(b'\t');
                            self.advance();
                        }
                        Some(b'v') => {
                            s.push(b'\x0b');
                            self.advance();
                        }
                        Some(b'\\') => {
                            s.push(b'\\');
                            self.advance();
                        }
                        Some(b'\"') => {
                            s.push(b'\"');
                            self.advance();
                        }
                        Some(b'\'') => {
                            s.push(b'\'');
                            self.advance();
                        }
                        Some(b'x') => {
                            // two hex digits
                            self.advance();

                            if let Some(first) = self.peek() {
                                if let Some(first) = Self::hex(first) {
                                    self.advance();
                                    if let Some(second) = self.peek() {
                                        if let Some(second) = Self::hex(second) {
                                            s.push((first * 16u32 + second) as u8);
                                            self.advance();
                                        } else {
                                            // not hex
                                            return Err(TokenizeError::ShortStringNotHex {
                                                start,
                                                pos: self.byte_offset,
                                            });
                                        }
                                    } else {
                                        // not closed
                                        return Err(TokenizeError::ShortStringNotClosed {
                                            delim: delim as char,
                                            start,
                                            end: self.byte_offset,
                                        });
                                    }
                                } else {
                                    // not hex
                                    return Err(TokenizeError::ShortStringNotHex {
                                        start,
                                        pos: self.byte_offset,
                                    });
                                }
                            } else {
                                // not closed
                                return Err(TokenizeError::ShortStringNotClosed {
                                    delim: delim as char,
                                    start,
                                    end: self.byte_offset,
                                });
                            }
                        }
                        Some(b'0'..=b'9') => {
                            // three decimal digits
                            let first: u32 = (self.peek().unwrap() - b'0') as u32;
                            self.advance();

                            if let Some(second) = self.peek() {
                                if second >= b'0' && second <= b'9' {
                                    let second: u32 = (second - b'0') as u32;
                                    self.advance();
                                    if let Some(third) = self.peek() {
                                        if third >= b'0' && third <= b'9' {
                                            let third: u32 = (third - b'0') as u32;
                                            self.advance();
                                            s.push((first * 100 + second * 10 + third) as u8);
                                        } else {
                                            // not decimal
                                            return Err(TokenizeError::ShortStringNotDecimal {
                                                start,
                                                pos: self.byte_offset,
                                            });
                                        }
                                    } else {
                                        // not closed
                                        return Err(TokenizeError::ShortStringNotClosed {
                                            delim: delim as char,
                                            start,
                                            end: self.byte_offset,
                                        });
                                    }
                                } else {
                                    // not decimal
                                    return Err(TokenizeError::ShortStringNotDecimal {
                                        start,
                                        pos: self.byte_offset,
                                    });
                                }
                            } else {
                                // not closed
                                return Err(TokenizeError::ShortStringNotClosed {
                                    delim: delim as char,
                                    start,
                                    end: self.byte_offset,
                                });
                            }
                        }
                        Some(b'u') => {
                            self.advance();
                            // \u{X+}

                            if let Some(open) = self.peek() {
                                if open == b'{' {
                                    self.advance();

                                    let mut codepoint = 0i32;
                                    let mut closed = false;
                                    let mut count = 0;
                                    while let Some(ch) = self.peek() {
                                        if ch == b'}' {
                                            closed = true;
                                            self.advance();
                                            break;
                                        }
                                        if let Some(digit) = Self::hex(ch) {
                                            count += 1;
                                            if let Some(mul) = codepoint.checked_mul(16i32) {
                                                codepoint = mul;
                                            } else {
                                                return Err(TokenizeError::ShortStringOverflow {
                                                    start,
                                                    pos: self.byte_offset,
                                                });
                                            }
                                            if let Some(add) = codepoint.checked_add(digit as i32) {
                                                codepoint = add;
                                            } else {
                                                return Err(TokenizeError::ShortStringOverflow {
                                                    start,
                                                    pos: self.byte_offset,
                                                });
                                            }
                                            self.advance();
                                        } else {
                                            // not hex
                                            return Err(TokenizeError::ShortStringNotHex {
                                                start,
                                                pos: self.byte_offset,
                                            });
                                        }
                                    }

                                    if !closed {
                                        // not closed
                                        return Err(TokenizeError::ShortStringNotClosed {
                                            delim: delim as char,
                                            start,
                                            end: self.byte_offset,
                                        });
                                    }
                                    if count == 0 {
                                        // empty codepoint
                                        return Err(TokenizeError::ShortStringEmptyCodepoint {
                                            start,
                                            escape_start,
                                            escape_end: self.byte_offset,
                                        });
                                    }

                                    let codepoint: char =
                                        std::char::from_u32(codepoint as u32).unwrap();
                                    let mut buffer = [0u8; 4];
                                    let len = codepoint.len_utf8();
                                    codepoint.encode_utf8(&mut buffer);
                                    for i in 0..len {
                                        s.push(buffer[i]);
                                    }
                                } else {
                                    // '{' not present
                                    return Err(TokenizeError::ShortStringNoOpenBrace {
                                        start,
                                        pos: self.byte_offset,
                                    });
                                }
                            } else {
                                // not closed
                                return Err(TokenizeError::ShortStringNotClosed {
                                    delim: delim as char,
                                    start,
                                    end: self.byte_offset,
                                });
                            }
                        }

                        Some(other) => {
                            return Err(TokenizeError::ShortStringInvalidEscape {
                                start,
                                pos: self.byte_offset,
                                escape: other as char,
                            });
                        }
                        None => {
                            return Err(TokenizeError::ShortStringNotClosed {
                                delim: delim as char,
                                start,
                                end: self.byte_offset,
                            });
                        }
                    }
                }
                b'\n' => {
                    return Err(TokenizeError::ShortStringNewline {
                        start,
                        pos: self.byte_offset,
                    });
                }
                _ => {
                    s.push(ch);
                    self.advance();
                }
            }
        }
        // not closed
        Err(TokenizeError::ShortStringNotClosed {
            delim: delim as char,
            start,
            end: self.byte_offset,
        })
    }
    pub(crate) fn long_string_literal(
        &mut self,
        equal_count: usize,
        start: usize,
    ) -> Result<String, TokenizeError> {
        let mut s = Vec::<u8>::new();
        while let Some(ch) = self.peek() {
            match ch {
                b']' => {
                    // check end of long string literal
                    let cursor0 = self.get_cursor();
                    if let Some(count) = self.long_bracket(b']') {
                        if count == equal_count {
                            match String::from_utf8(s) {
                                Ok(s) => return Ok(s),
                                Err(e) => {
                                    return Err(TokenizeError::InvalidUtf8 {
                                        start: cursor0,
                                        end: self.byte_offset,
                                        error: e,
                                    });
                                }
                            }
                        } else {
                            self.set_cursor(cursor0);
                            self.advance();
                            s.push(ch);
                        }
                    } else {
                        self.advance();
                        s.push(ch);
                    }
                }

                _ => {
                    s.push(ch);
                    self.advance();
                }
            }
        }
        // not closed
        Err(TokenizeError::LongStringNotClosed {
            start,
            end: self.byte_offset,
            equal_count,
        })
    }
    pub(crate) fn tokenize_string(&mut self) -> Result<Option<Token>, TokenizeError> {
        match self.peek() {
            Some(b'\'') | Some(b'"') => {
                // since ' or " is consumed, it is definitely short string literal.
                let i0 = self.get_cursor();
                let quote = self.peek().unwrap();
                self.advance();

                let s = self.short_string_literal(quote, i0)?;

                let token = Token {
                    token_type: TokenType::String(s),
                    span: Span::new(i0, self.byte_offset),
                };
                Ok(Some(token))
            }
            Some(b'[') => {
                // long string literal
                let i0 = self.get_cursor();
                if let Some(open_count) = self.long_bracket(b'[') {
                    // since long bracket '[[' is consumed, it is definitely long string literal.
                    let s = self.long_string_literal(open_count, i0)?;

                    let token = Token {
                        token_type: TokenType::String(s),
                        span: Span::new(i0, self.byte_offset),
                    };
                    Ok(Some(token))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    /// consume long bracket and return the number of '='.
    /// `bracket` must be either b'[' or b']'.
    pub(crate) fn long_bracket(&mut self, bracket: u8) -> Option<usize> {
        assert!(bracket == b'[' || bracket == b']');
        let cursor0 = self.get_cursor();
        if self.peek() == Some(bracket) {
            // consume '['
            self.advance();

            // the number of '='
            let mut count = 0;
            while let Some(ch) = self.peek() {
                if ch == bracket {
                    // consume '['
                    self.advance();
                    break;
                } else if ch == b'=' {
                    // consume '='
                    self.advance();
                    count += 1;
                } else {
                    self.set_cursor(cursor0);
                    return None;
                }
            }
            Some(count)
        } else {
            return None;
        }
    }
    pub(crate) fn starts_with_and_advance(&mut self, prefix: &[u8]) -> bool {
        let slice = &self.source[self.byte_offset..];
        if slice.starts_with(prefix) {
            self.advance_n(prefix.len());
            true
        } else {
            false
        }
    }

    /// try tokenize single token.
    pub(crate) fn try_tokenize(&mut self) -> Result<Option<Token>, TokenizeError> {
        self.ignore_whitespace();
        // check eof
        if self.byte_offset >= self.source.len() {
            return Ok(None);
        }

        if let Some(token) = self.tokenize_ident() {
            // try ident
            Ok(Some(token))
        } else if let Some(token) = self.tokenize_literal()? {
            // try literal
            Ok(Some(token))
        } else {
            // try punctuator

            macro_rules! advance_and_return {
                ($token_type:ident) => {{
                    self.advance();
                    Ok(Some(Token {
                        token_type: TokenType::$token_type,
                        span: Span::new(self.byte_offset - 1, self.byte_offset),
                    }))
                }};
            }

            let ch = self.peek().unwrap();
            match ch {
                b'+' => {
                    advance_and_return!(Plus)
                }
                b'*' => {
                    advance_and_return!(Asterisk)
                }
                b'/' => {
                    // check for SlashSlash
                    let i0 = self.byte_offset;
                    self.advance();

                    if self.peek() == Some(b'/') {
                        self.advance();
                        Ok(Some(Token {
                            token_type: TokenType::SlashSlash,
                            span: Span::new(i0, self.byte_offset),
                        }))
                    } else {
                        Ok(Some(Token {
                            token_type: TokenType::Slash,
                            span: Span::new(i0, i0 + 1),
                        }))
                    }
                }
                b'%' => {
                    advance_and_return!(Percent)
                }
                b'^' => {
                    advance_and_return!(Caret)
                }
                b'#' => {
                    advance_and_return!(Hash)
                }
                b'&' => {
                    advance_and_return!(Ampersand)
                }
                b'~' => {
                    // check for TildeEqual
                    let i0 = self.byte_offset;
                    self.advance();

                    if self.peek() == Some(b'=') {
                        self.advance();
                        Ok(Some(Token {
                            token_type: TokenType::TildeEqual,
                            span: Span::new(i0, self.byte_offset),
                        }))
                    } else {
                        Ok(Some(Token {
                            token_type: TokenType::Tilde,
                            span: Span::new(i0, i0 + 1),
                        }))
                    }
                }
                b'|' => {
                    advance_and_return!(Pipe)
                }
                b'<' => {
                    // check for LessLess
                    let i0 = self.byte_offset;
                    self.advance();

                    match self.peek() {
                        Some(b'<') => {
                            self.advance();
                            Ok(Some(Token {
                                token_type: TokenType::LessLess,
                                span: Span::new(i0, self.byte_offset),
                            }))
                        }
                        Some(b'=') => {
                            self.advance();
                            Ok(Some(Token {
                                token_type: TokenType::LessEqual,
                                span: Span::new(i0, self.byte_offset),
                            }))
                        }

                        _ => Ok(Some(Token {
                            token_type: TokenType::Less,
                            span: Span::new(i0, i0 + 1),
                        })),
                    }
                }
                b'>' => {
                    // check for LessLess
                    let i0 = self.byte_offset;
                    self.advance();

                    match self.peek() {
                        Some(b'>') => {
                            self.advance();
                            Ok(Some(Token {
                                token_type: TokenType::GreaterGreater,
                                span: Span::new(i0, self.byte_offset),
                            }))
                        }
                        Some(b'=') => {
                            self.advance();
                            Ok(Some(Token {
                                token_type: TokenType::GreaterEqual,
                                span: Span::new(i0, self.byte_offset),
                            }))
                        }

                        _ => Ok(Some(Token {
                            token_type: TokenType::Greater,
                            span: Span::new(i0, i0 + 1),
                        })),
                    }
                }
                b'=' => {
                    // check for EqualEqual
                    let i0 = self.byte_offset;
                    self.advance();

                    if self.peek() == Some(b'=') {
                        self.advance();
                        Ok(Some(Token {
                            token_type: TokenType::EqualEqual,
                            span: Span::new(i0, self.byte_offset),
                        }))
                    } else {
                        Ok(Some(Token {
                            token_type: TokenType::Equal,
                            span: Span::new(i0, i0 + 1),
                        }))
                    }
                }

                b'(' => {
                    advance_and_return!(LParen)
                }
                b')' => {
                    advance_and_return!(RParen)
                }
                b'{' => {
                    advance_and_return!(LBrace)
                }
                b'}' => {
                    advance_and_return!(RBrace)
                }
                b'[' => {
                    advance_and_return!(LBracket)
                }
                b']' => {
                    advance_and_return!(RBracket)
                }
                b':' => {
                    // check for ColonColon
                    let i0 = self.byte_offset;
                    self.advance();

                    if self.peek() == Some(b':') {
                        self.advance();
                        Ok(Some(Token {
                            token_type: TokenType::ColonColon,
                            span: Span::new(i0, self.byte_offset),
                        }))
                    } else {
                        Ok(Some(Token {
                            token_type: TokenType::Colon,
                            span: Span::new(i0, i0 + 1),
                        }))
                    }
                }
                b';' => {
                    advance_and_return!(Semicolon)
                }
                b',' => {
                    advance_and_return!(Comma)
                }
                b'.' => {
                    let i0 = self.byte_offset;
                    self.advance();

                    if self.peek() == Some(b'.') {
                        let i1 = self.byte_offset;
                        self.advance();

                        if self.peek() == Some(b'.') {
                            self.advance();
                            Ok(Some(Token {
                                token_type: TokenType::DotDotDot,
                                span: Span::new(i0, self.byte_offset),
                            }))
                        } else {
                            Ok(Some(Token {
                                token_type: TokenType::DotDot,
                                span: Span::new(i0, i1),
                            }))
                        }
                    } else {
                        Ok(Some(Token {
                            token_type: TokenType::Dot,
                            span: Span::new(i0, i0 + 1),
                        }))
                    }
                }
                b'-' => {
                    let i0 = self.byte_offset;
                    // check start of comment
                    if self.starts_with_and_advance(b"--") {
                        // check start of multi-line comment
                        if let Some(open_equal_count) = self.long_bracket(b'[') {
                            let multiline_comment_begin = (i0, self.byte_offset);

                            while self.byte_offset < self.source.len() {
                                if let Some(close_equal_count) = self.long_bracket(b']') {
                                    if close_equal_count == open_equal_count {
                                        if self.starts_with_and_advance(b"--") {
                                            return self.try_tokenize();
                                        }
                                    }
                                    // since `long_bracket` is parsed, the cursor is currently at the next position of ']'.
                                    // ]====]
                                    //       ^ here
                                    // move back cursor so that it points to the last ']'.
                                    self.byte_offset -= 1;
                                } else {
                                    self.advance()
                                }
                            }
                            // eof reached
                            // multi-line comment not closed
                            Err(TokenizeError::MultilineCommentNotClosed {
                                start: multiline_comment_begin.0,
                                end: multiline_comment_begin.1,
                            })
                        } else {
                            // it is line comment
                            while let Some(ch) = self.peek() {
                                self.advance();
                                if ch == b'\n' {
                                    break;
                                }
                            }
                            self.try_tokenize()
                        }
                    } else {
                        // it is not comment, put '-'
                        advance_and_return!(Minus)
                    }
                }

                _ => {
                    // invalid punctuator
                    Err(TokenizeError::InvalidPunct {
                        pos: self.byte_offset,
                        punct: ch as char,
                    })
                }
            }
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token, TokenizeError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_tokenize() {
            Ok(Some(token)) => Some(Ok(token)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
