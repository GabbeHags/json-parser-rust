#![allow(dead_code)]

use crate::lexer::{Lexer, Token, TokenKind};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::iter::Peekable;

#[derive(PartialEq)]
enum In {
    Nothing,
    Array,
    Object,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum JsonData {
    Eof,
    Null,
    Bool(bool),
    Str(String),
    Float(f64),
    Integer(i64),
    Array(Vec<JsonData>),
    Object(HashMap<String, JsonData>),
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) enum ParseError {
    SyntaxError(Token),
    UnexpectedEof,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::SyntaxError(token) => {
                let msg = "Invalid Json Syntax";
                write!(
                    f,
                    "{} `{}` at {}:{}\n{}{}",
                    msg,
                    token.text,
                    token.loc.row,
                    token.loc.col,
                    " ".repeat(msg.len() + 2),
                    "^".repeat(token.text.len())
                )
            }
            ParseError::UnexpectedEof => {
                write!(f, "Unexpected end of file")
            }
        }
    }
}

impl fmt::Display for JsonData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            JsonData::Eof => write!(f, ""),
            JsonData::Null => write!(f, "null"),
            JsonData::Bool(b) => write!(f, "{b}"),
            JsonData::Str(s) => write!(f, "\"{s}\""),
            JsonData::Float(float) => write!(f, "{float}"),
            JsonData::Integer(i) => write!(f, "{i}"),
            JsonData::Array(v) => {
                if v.is_empty() {
                    write!(f, "[]")
                } else {
                    write!(f, "[").expect("THIS SHOULD NEVER PANIC");
                    for i in 0..v.len() - 1 {
                        write!(f, "{}, ", v[i]).expect("THIS SHOULD NEVER PANIC");
                    }
                    write!(f, "{}]", v[v.len() - 1])
                }
            }
            JsonData::Object(m) => {
                if m.is_empty() {
                    write!(f, "{{}}")
                } else {
                    writeln!(f, "{{").expect("THIS SHOULD NEVER PANIC");
                    for (count, (s, j)) in m.iter().enumerate() {
                        if m.len() - 1 == count {
                            return write!(f, "\"{s}\" : {j}\n}}");
                        } else {
                            writeln!(f, "\"{s}\" : {j},").expect("THIS SHOULD NEVER PANIC");
                        }
                    }
                    unreachable!();
                }
            }
        }
    }
}

pub(crate) fn parse_json<S: AsRef<str>>(json: S) -> Result<JsonData, ParseError> {
    let mut lexer = Lexer::new(json.as_ref().chars()).peekable();
    eat(&mut lexer, &In::Nothing)
}

fn eat(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In,
) -> Result<JsonData, ParseError> {
    if let Some(token) = lexer.peek() {
        // println!("{token:?}");
        match token.kind {
            TokenKind::CloseBracket => Err(ParseError::SyntaxError(token.to_owned())),
            TokenKind::Comma => Err(ParseError::SyntaxError(token.to_owned())),
            TokenKind::Colon => Err(ParseError::SyntaxError(token.to_owned())),
            TokenKind::CloseCurly => Err(ParseError::SyntaxError(token.to_owned())),
            TokenKind::Invalid => Err(ParseError::SyntaxError(token.to_owned())),
            TokenKind::OpenCurly => parse_json_object(lexer, is_in),
            TokenKind::OpenBracket => parse_json_array(lexer, is_in),
            TokenKind::Integer => parse_json_integer(lexer, is_in),
            TokenKind::Float => parse_json_float(lexer, is_in),
            TokenKind::Str => parse_json_str(lexer, is_in),
            TokenKind::Null => parse_json_null(lexer, is_in),
            TokenKind::True => parse_json_true(lexer, is_in),
            TokenKind::False => parse_json_false(lexer, is_in),
            TokenKind::Eof => parse_json_eof(lexer, is_in),
        }
    } else {
        parse_json_eof(lexer, is_in)
    }
}

fn parse_json_eof(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    _is_in: &In,
) -> Result<JsonData, ParseError> {
    lexer.next();
    Ok(JsonData::Eof)
}

fn parse_json_null(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In,
) -> Result<JsonData, ParseError> {
    lexer.next();
    is_next_valid(lexer, JsonData::Null, is_in)
}

fn parse_json_false(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In,
) -> Result<JsonData, ParseError> {
    lexer.next();
    is_next_valid(lexer, JsonData::Bool(false), is_in)
}

fn parse_json_true(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In,
) -> Result<JsonData, ParseError> {
    lexer.next();
    is_next_valid(lexer, JsonData::Bool(true), is_in)
}

fn parse_json_str(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In,
) -> Result<JsonData, ParseError> {
    let token = lexer.next().unwrap();
    // println!("Current Token: {token:?}");
    is_next_valid(
        lexer,
        JsonData::Str(remove_surrounding_quotes(token.text.as_str())),
        is_in,
    )
}

fn parse_json_float(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In,
) -> Result<JsonData, ParseError> {
    let token = lexer.next().unwrap();
    // println!("Current Token: {token:?}");
    if let Ok(f) = token.text.parse::<f64>() {
        is_next_valid(lexer, JsonData::Float(f), is_in)
    } else {
        Err(ParseError::SyntaxError(token))
    }
}

fn parse_json_integer(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In,
) -> Result<JsonData, ParseError> {
    let token = lexer.next().unwrap();
    // println!("Current Token: {token:?}");
    if let Ok(i) = token.text.parse::<i64>() {
        let next = is_next_valid(lexer, JsonData::Integer(i), is_in);
        if next.is_err() {
            Err(ParseError::SyntaxError(token))
        } else {
            next
        }
    } else {
        Err(ParseError::SyntaxError(token))
    }
}

fn parse_json_array(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In,
) -> Result<JsonData, ParseError> {
    lexer.next();
    let mut arr: Vec<JsonData> = Vec::new();
    let mut elem: Result<JsonData, ParseError>;
    while let Some(token) = lexer.peek() {
        // println!("Current Token: {token:?}");
        elem = match token.kind {
            TokenKind::CloseBracket => {
                lexer.next();
                break;
            }
            TokenKind::Comma => {
                lexer.next();
                continue;
            }
            _ => eat(lexer, &In::Array),
        };
        if let Ok(e) = elem {
            arr.push(e);
        } else {
            return elem;
        }
    }
    is_next_valid(lexer, JsonData::Array(arr), is_in)
}

fn parse_json_object(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In,
) -> Result<JsonData, ParseError> {
    lexer.next();
    let mut map: HashMap<String, JsonData> = HashMap::new();
    let mut elem: Result<JsonData, ParseError>;
    let mut is_key = true;
    let mut key: String = "".into();
    while let Some(token) = lexer.peek() {
        // println!("Current Token: {token:?}");
        elem = match token.kind {
            TokenKind::CloseCurly => {
                lexer.next();
                break;
            }
            TokenKind::Comma => {
                if is_key {
                    return Err(ParseError::SyntaxError(token.to_owned()));
                }
                is_key = true;
                lexer.next();
                continue;
            }
            TokenKind::Colon => {
                if !is_key {
                    return Err(ParseError::SyntaxError(token.to_owned()));
                }
                is_key = false;
                lexer.next();
                continue;
            }
            TokenKind::Str => {
                if is_key {
                    key = remove_surrounding_quotes(token.text.as_str());
                    lexer.next();
                    continue;
                } else {
                    parse_json_str(lexer, &In::Object)
                }
            }
            _ => {
                if is_key {
                    Err(ParseError::SyntaxError(token.to_owned()))
                } else {
                    eat(lexer, &In::Object)
                }
            }
        };
        if let Ok(e) = elem {
            map.insert(key.to_string(), e);
        } else {
            return elem;
        }
    }
    is_next_valid(lexer, JsonData::Object(map), is_in)
}

fn is_next_valid(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    current: JsonData,
    is_in: &In,
) -> Result<JsonData, ParseError> {
    if let Some(next_token) = lexer.peek() {
        // println!("Next Token: {next_token:?}");
        let kind = &next_token.kind;
        return if (kind == &TokenKind::Comma && (is_in == &In::Array || is_in == &In::Object))
            || (kind == &TokenKind::CloseBracket && is_in == &In::Array)
            || (kind == &TokenKind::CloseCurly && is_in == &In::Object)
            || (kind == &TokenKind::Eof && is_in == &In::Nothing)
        {
            Ok(current)
        } else {
            Err(ParseError::SyntaxError(next_token.to_owned()))
        };
    }
    Err(ParseError::UnexpectedEof)
}

// Removes the surrounding quotes from the string
fn remove_surrounding_quotes<S: AsRef<str>>(text: S) -> String {
    // println!("{:?}", text.as_ref());
    let text = text.as_ref();
    debug_assert!(
        text.find('"') == Some(0) && text.rfind('"') == Some(text.len() - 1) && text.len() >= 2,
        "String was not surrounded with quotes, THIS SHOULD ALWAYS BE TRUE"
    );
    text[1..text.len() - 1].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_json() -> impl Strategy<Value = JsonData> {
        // https://altsysrq.github.io/proptest-book/proptest/tutorial/recursive.html
        let leaf = prop_oneof![
            Just(JsonData::Null),
            any::<bool>().prop_map(JsonData::Bool),
            any::<i64>().prop_map(JsonData::Integer),
            (-1000.0..1000.0).prop_map(JsonData::Float),
            r#"[^\\"]*"#.prop_map(JsonData::Str)
        ];
        leaf.prop_recursive(4, 64, 8, |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..12).prop_map(JsonData::Array),
                prop::collection::hash_map(r#"[^\\"]*"#, inner, 0..12).prop_map(JsonData::Object),
            ]
        })
    }

    proptest! {
        #[test]
        fn random_input_test(ref s in r"\s*\PC*\s*") {
            let json = parse_json(s);
            let s = s.trim();
            // println!("{s}");
            if s.is_empty() {
                prop_assert_eq!(Ok(JsonData::Eof), json);
            }
            else if s.find('\"') == Some(0) && s.rfind('\"') == Some(s.len()-1) && s.len() >= 2 {
                prop_assume!(
                    (remove_surrounding_quotes(s).contains('\"') && remove_surrounding_quotes(s).contains(r#"\\""#))
                    || (!remove_surrounding_quotes(s).contains('\"') && !remove_surrounding_quotes(s).contains(r#"\\""#))
                );
                prop_assert_eq!(Ok(JsonData::Str(remove_surrounding_quotes(s))), json);
            }
            else if let Ok(i) =  s.parse::<i64>() {
                if s.starts_with('+') || s.starts_with('.') {
                    prop_assert!(json.is_err());
                    // prop_assert_eq!(Err(JsonErr::Err(_)), json);
                } else {
                    prop_assert_eq!(Ok(JsonData::Integer(i)), json);
                }
            }
            else if let Ok(f) =  s.parse::<f64>() {
                if s.ends_with('.') || s.starts_with('+') || s.starts_with('.'){
                    prop_assert!(json.is_err());
                    // prop_assert_eq!(Err(JsonErr::Err), json);
                } else {
                    prop_assert_eq!(Ok(JsonData::Float(f)), json);
                }
            }
            else if s.find('{') == Some(0) && s.rfind('}') == Some(s.len()-1){
                prop_assume!(s.len() == 2);
                prop_assert_eq!(Ok(JsonData::Object(HashMap::new())), json);
            }
            else {
                prop_assert!(json.is_err());
                // prop_assert_eq!(Err(JsonErr::Err), json);
            }
        }

        #[test]
        fn valid_random_str(ref s in r#"\s*"[^\\"]*"\s*"#) {
            let json = parse_json(s);
            let s = s.trim();
            prop_assert_eq!(Ok(JsonData::Str(remove_surrounding_quotes(s))), json)
        }

        #[test]
        fn valid_random_json(ref s in arb_json()) {
            let json = parse_json(s.to_string()).unwrap();
            prop_assert_eq!(s, &json);
        }
    }

    #[test]
    fn invalid_object_comma() {
        assert!(parse_json("{,").is_err())
    }

    #[test]
    fn invalid_integer_trailing_closed_curly() {
        assert!(parse_json("0}").is_err());
    }

    #[test]
    fn valid_str_one_escaped_quotation() {
        let s = r#""\"""#;
        let json = parse_json(s);
        assert_eq!(Ok(JsonData::Str(remove_surrounding_quotes(s))), json);
    }

    #[test]
    fn valid_null() {
        let json = parse_json("null");
        assert_eq!(Ok(JsonData::Null), json);
    }

    #[test]
    fn valid_true() {
        let json = parse_json("true");
        assert_eq!(Ok(JsonData::Bool(true)), json);
    }

    #[test]
    fn valid_false() {
        let json = parse_json("false");
        assert_eq!(Ok(JsonData::Bool(false)), json);
    }

    #[test]
    fn valid_eof() {
        let json = parse_json("");
        assert_eq!(Ok(JsonData::Eof), json);
    }

    #[test]
    fn valid_empty_str() {
        let json = parse_json("\"\"");
        assert_eq!(Ok(JsonData::Str(String::from(""))), json);
    }

    #[test]
    fn valid_str() {
        let json = parse_json("\"test1234\"");
        assert_eq!(Ok(JsonData::Str(String::from("test1234"))), json);
    }

    #[test]
    fn valid_integer() {
        let json = parse_json("1000");
        assert_eq!(Ok(JsonData::Integer(1000)), json);
    }

    #[test]
    fn valid_float() {
        let json = parse_json("1000.0");
        assert_eq!(Ok(JsonData::Float(1000.0)), json);
    }

    #[test]
    fn valid_empty_array() {
        let json = parse_json("[]");
        assert_eq!(Ok(JsonData::Array(vec![])), json);
    }

    #[test]
    fn valid_array_one_str_elem_array() {
        let json = parse_json("[\"t\"]");
        assert_eq!(Ok(JsonData::Array(vec![JsonData::Str("t".into())])), json);
    }

    #[test]
    fn valid_array_one_integer_elem_array() {
        let json = parse_json("[4]");
        assert_eq!(Ok(JsonData::Array(vec![JsonData::Integer(4)])), json);
    }

    #[test]
    fn valid_array() {
        let json = parse_json("[\"t\", \"e\", \"s\", \"t\", 1, 2, 3, 4]");
        // println!("{}", json.as_ref().unwrap());
        assert_eq!(
            Ok(JsonData::Array(vec![
                JsonData::Str("t".into()),
                JsonData::Str("e".into()),
                JsonData::Str("s".into()),
                JsonData::Str("t".into()),
                JsonData::Integer(1),
                JsonData::Integer(2),
                JsonData::Integer(3),
                JsonData::Integer(4)
            ])),
            json
        );
    }

    #[test]
    fn invalid_object_no_colon() {
        let json = parse_json("{\"hej\"123}");
        assert!(json.is_err())
    }

    #[test]
    fn valid_object_many_kv() {
        let json =
            parse_json("{\"test_name1\":1,\"test_name2\":2,\"test_name3\":3,\"test_name4\":4}");
        assert_eq!(
            Ok(JsonData::Object(HashMap::from([
                ("test_name1".to_string(), JsonData::Integer(1)),
                ("test_name2".to_string(), JsonData::Integer(2)),
                ("test_name3".to_string(), JsonData::Integer(3)),
                ("test_name4".to_string(), JsonData::Integer(4)),
            ]))),
            json
        );
    }

    #[test]
    fn valid_object_one_kv() {
        let json = parse_json("{\"test_name\":1}");
        assert_eq!(
            Ok(JsonData::Object(HashMap::from([(
                "test_name".to_string(),
                JsonData::Integer(1)
            )]))),
            json
        );
    }

    #[test]
    fn valid_empty_object() {
        let json = parse_json("{}");
        assert_eq!(Ok(JsonData::Object(HashMap::from([]))), json);
    }

    fn parse_array_of_all_non_recursive_types() {
        let json = parse_json("[null, \"hej\", 1337, 1337.0, true, false]");
        // println!("{}", json.as_ref().unwrap());
        assert_eq!(
            Ok(JsonData::Array(vec![
                JsonData::Null,
                JsonData::Str(String::from("hej")),
                JsonData::Integer(1337),
                JsonData::Float(1337.0),
                JsonData::Bool(true),
                JsonData::Bool(false)
            ])),
            json
        );
    }

    #[test]
    fn parse_array_with_array_in_array() {
        let json = parse_json(
            "[null, \"hej\", 1337, 1337.0, true, false, [null, \"hej\", 1337, true, false]]",
        );
        // println!("{}", json.as_ref().unwrap());
        assert_eq!(
            Ok(JsonData::Array(vec![
                JsonData::Null,
                JsonData::Str(String::from("hej")),
                JsonData::Integer(1337),
                JsonData::Float(1337.0),
                JsonData::Bool(true),
                JsonData::Bool(false),
                JsonData::Array(vec![
                    JsonData::Null,
                    JsonData::Str(String::from("hej")),
                    JsonData::Integer(1337),
                    JsonData::Bool(true),
                    JsonData::Bool(false),
                ])
            ])),
            json
        );
    }

    #[test]
    fn parse_object_with_a_json_value_in_str() {
        assert_eq!(
            Ok(JsonData::Object({
                let mut h = HashMap::new();
                h.insert(String::from("s1"), JsonData::Str(String::from("s1val")));
                h
            })),
            parse_json("{\"s1\":\"s1val\"}")
        );
    }

    #[test]
    fn parse_object_with_all_types_except_with_object() {
        let json = parse_json(
            "\
    {\
        \"string1\" : \"string1\",\
        \"string2\" : \"\",\
        \"null\" : null,\
        \"integer\":1337,\
        \"float\":1337.0,\
        \"true\": true,\
        \"false\": false,\
        \"arr1\" :[],\
        \"arr2\" :[null, \"hej\", 1337, true, false],\
        \"arr3\":[null, \"hej\", 1337, true, false, [null, \"hej\", 1337, true, false]]\
    }",
        );
        assert_eq!(
            Ok(JsonData::Object({
                let mut h = HashMap::new();
                h.insert(
                    String::from("string1"),
                    JsonData::Str(String::from("string1")),
                );
                h.insert(String::from("string2"), JsonData::Str(String::from("")));
                h.insert(String::from("null"), JsonData::Null);
                h.insert(String::from("integer"), JsonData::Integer(1337));
                h.insert(String::from("float"), JsonData::Float(1337.0));
                h.insert(String::from("true"), JsonData::Bool(true));
                h.insert(String::from("false"), JsonData::Bool(false));
                h.insert(String::from("arr1"), JsonData::Array(vec![]));
                h.insert(
                    String::from("arr2"),
                    JsonData::Array(vec![
                        JsonData::Null,
                        JsonData::Str(String::from("hej")),
                        JsonData::Integer(1337),
                        JsonData::Bool(true),
                        JsonData::Bool(false),
                    ]),
                );
                h.insert(
                    String::from("arr3"),
                    JsonData::Array(vec![
                        JsonData::Null,
                        JsonData::Str(String::from("hej")),
                        JsonData::Integer(1337),
                        JsonData::Bool(true),
                        JsonData::Bool(false),
                        JsonData::Array(vec![
                            JsonData::Null,
                            JsonData::Str(String::from("hej")),
                            JsonData::Integer(1337),
                            JsonData::Bool(true),
                            JsonData::Bool(false),
                        ]),
                    ]),
                );
                h
            })),
            json
        );
    }
}
