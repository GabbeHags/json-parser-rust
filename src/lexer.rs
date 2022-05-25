#![allow(dead_code)]

use std::iter::Peekable;

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    OpenCurly,
    CloseCurly,
    OpenBracket,
    CloseBracket,
    Comma,
    Colon,
    Integer,
    Float,
    Str,
    Null,
    True,
    False,
    Eof,
    Invalid,
}

#[derive(Debug)]
pub struct Loc {
    col: usize,
    row: usize,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub loc: Loc,
}

#[derive(Debug)]
pub struct Lexer<Chars: Iterator<Item = char>> {
    chars: Peekable<Chars>,
    exhausted: bool,
    col: usize,
    row: usize,
    char_count: usize,
}

impl<Chars: Iterator<Item = char>> Lexer<Chars> {
    pub fn new(chars: Chars) -> Self {
        Self {
            chars: chars.peekable(),
            exhausted: false,
            col: 0,
            row: 0,
            char_count: 0,
        }
    }

    fn get_loc(&self) -> Loc {
        Loc {
            col: self.col - self.char_count + 1,
            row: self.row + 1,
        }
    }

    fn next_token(&mut self) -> Token {
        self.trim();

        if let Some(c) = self.chars.peek() {
            self.col += 1;
            match c {
                '{' => Token {
                    kind: TokenKind::OpenCurly,
                    text: self.chars.next().unwrap().to_string(),
                    loc: self.get_loc(),
                },
                '[' => Token {
                    kind: TokenKind::OpenBracket,
                    text: self.chars.next().unwrap().to_string(),
                    loc: self.get_loc(),
                },
                '}' => Token {
                    kind: TokenKind::CloseCurly,
                    text: self.chars.next().unwrap().to_string(),
                    loc: self.get_loc(),
                },
                ']' => Token {
                    kind: TokenKind::CloseBracket,
                    text: self.chars.next().unwrap().to_string(),
                    loc: self.get_loc(),
                },
                ',' => Token {
                    kind: TokenKind::Comma,
                    text: self.chars.next().unwrap().to_string(),
                    loc: self.get_loc(),
                },
                ':' => Token {
                    kind: TokenKind::Colon,
                    text: self.chars.next().unwrap().to_string(),
                    loc: self.get_loc(),
                },
                'n' => self.get_null_token(),
                't' => self.get_true_token(),
                'f' => self.get_false_token(),
                '"' => self.get_str_token(),
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '-' => {
                    self.get_number_token()
                }
                _ => Token {
                    kind: TokenKind::Invalid,
                    text: self.chars.next().unwrap().to_string(),
                    loc: self.get_loc(),
                },
            }
        } else {
            self.exhausted = true;
            Token {
                kind: TokenKind::Eof,
                text: "".to_string(),
                loc: self.get_loc(),
            }
        }
    }

    fn get_null_token(&mut self) -> Token {
        const ARR: [char; 4] = ['n', 'u', 'l', 'l'];
        self.col -= 1;
        let mut text = String::new();
        for expected in ARR {
            if let Some(c) = self.chars.next_if_eq(&expected) {
                text.push(c);
                self.col += 1;
            } else {
                return Token {
                    kind: TokenKind::Invalid,
                    text,
                    loc: self.get_loc(),
                };
            }
        }
        Token {
            kind: TokenKind::Null,
            text,
            loc: self.get_loc(),
        }
    }

    fn get_true_token(&mut self) -> Token {
        const ARR: [char; 4] = ['t', 'r', 'u', 'e'];
        self.col -= 1;
        let mut text = String::new();
        for expected in ARR {
            if let Some(c) = self.chars.next_if_eq(&expected) {
                text.push(c);
                self.col += 1;
            } else {
                return Token {
                    kind: TokenKind::Invalid,
                    text,
                    loc: self.get_loc(),
                };
            }
        }
        Token {
            kind: TokenKind::True,
            text,
            loc: self.get_loc(),
        }
    }

    fn get_false_token(&mut self) -> Token {
        const ARR: [char; 5] = ['f', 'a', 'l', 's', 'e'];
        self.col -= 1;
        let mut text = String::new();
        for expected in ARR {
            if let Some(c) = self.chars.next_if_eq(&expected) {
                text.push(c);
                self.col += 1;
            } else {
                return Token {
                    kind: TokenKind::Invalid,
                    text,
                    loc: self.get_loc(),
                };
            }
        }
        Token {
            kind: TokenKind::False,
            text,
            loc: self.get_loc(),
        }
    }

    fn get_str_token(&mut self) -> Token {
        let mut text = String::from(self.chars.next().unwrap()); // take the first quotation mark
        let mut escape_next = false;
        while let Some(c) = self.chars.next() {
            self.col += 1;
            text.push(c);
            if escape_next {
                escape_next = false;
                continue;
            }
            match c {
                '\\' => {
                    escape_next = true;
                }
                '"' => {
                    return Token {
                        kind: TokenKind::Str,
                        text,
                        loc: self.get_loc(),
                    }
                }
                _ => continue,
            }
        }
        Token {
            kind: TokenKind::Invalid,
            text,
            loc: self.get_loc(),
        }
    }

    fn get_number_token(&mut self) -> Token {
        let mut text = String::new();
        let mut is_float = false;

        if let Some(c) = self.chars.next_if(|c| c == &'-') {
            self.col += 1;
            text.push(c);
            if let Some(c) = self.chars.peek() {
                if !c.is_ascii_digit() {
                    return Token {
                        kind: TokenKind::Invalid,
                        text,
                        loc: self.get_loc(),
                    };
                }
            } else {
                return Token {
                    kind: TokenKind::Invalid,
                    text,
                    loc: self.get_loc(),
                };
            }
        }
        while let Some(c) = self.chars.next_if(|c| c.is_ascii_digit() || c == &'.') {
            self.col += 1;
            text.push(c);
            if c == '.' && !is_float {
                if let Some(c) = self.chars.next_if(|c| c.is_ascii_digit()) {
                    text.push(c);
                } else {
                    return Token {
                        kind: TokenKind::Invalid,
                        text,
                        loc: self.get_loc(),
                    };
                }
                is_float = true;
            }
        }
        Token {
            kind: {
                if is_float {
                    TokenKind::Float
                } else {
                    TokenKind::Integer
                }
            },
            text,
            loc: self.get_loc(),
        }
    }

    fn trim(&mut self) {
        loop {
            if self.chars.next_if_eq(&'\n').is_some() {
                self.row += 1;
                self.char_count = self.col;
                continue;
            } else if self.chars.next_if(|c| c.is_whitespace()).is_some() {
                self.col += 1;
                continue;
            }
            break;
        }
    }
}

impl<Chars: Iterator<Item = char>> Iterator for Lexer<Chars> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            None
        } else {
            Some(self.next_token())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::borrow::Cow;

    fn test_token_eq<'a, S: Into<Cow<'a, str>> + std::fmt::Display>(
        token: &Token,
        expected_kind: TokenKind,
        expected_text: S,
    ) -> Result<(), TestCaseError> {
        prop_assert_eq!(&expected_kind, &token.kind, "\n{:?}\n", token);
        prop_assert_eq!(
            &expected_text.into().to_string(),
            &token.text,
            "\n{:?}\n",
            token
        );
        Ok(())
    }

    fn test_invalid(token: &Token) -> Result<(), TestCaseError> {
        prop_assert_eq!(&TokenKind::Invalid, &token.kind, "\n{:?}\n", token);
        Ok(())
    }

    fn test_eof(token: &Token) -> Result<(), TestCaseError>{
        test_token_eq(token, TokenKind::Eof, "")
    }

    fn test_token_eq_std<'a, S: Into<Cow<'a, str>> + std::fmt::Display>(
        token: &Token,
        expected_kind: TokenKind,
        expected_text: S,
    ){
        assert_eq!(&expected_kind, &token.kind, "\n{:?}\n", token);
        assert_eq!(
            &expected_text.into().to_string(),
            &token.text,
            "\n{:?}\n",
            token
        );
    }

    fn test_invalid_std(token: &Token) {
        assert_eq!(&TokenKind::Invalid, &token.kind, "\n{:?}\n", token);
    }

    fn test_eof_std(token: &Token) {
        test_token_eq_std(token, TokenKind::Eof, "")
    }

    proptest! {
        #[test]
        fn random_input_test(ref s in r"\s*\PC*\s*") {
            let lexer = Lexer::new(s.chars());
            for token in lexer {
                if token.kind == TokenKind::Invalid {
                    break
                }
                println!("{token:?}");
                match token.kind {
                    TokenKind::Str => continue,
                    TokenKind::Integer => {
                        prop_assert!(token.text.parse::<isize>().is_ok(), "\n{:?}\n", token);
                    }
                    TokenKind::Float => {
                        prop_assert!(token.text.parse::<f64>().is_ok(), "\n{:?}\n", token);
                    }
                    TokenKind::Eof => prop_assert_eq!(&"".to_string(), &token.text, "\n{:?}\n", token),
                    TokenKind::OpenCurly => prop_assert_eq!(&"{".to_string(), &token.text, "\n{:?}\n", token),
                    TokenKind::CloseCurly => prop_assert_eq!(&"}".to_string(), &token.text, "\n{:?}\n", token),
                    TokenKind::OpenBracket => prop_assert_eq!(&"[".to_string(), &token.text, "\n{:?}\n", token),
                    TokenKind::CloseBracket => prop_assert_eq!(&"]".to_string(), &token.text, "\n{:?}\n", token),
                    TokenKind::Comma => prop_assert_eq!(&",".to_string(), &token.text, "\n{:?}\n", token),
                    TokenKind::Colon => prop_assert_eq!(&":".to_string(), &token.text, "\n{:?}\n", token),
                    _ => {panic!()}
                }
            }
        }

        #[test]
        fn invalid_string_only_open(ref s in r#"\s*"\s*"#) {
            let mut lexer = Lexer::new(s.chars());
            let token = lexer.next().unwrap();
            test_invalid(&token)?;
            prop_assert!(token.text.get(..1) == Some("\""), "\n{:?}\n", token);
        }

        #[test]
        fn valid_string_with_random_unicode_whitespaces(ref s in r#"\s*"\s*"\s*"#) {
            let mut lexer = Lexer::new(s.chars());
            test_token_eq(&lexer.next().unwrap(), TokenKind::Str, s.trim())?;
            test_eof(&lexer.next().unwrap())?;
        }

        #[test]
        fn valid_string_with_random_unicode_chars(ref s in r#"\s*"\w*"\s*"#) {
            let mut lexer = Lexer::new(s.chars());
            test_token_eq(&lexer.next().unwrap(), TokenKind::Str, s.trim())?;
            test_eof(&lexer.next().unwrap())?;
        }

        #[test]
        fn valid_string_with_random_digits(ref s in r#"\s*"[0-9]*"\s*"#) {
            let mut lexer = Lexer::new(s.chars());
            test_token_eq(&lexer.next().unwrap(), TokenKind::Str, s.trim())?;
            test_eof(&lexer.next().unwrap())?;
        }

        #[test]
        fn valid_string_with_random_unicode_digits(ref s in r#"\s*"\d*"\s*"#) {
            let mut lexer = Lexer::new(s.chars());
            test_token_eq(&lexer.next().unwrap(), TokenKind::Str, s.trim())?;
            test_eof(&lexer.next().unwrap())?;
        }

        #[test]
        fn valid_string_with_random_keywords(ref s in r#"\s*"(null|true|false)"\s*"#) {
            let mut lexer = Lexer::new(s.chars());
            test_token_eq(&lexer.next().unwrap(), TokenKind::Str, s.trim())?;
            test_eof(&lexer.next().unwrap())?;
        }

        #[test]
        fn valid_string_with_random_unicode_text_without_backslash_or_quotation_mark(ref s in (r#"[^\\"]*"#, r"\s*", r"\s*")
            .prop_map(|(s, ws1, ws2)| ws1 + "\"" + s.as_str() + "\"" + ws2.as_str()))
        {
            let mut lexer = Lexer::new(s.chars());
            test_token_eq(&lexer.next().unwrap(), TokenKind::Str, s.trim())?;
            test_eof(&lexer.next().unwrap())?;
        }

        #[test]
        fn valid_string_with_backslash_and_quotation_mark(ref s in ("(s|q)*", r"\s*", r"\s*")
            .prop_map(|(s, ws1, ws2)| (s.replace('s', "\\\\").replace('q', "\\\""), ws1, ws2))
            .prop_map(|(s, ws1, ws2)| ws1 + "\"" + s.as_str() + "\"" + ws2.as_str()))
        {
            let mut lexer = Lexer::new(s.chars());
            test_token_eq(&lexer.next().unwrap(), TokenKind::Str, s.trim())?;
            test_eof(&lexer.next().unwrap())?;
        }

        #[test]
        fn only_minus_sign(ref s in r"\s*-\s*") {
            let mut lexer = Lexer::new(s.chars());
            test_token_eq(&lexer.next().unwrap(), TokenKind::Invalid, "-")?;
        }

        #[test]
        fn only_punctuation(ref s in r"\s*\.\s*") {
            let mut lexer = Lexer::new(s.chars());
            test_token_eq(&lexer.next().unwrap(), TokenKind::Invalid, ".")?;
        }

        #[test]
        fn punctuation_with_number_after(ref s in (any::<u32>()
            .prop_map(|s| s.to_string()), r"\s*", r"\s*")
            .prop_map(|(d, ws1, ws2)| ws1 + "." + d.as_str() + ws2.as_str()))
        {
            let mut lexer = Lexer::new(s.chars());
            test_invalid(&lexer.next().unwrap())?;
        }

        #[test]
        fn punctuation_with_number_before(ref s in (any::<u32>()
            .prop_map(|s| s.to_string()), r"\s*", r"\s*")
            .prop_map(|(d, ws1, ws2)| ws1 + d.as_str() + "." + ws2.as_str()))
        {
            let mut lexer = Lexer::new(s.chars());
            test_invalid(&lexer.next().unwrap())?;
        }

        #[test]
        fn valid_positive_integer(ref s in (any::<u32>()
            .prop_map(|s| s.to_string()), r"\s*", r"\s*")
            .prop_map(|(d, ws1, ws2)| ws1 + d.as_str() + ws2.as_str()))
        {
            let mut lexer = Lexer::new(s.chars());
            test_token_eq(&lexer.next().unwrap(), TokenKind::Integer, s.trim())?;
            test_eof(&lexer.next().unwrap())?;
        }

        #[test]
        fn valid_negative_integer(ref s in (any::<i32>()
            .prop_filter("Value must be negative".to_owned(), |d| d < &0)
            .prop_map(|s| s.to_string()), r"\s*", r"\s*")
            .prop_map(|(d, ws1, ws2)| ws1 + d.as_str() + ws2.as_str()))
        {
            let mut lexer = Lexer::new(s.chars());
            let token = lexer.next().unwrap();
            test_token_eq(&token, TokenKind::Integer, s.trim())?;
            prop_assert_eq!("-", token.text.get(..1).unwrap(), "\n{:?}\n", token);
            test_eof(&lexer.next().unwrap())?;
        }

        #[test]
        fn valid_positive_float(ref s in (any::<f32>()
            .prop_filter("Value must be bigger than 0 or value is to big, above 100000".to_owned(), |d| d > &0.0 && d < &100000.0)
            .prop_map(|s| s.to_string()), r"\s*", r"\s*")
            .prop_map(|(d, ws1, ws2)| ws1 + d.as_str() + ws2.as_str()))
        {
            prop_assume!(s.contains('.'));
            let mut lexer = Lexer::new(s.chars());
            test_token_eq(&lexer.next().unwrap(), TokenKind::Float, s.trim())?;
            test_eof(&lexer.next().unwrap())?;
        }

        #[test]
        fn valid_negative_float(ref s in (any::<f32>()
            .prop_filter("Value must be negative".to_owned(), |d| d < &0.0)
            .prop_map(|s| s.to_string()), r"\s*", r"\s*")
            .prop_map(|(d, ws1, ws2)| ws1 + d.as_str() + ws2.as_str()))
        {
            prop_assume!(s.contains('.'));
            let mut lexer = Lexer::new(s.chars());
            let token = lexer.next().unwrap();
            test_token_eq(&token, TokenKind::Float, s.trim())?;
            prop_assert_eq!("-", token.text.get(..1).unwrap(), "\n{:?}\n", token);
            test_eof(&lexer.next().unwrap())?;
        }

        // #[test]
        // fn invalid_null_token(ref s in r"\s*(\w+null|\w+null\w+|null\w+)\s*") {
        //     prop_assume!(s.trim() != "null");
        //     let mut lexer = Lexer::new(s.chars());
        //     test_invalid(&lexer.next().unwrap())?;
        // }

        #[test]
        fn valid_null_token(ref s in r"\s*null\s*") {
            let mut lexer = Lexer::new(s.chars());
            test_token_eq(&lexer.next().unwrap(), TokenKind::Null, "null")?;
            test_eof(&lexer.next().unwrap())?;
        }

        // #[test]
        // fn invalid_true_token(ref s in r"\s*(\w+true|\w+true\w+|true\w+)\s*") {
        //     // prop_assume!(s.trim() != "true");
        //     let mut lexer = Lexer::new(s.chars());
        //     test_invalid(&lexer.next().unwrap())?;
        // }

        #[test]
        fn valid_true_token(ref s in r"\s*true\s*") {
            let mut lexer = Lexer::new(s.chars());
            test_token_eq(&lexer.next().unwrap(), TokenKind::True, "true")?;
            test_eof(&lexer.next().unwrap())?;
        }

        // #[test]
        // fn invalid_false_token(ref s in r"\s*(\w+false|\w+false\w+|false\w+)\s*") {
        //     // prop_assume!(s.trim() != "false");
        //     let mut lexer = Lexer::new(s.chars());
        //     // test_invalid(&lexer.next().unwrap())?;
        //     prop_assert_ne!(TokenKind::False, lexer.next().unwrap().kind);
        // }

        #[test]
        fn valid_false_token(ref s in r"\s*false\s*") {
            let mut lexer = Lexer::new(s.chars());
            test_token_eq(&lexer.next().unwrap(), TokenKind::False, "false")?;
            test_eof(&lexer.next().unwrap())?;
        }
    }


    #[test]
    fn valid_float_is_zero() {
        let s = "0.0";
        let mut lexer = Lexer::new(s.chars());
        test_token_eq_std(&lexer.next().unwrap(), TokenKind::Float, s);
        test_eof_std(&lexer.next().unwrap());
    }

    #[test]
    fn empty_string() {
        let s = "\"\"";
        let mut lexer = Lexer::new(s.chars());
        test_token_eq_std(&lexer.next().unwrap(), TokenKind::Str, s);
        test_eof_std(&lexer.next().unwrap());
    }

    #[test]
    fn backslash_string() {
        let s = r#""\""#;
        let mut lexer = Lexer::new(s.chars());
        test_token_eq_std(&lexer.next().unwrap(), TokenKind::Invalid, s);
        test_eof_std(&lexer.next().unwrap());
    }

    #[test]
    fn escaped_quotation_mark_string() {
        let s = r#""\"""#;
        let mut lexer = Lexer::new(s.chars());
        test_token_eq_std(&lexer.next().unwrap(), TokenKind::Str, s);
        test_eof_std(&lexer.next().unwrap());
    }

    #[test]
    fn valid_one_integer_elem_array() {
        let s = r"[4]";
        let mut lexer = Lexer::new(s.chars());
        test_token_eq_std(&lexer.next().unwrap(), TokenKind::OpenBracket, "[");
        test_token_eq_std(&lexer.next().unwrap(), TokenKind::Integer, "4");
        test_token_eq_std(&lexer.next().unwrap(), TokenKind::CloseBracket, "]");
        test_eof_std(&lexer.next().unwrap());
    }

    #[test]
    fn valid_one_float_elem_array() {
        let s = r"[4.0]";
        let mut lexer = Lexer::new(s.chars());
        test_token_eq_std(&lexer.next().unwrap(), TokenKind::OpenBracket, "[");
        test_token_eq_std(&lexer.next().unwrap(), TokenKind::Float, "4.0");
        test_token_eq_std(&lexer.next().unwrap(), TokenKind::CloseBracket, "]");
        test_eof_std(&lexer.next().unwrap());
    }

    #[test]
    fn valid_many_integers_array() {
        let s = r"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]";
        let mut lexer = Lexer::new(s.chars());
        test_token_eq_std(&lexer.next().unwrap(), TokenKind::OpenBracket, "[");
        for i in 0..9 {
            test_token_eq_std(&lexer.next().unwrap(), TokenKind::Integer, i.to_string());
            test_token_eq_std(&lexer.next().unwrap(), TokenKind::Comma, ",");
        }
        test_token_eq_std(&lexer.next().unwrap(), TokenKind::Integer, "9");
        test_token_eq_std(&lexer.next().unwrap(), TokenKind::CloseBracket, "]");
        test_eof_std(&lexer.next().unwrap());
    }
}
