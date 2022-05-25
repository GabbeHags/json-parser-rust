#![allow(dead_code)]

use crate::lexer::{Lexer, TokenKind};
use std::collections::HashMap;
use std::iter::Peekable;

#[derive(PartialEq)]
enum In {
    Nothing,
    Array,
    Object
}

#[derive(Debug, PartialEq)]
enum Json {
    Eof,
    Null,
    True,
    False,
    Str(String),
    Float(f64),
    Integer(i64),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
}

#[derive(Debug, PartialEq)]
enum JsonErr {
    err,
}

fn parse_json<S: AsRef<str>>(json: S) -> Result<Json, JsonErr> {
    let mut lexer = Lexer::new(json.as_ref().chars()).peekable();
    eat(&mut lexer, &In::Nothing)
}

fn eat(lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>, is_in: &In) -> Result<Json, JsonErr> {
    if let Some(token) = lexer.peek() {
        match token.kind {
            TokenKind::CloseBracket => Err(JsonErr::err),
            TokenKind::Comma => Err(JsonErr::err),
            TokenKind::Colon => Err(JsonErr::err),
            TokenKind::CloseCurly => Err(JsonErr::err),
            TokenKind::Invalid => Err(JsonErr::err),
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
    _is_in: &In
) -> Result<Json, JsonErr> {
    lexer.next();
    Ok(Json::Eof)
}

fn parse_json_null(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In
) -> Result<Json, JsonErr> {
    lexer.next();
    is_next_valid(lexer, Json::Null, is_in)
}

fn parse_json_false(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In
) -> Result<Json, JsonErr> {
    lexer.next();
    is_next_valid(lexer, Json::False, is_in)
}

fn parse_json_true(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In
) -> Result<Json, JsonErr> {
    lexer.next();
    is_next_valid(lexer, Json::True, is_in)
}

fn parse_json_str(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In
) -> Result<Json, JsonErr> {
    let token = lexer.next().unwrap();
    println!("Current Token: {token:?}");
    is_next_valid(lexer, Json::Str(remove_surrounding_quotes(token.text.as_str())), is_in)
}

fn parse_json_float(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In
) -> Result<Json, JsonErr> {
    let token = lexer.next().unwrap();
    println!("Current Token: {token:?}");
    if let Ok(f) = token.text.parse::<f64>() {
        is_next_valid(lexer, Json::Float(f), is_in)
    } else {
        Err(JsonErr::err)
    }
}

fn parse_json_integer(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In
) -> Result<Json, JsonErr> {
    let token = lexer.next().unwrap();
    println!("Current Token: {token:?}");
    if let Ok(i) = token.text.parse::<i64>() {
        is_next_valid(lexer, Json::Integer(i), is_in)
    } else {
        Err(JsonErr::err)
    }
}

fn parse_json_array(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In
) -> Result<Json, JsonErr> {
    lexer.next();
    let mut arr: Vec<Json> = Vec::new();
    let mut elem: Result<Json, JsonErr>;
    while let Some(token) = lexer.peek() {
        println!("Current Token: {token:?}");
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
    is_next_valid(lexer, Json::Array(arr), is_in)
}

fn parse_json_object(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    is_in: &In
) -> Result<Json, JsonErr> {
    lexer.next();
    let mut map: HashMap<String, Json> = HashMap::new();
    let mut elem: Result<Json, JsonErr>;
    let mut is_key = true;
    let mut key: String = "".into();
    while let Some(token) = lexer.peek() {
        println!("Current Token: {token:?}");
        elem = match token.kind {
            TokenKind::CloseCurly => {
                lexer.next();
                break;
            }
            TokenKind::Comma => {
                lexer.next();
                is_key = true;
                continue;
            }
            TokenKind::Colon => {
                lexer.next();
                is_key = false;
                continue;
            }
            TokenKind::Str => {
                if is_key {
                    key =  remove_surrounding_quotes(token.text.as_str());
                    lexer.next();
                    continue;
                } else {
                    parse_json_str(lexer, &In::Object)
                }
            }
            _ => eat(lexer, &In::Object),
        };
        if let Ok(e) = elem {
            map.insert(key.to_string(), e);
        } else {
            return elem;
        }
    }
    is_next_valid(lexer, Json::Object(map), is_in)
}

fn is_next_valid(
    lexer: &mut Peekable<Lexer<impl Iterator<Item = char>>>,
    current: Json,
    is_in: &In
) -> Result<Json, JsonErr> {
    if let Some(next_token) = lexer.peek() {
        println!("Next Token: {next_token:?}");
        let kind = &next_token.kind;
        if (kind == &TokenKind::Comma && (is_in == &In::Array || is_in == &In::Object))
            || (kind == &TokenKind::CloseBracket && is_in == &In::Array)
            || (kind == &TokenKind::CloseCurly && is_in == &In::Object)
            || (kind == &TokenKind::Eof && is_in == &In::Nothing)
        {
            return Ok(current)
        }
    }
    Err(JsonErr::err)
}

// Removes the surrounding quotes from the string
fn remove_surrounding_quotes<S: AsRef<str>>(text: S) -> String {
    // println!("{:?}", text.as_ref());
    let text = text.as_ref();
    if text.find('"') == Some(0) && text.rfind('"') == Some(text.len()-1) && text.len() >= 2{
        text[1..text.len()-1].to_string()
    } else {
        panic!("String was not surrounded with quotes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn random_input_test(ref s in r"\s*\PC*\s*") {
            let json = parse_json(s);
            let s = s.trim();
            // println!("{s}");
            if s.is_empty() {
                prop_assert_eq!(Ok(Json::Eof), json);
            }
            else if s.find('\"') == Some(0) && s.rfind('\"') == Some(s.len()-1) && s.len() >= 2 {
                prop_assume!(
                    (remove_surrounding_quotes(s).contains('\"') && remove_surrounding_quotes(s).contains(r#"\\""#))
                    || (!remove_surrounding_quotes(s).contains('\"') && !remove_surrounding_quotes(s).contains(r#"\\""#))
                );
                prop_assert_eq!(Ok(Json::Str(remove_surrounding_quotes(s))), json);
            }
            else if let Ok(i) =  s.parse::<i64>() {
                prop_assert_eq!(Ok(Json::Integer(i)), json);
            }
            else if let Ok(f) =  s.parse::<f64>() {
                prop_assert_eq!(Ok(Json::Float(f)), json);
            }
            else if s.find('{') == Some(0) && s.rfind('}') == Some(s.len()-1){
                prop_assume!(s.len() == 2);
                prop_assert_eq!(Ok(Json::Object(HashMap::new())), json);
            }
            else {
                prop_assert_eq!(Err(JsonErr::err), json);
            }
        }

        #[test]
        fn valid_random_str(ref s in r#"\s*"[^\\"]*"\s*"#) {
            let json = parse_json(s);
            prop_assert_eq!(Ok(Json::Str(remove_surrounding_quotes(s.trim()))), json)
        }

        #[test]
        fn valid_random_json(ref s in "") {

        }
    }

    #[test]
    fn invalid_integer_trailing() {
        let json = parse_json("0}");
        assert_eq!(Err(JsonErr::err), json)
    }

    #[test]
    fn valid_str_one_escaped_quotation() {
        let s = r#""\"""#;
        let json = parse_json(s);
        assert_eq!(Ok(Json::Str(remove_surrounding_quotes(s))), json);
    }

    #[test]
    fn valid_null() {
        let json = parse_json("null");
        assert_eq!(Ok(Json::Null), json);
    }

    #[test]
    fn valid_true() {
        let json = parse_json("true");
        assert_eq!(Ok(Json::True), json);
    }

    #[test]
    fn valid_false() {
        let json = parse_json("false");
        assert_eq!(Ok(Json::False), json);
    }

    #[test]
    fn valid_eof() {
        let json = parse_json("");
        assert_eq!(Ok(Json::Eof), json);
    }

    #[test]
    fn valid_empty_str() {
        let json = parse_json("\"\"");
        assert_eq!(Ok(Json::Str(String::from(""))), json);
    }

    #[test]
    fn valid_str() {
        let json = parse_json("\"test1234\"");
        assert_eq!(Ok(Json::Str(String::from("test1234"))), json);
    }

    #[test]
    fn valid_integer() {
        let json = parse_json("1000");
        assert_eq!(Ok(Json::Integer(1000)), json);
    }

    #[test]
    fn valid_float() {
        let json = parse_json("1000.0");
        assert_eq!(Ok(Json::Float(1000.0)), json);
    }

    #[test]
    fn valid_empty_array() {
        let json = parse_json("[]");
        assert_eq!(Ok(Json::Array(vec![])), json);
    }

    #[test]
    fn valid_array_one_str_elem_array() {
        let json = parse_json("[\"t\"]");
        assert_eq!(Ok(Json::Array(vec![Json::Str("t".into())])), json);
    }

    #[test]
    fn valid_array_one_integer_elem_array() {
        let json = parse_json("[4]");
        assert_eq!(Ok(Json::Array(vec![Json::Integer(4)])), json);
    }

    #[test]
    fn valid_array() {
        let json = parse_json("[\"t\", \"e\", \"s\", \"t\", 1, 2, 3, 4]");
        assert_eq!(
            Ok(Json::Array(vec![
                Json::Str("t".into()),
                Json::Str("e".into()),
                Json::Str("s".into()),
                Json::Str("t".into()),
                Json::Integer(1),
                Json::Integer(2),
                Json::Integer(3),
                Json::Integer(4)
            ])),
            json
        );
    }

    #[test]
    fn valid_object_many_kv() {
        let json =
            parse_json("{\"test_name1\":1,\"test_name2\":2,\"test_name3\":3,\"test_name4\":4}");
        assert_eq!(
            Ok(Json::Object(HashMap::from([
                ("test_name1".to_string(), Json::Integer(1)),
                ("test_name2".to_string(), Json::Integer(2)),
                ("test_name3".to_string(), Json::Integer(3)),
                ("test_name4".to_string(), Json::Integer(4)),
            ]))),
            json
        );
    }

    #[test]
    fn valid_object_one_kv() {
        let json = parse_json("{\"test_name\":1}");
        assert_eq!(
            Ok(Json::Object(HashMap::from([(
                "test_name".to_string(),
                Json::Integer(1)
            )]))),
            json
        );
    }

    #[test]
    fn valid_empty_object() {
        let json = parse_json("{}");
        assert_eq!(Ok(Json::Object(HashMap::from([]))), json);
    }

    fn parse_array_of_all_non_recursive_types() {
        assert_eq!(
            Ok(Json::Array(vec![
                Json::Null,
                Json::Str(String::from("hej")),
                Json::Integer(1337),
                Json::Float(1337.0),
                Json::True,
                Json::False
            ])),
            parse_json("[null, \"hej\", 1337, 1337.0, true, false]")
        );
    }

    #[test]
    fn parse_array_with_array_in_array() {
        assert_eq!(
            Ok(Json::Array(vec![
                Json::Null,
                Json::Str(String::from("hej")),
                Json::Integer(1337),
                Json::Float(1337.0),
                Json::True,
                Json::False,
                Json::Array(vec![
                    Json::Null,
                    Json::Str(String::from("hej")),
                    Json::Integer(1337),
                    Json::True,
                    Json::False,
                ])
            ])),
            parse_json(
                "[null, \"hej\", 1337, 1337.0, true, false, [null, \"hej\", 1337, true, false]]"
            )
        );
    }

    #[test]
    fn parse_object_with_a_json_value_in_str() {
        assert_eq!(
            Ok(Json::Object({
                let mut h = HashMap::new();
                h.insert(String::from("s1"), Json::Str(String::from("s1val")));
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
            Ok(Json::Object({
                let mut h = HashMap::new();
                h.insert(String::from("string1"), Json::Str(String::from("string1")));
                h.insert(String::from("string2"), Json::Str(String::from("")));
                h.insert(String::from("null"), Json::Null);
                h.insert(String::from("integer"), Json::Integer(1337));
                h.insert(String::from("float"), Json::Float(1337.0));
                h.insert(String::from("true"), Json::True);
                h.insert(String::from("false"), Json::False);
                h.insert(String::from("arr1"), Json::Array(vec![]));
                h.insert(
                    String::from("arr2"),
                    Json::Array(vec![
                        Json::Null,
                        Json::Str(String::from("hej")),
                        Json::Integer(1337),
                        Json::True,
                        Json::False,
                    ]),
                );
                h.insert(
                    String::from("arr3"),
                    Json::Array(vec![
                        Json::Null,
                        Json::Str(String::from("hej")),
                        Json::Integer(1337),
                        Json::True,
                        Json::False,
                        Json::Array(vec![
                            Json::Null,
                            Json::Str(String::from("hej")),
                            Json::Integer(1337),
                            Json::True,
                            Json::False,
                        ]),
                    ]),
                );
                h
            })),
            json
        );
    }
}
