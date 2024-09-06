use crate::Tokenizer;

mod test {
    use super::*;
    use crate::IntOrFloat;
    use crate::TokenType;

    #[test]
    fn ignore_whitespace1() {
        let string = " \t\n\r\n aa";
        let mut tokenizer = Tokenizer::new(string.as_bytes());
        tokenizer.ignore_whitespace();
        assert_eq!(tokenizer.byte_offset, 6);
        assert_eq!(tokenizer.tokens.len(), 0);
    }
    #[test]
    fn ignore_whitespace2() {
        let string = "aa ";
        let mut tokenizer = Tokenizer::new(string.as_bytes());
        tokenizer.ignore_whitespace();
        assert_eq!(tokenizer.byte_offset, 0);
        assert_eq!(tokenizer.tokens.len(), 0);
    }

    #[test]
    fn ident1() {
        let string = "_abc123*";
        let mut tokenizer = Tokenizer::new(string.as_bytes());
        tokenizer.tokenize_ident();
        assert_eq!(tokenizer.byte_offset, 7);
        assert_eq!(tokenizer.tokens.len(), 1);
        let token = tokenizer.tokens.into_iter().next().unwrap();
        assert_eq!(token.byte_start, 0);
        assert_eq!(token.byte_end, 7);
        if let TokenType::Ident(ident) = token.token_type {
            assert_eq!(ident, "_abc123");
        } else {
            panic!("Expected Ident");
        }
    }

    #[test]
    fn ident2() {
        let string = "abc_123*";
        let mut tokenizer = Tokenizer::new(string.as_bytes());
        tokenizer.tokenize_ident();
        assert_eq!(tokenizer.byte_offset, 7);
        assert_eq!(tokenizer.tokens.len(), 1);
        let token = tokenizer.tokens.into_iter().next().unwrap();
        assert_eq!(token.byte_start, 0);
        assert_eq!(token.byte_end, 7);
        if let TokenType::Ident(ident) = token.token_type {
            assert_eq!(ident, "abc_123");
        } else {
            panic!("Expected Ident");
        }
    }
    #[test]
    fn ident3() {
        let string = "123abc*";
        let mut tokenizer = Tokenizer::new(string.as_bytes());
        tokenizer.tokenize_ident();
        assert_eq!(tokenizer.byte_offset, 0);
        assert_eq!(tokenizer.tokens.len(), 0);
    }

    #[test]
    fn line_comment() {}

    #[test]
    fn short_string1() {
        let string = r#""abcd"xxx"#;
        let mut tokenizer = Tokenizer::new(string.as_bytes());
        if let Ok(ret) = tokenizer.tokenize_string() {
            assert_eq!(ret, true);
            assert_eq!(tokenizer.tokens.len(), 1);
            assert_eq!(tokenizer.byte_offset, 6);

            let token = tokenizer.tokens.into_iter().next().unwrap();
            assert_eq!(token.byte_start, 0);
            assert_eq!(token.byte_end, 6);
            if let TokenType::String(s) = token.token_type {
                assert_eq!(s, "abcd");
            } else {
                panic!("Expected String Literal");
            }
        } else {
            panic!("Expected Ok");
        }
    }
    #[test]
    fn short_string2() {
        let string = r#""a\z 
         b"xxx"#;
        let mut tokenizer = Tokenizer::new(string.as_bytes());
        if let Ok(ret) = tokenizer.tokenize_string() {
            assert_eq!(ret, true);
            assert_eq!(tokenizer.tokens.len(), 1);

            let token = tokenizer.tokens.into_iter().next().unwrap();
            assert_eq!(token.byte_start, 0);
            if let TokenType::String(s) = token.token_type {
                assert_eq!(s, "ab");
            } else {
                panic!("Expected StringLiteral");
            }
        } else {
            panic!("Expected Ok");
        }
    }

    /// normal integer
    #[test]
    fn integer1() {
        let string = "12345abc";
        let mut tokenizer = Tokenizer::new(string.as_bytes());
        if let Ok(ret) = tokenizer.tokenize_numeric() {
            assert_eq!(ret, true);
            assert_eq!(tokenizer.tokens.len(), 1);
            let token = tokenizer.tokens.into_iter().next().unwrap();
            assert_eq!(token.byte_start, 0);
            assert_eq!(token.byte_end, 5);

            if let TokenType::Numeric(i) = token.token_type {
                assert_eq!(i, IntOrFloat::Int(12345));
            } else {
                panic!("Expected Integer");
            }
        } else {
            panic!("Expected Ok");
        }
    }

    /// hex integer
    #[test]
    fn integer2() {
        let string = "0x12345abcgg";
        let mut tokenizer = Tokenizer::new(string.as_bytes());
        if let Ok(ret) = tokenizer.tokenize_numeric() {
            assert_eq!(ret, true);
            assert_eq!(tokenizer.tokens.len(), 1);
            let token = tokenizer.tokens.into_iter().next().unwrap();
            assert_eq!(token.byte_start, 0);
            assert_eq!(token.byte_end, 10);

            if let TokenType::Numeric(i) = token.token_type {
                assert_eq!(i, IntOrFloat::Int(0x12345abc));
            } else {
                panic!("Expected Integer");
            }
        } else {
            panic!("Expected Ok");
        }
    }
    #[test]
    fn float1() {
        let string = "123.456abc";
        let mut tokenizer = Tokenizer::new(string.as_bytes());
        if let Ok(ret) = tokenizer.tokenize_numeric() {
            assert_eq!(ret, true);
            assert_eq!(tokenizer.tokens.len(), 1);
            let token = tokenizer.tokens.into_iter().next().unwrap();
            assert_eq!(token.byte_start, 0);
            assert_eq!(token.byte_end, 7);

            if let TokenType::Numeric(IntOrFloat::Float(f)) = token.token_type {
                let abs = (f - 123.456).abs();
                assert!(abs < 0.00001);
            } else {
                panic!("Expected Integer");
            }
        } else {
            panic!("Expected Ok");
        }
    }

    #[test]
    fn float2() {
        let string = "123.456e+2abc";
        let mut tokenizer = Tokenizer::new(string.as_bytes());
        if let Ok(ret) = tokenizer.tokenize_numeric() {
            assert_eq!(ret, true);
            assert_eq!(tokenizer.tokens.len(), 1);
            let token = tokenizer.tokens.into_iter().next().unwrap();
            assert_eq!(token.byte_start, 0);
            assert_eq!(token.byte_end, 10);

            if let TokenType::Numeric(IntOrFloat::Float(f)) = token.token_type {
                let abs = (f - 12345.6).abs();
                assert!(abs < 0.00001);
            } else {
                panic!("Expected Integer");
            }
        } else {
            panic!("Expected Ok");
        }
    }
    #[test]
    fn float3() {
        let string = "123.456e-2abc";
        let mut tokenizer = Tokenizer::new(string.as_bytes());
        if let Ok(ret) = tokenizer.tokenize_numeric() {
            assert_eq!(ret, true);
            assert_eq!(tokenizer.tokens.len(), 1);
            let token = tokenizer.tokens.into_iter().next().unwrap();
            assert_eq!(token.byte_start, 0);
            assert_eq!(token.byte_end, 10);

            if let TokenType::Numeric(IntOrFloat::Float(f)) = token.token_type {
                let abs = (f - 1.23456).abs();
                assert!(abs < 0.00001);
            } else {
                panic!("Expected Integer");
            }
        } else {
            panic!("Expected Ok");
        }
    }
}
