use crate::{LuaNumber, RuntimeError};

/// A Lua string, which can be stored on the heap, stack, or as a static reference, based on its length.
#[derive(Clone)]
pub enum LuaString {
    Heap(Vec<u8>),
    Static(&'static [u8]),
    Stack([u8; 23], u8),
}

impl LuaString {
    pub fn from_static_str(s: &'static str) -> Self {
        LuaString::Static(s.as_bytes())
    }
    pub fn from_static(s: &'static [u8]) -> Self {
        LuaString::Static(s)
    }
    pub fn from_vec(v: Vec<u8>) -> Self {
        LuaString::Heap(v)
    }
    pub fn from_string(s: String) -> Self {
        if s.len() <= 23 {
            let mut stack = [0; 23];
            for (i, c) in s.bytes().enumerate() {
                stack[i] = c;
            }
            LuaString::Stack(stack, s.len() as u8)
        } else {
            LuaString::Heap(s.into_bytes())
        }
    }
    pub fn from_slice(s: &[u8]) -> Self {
        if s.len() <= 23 {
            let mut stack = [0; 23];
            for (i, &c) in s.iter().enumerate() {
                stack[i] = c;
            }
            LuaString::Stack(stack, s.len() as u8)
        } else {
            LuaString::Heap(s.to_vec())
        }
    }
    pub fn from_str(s: &str) -> Self {
        LuaString::from_string(s.to_string())
    }

    pub fn len(&self) -> usize {
        match self {
            LuaString::Heap(v) => v.len(),
            LuaString::Static(s) => s.len(),
            LuaString::Stack(_, len) => *len as usize,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            LuaString::Heap(v) => v,
            LuaString::Static(s) => s,
            LuaString::Stack(s, len) => &s[..*len as usize],
        }
    }

    pub fn into_vec(self) -> Vec<u8> {
        match self {
            LuaString::Heap(v) => v,
            LuaString::Static(s) => s.to_vec(),
            LuaString::Stack(s, len) => s[..len as usize].to_vec(),
        }
    }

    pub fn into_mapped(self, f: impl Fn(u8) -> u8) -> Self {
        match self {
            LuaString::Heap(v) => LuaString::Heap(v.into_iter().map(f).collect()),
            LuaString::Static(s) => LuaString::Heap(s.iter().map(|&c| f(c)).collect()),
            LuaString::Stack(s, len) => {
                let mut stack = [0; 23];
                for i in 0..len as usize {
                    stack[i] = f(s[i]);
                }
                LuaString::Stack(stack, len)
            }
        }
    }

    pub fn try_to_number(&self) -> Result<LuaNumber, RuntimeError> {
        // use `lua_tokenizer` to parse the string into a number
        let mut tokenizer = lua_tokenizer::Tokenizer::from_bytes(self.as_bytes());
        tokenizer.ignore_whitespace();
        // sign
        let neg = match tokenizer.peek() {
            Some(b'-') => {
                tokenizer.next();
                true
            }
            Some(b'+') => {
                tokenizer.next();
                false
            }
            _ => false,
        };
        // number
        let tokenize_res = tokenizer.tokenize_numeric();
        match tokenize_res {
            Ok(Some(res)) => {
                tokenizer.ignore_whitespace();
                if tokenizer.is_end() {
                    match res.token_type {
                        lua_tokenizer::TokenType::Numeric(numeric) => match numeric {
                            lua_tokenizer::IntOrFloat::Int(i) => {
                                if neg {
                                    Ok((-i).into())
                                } else {
                                    Ok(i.into())
                                }
                            }
                            lua_tokenizer::IntOrFloat::Float(f) => {
                                if neg {
                                    Ok((-f).into())
                                } else {
                                    Ok(f.into())
                                }
                            }
                        },
                        _ => Err(RuntimeError::Expected("number", Some("string"))),
                    }
                } else {
                    Err(RuntimeError::Expected("number", Some("string")))
                }
            }
            Ok(None) => Err(RuntimeError::Expected("number", Some("string"))),
            Err(tokenize_error) => Err(RuntimeError::TokenizeError(tokenize_error)),
        }
    }
}

impl std::hash::Hash for LuaString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}
impl std::cmp::PartialEq for LuaString {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}
impl std::cmp::Eq for LuaString {}
impl std::cmp::PartialOrd for LuaString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl std::cmp::Ord for LuaString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl std::fmt::Display for LuaString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.as_bytes()))
    }
}
impl std::fmt::Debug for LuaString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", String::from_utf8_lossy(self.as_bytes()))
    }
}

impl std::ops::Index<usize> for LuaString {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.as_bytes()[index]
    }
}
