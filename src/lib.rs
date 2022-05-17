use std::collections::HashMap;


#[allow(dead_code)]
const DEBUG: bool = false;

#[derive(Debug, PartialEq)]
enum JsonData {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    Str(String),
    Array(Vec<JsonData>),
    Object(HashMap<String, JsonData>),
}


#[derive(Debug, PartialEq, Clone, Copy)]
enum JsonType {
    Null,
    Bool,
    Number,
    Str,
    Array,
    Object,
    ValueSep,
    NameSep,
    Closing,
    Opening,
    Ignore,
}

#[derive(Debug)]
pub struct Lexer {
    cursor: usize,
    input: Vec<char>,
    enclosing_stack: Vec<char>,
}

impl Lexer {
    fn new(json: &str) -> Self {
        Self {
            cursor: 0,
            input: json.trim().chars().collect(),
            enclosing_stack: Vec::new(),
        }
    }

    fn parse(&mut self) -> JsonData {
        if DEBUG {
            print!("Was: ");
            self.print();
        }

        let root = self.eat(self.get_next_token_type());

        if DEBUG {
            print!("Now: ");
            self.print();
            println!("{:?}", root.as_ref().unwrap());
            println!("-------------");
        }
        assert!(self.enclosing_stack.is_empty(), "Mismatched enclosings");
        assert!(self.is_empty(), "Parsing is done, but there is still input left to read.");
        root.unwrap()
    }

    fn parse_null(&mut self) -> JsonData {
        if self.equal("null") {
            self.cursor += 4;
            JsonData::Null
        } else {
            panic!("Tried to parse null, but null was not found")
        }
    }

    fn parse_str(&mut self) -> JsonData {
        assert_eq!(self.peek(), '"');
        self.cursor += 1;
        if DEBUG {
            println!(
                "StrVal: {:#?}, Pos: {}, Remaining: {:?}",
                self.input[self.cursor - 1],
                self.cursor - 1,
                self.dump_to_string()
            );
        }
        let mut s = String::new();
        for _ in self.cursor..self.input.len() {
            let c = &self.peek();
            if c != &'"' {
                if self.equal("\\\\") {
                    s.push('\\');
                    s.push('\\');
                    self.cursor += 2;
                } else if self.equal("\\\"") {
                    s.push('"');
                    self.cursor += 2;
                } else {
                    s.push(*c);
                    self.cursor += 1;
                }
                if DEBUG {
                    println!(
                        "StrVal: {:#?}, Pos: {}, Remaining: {:?}",
                        self.input[self.cursor - 1],
                        self.cursor - 1,
                        self.dump_to_string()
                    );
                }
            } else {
                assert_eq!(c, &'"');
                self.cursor += 1;
                if DEBUG {
                    println!(
                        "StrVal: {:#?}, Pos: {}, Remaining: {:?}",
                        self.input[self.cursor - 1],
                        self.cursor - 1,
                        self.dump_to_string()
                    );
                }
                break;
            }
        }
        JsonData::Str(s)
    }


    fn parse_number(&mut self) -> JsonData {
        let mut s = String::new();
        while !self.is_empty() && (self.peek().is_ascii_digit() || self.peek() == '-' || self.peek() == '.')
        {
            s.push(self.peek());
            self.cursor += 1;
        }
        if s.contains('.') {
            JsonData::Float(s.parse::<f64>().unwrap())
        } else {
            JsonData::Integer(s.parse::<i64>().unwrap())
        }

    }

    fn parse_bool(&mut self) -> JsonData {
        if self.equal("true") {
            self.cursor += 4;
            JsonData::Bool(true)
        } else if self.equal("false") {
            self.cursor += 5;
            JsonData::Bool(false)
        } else {
            panic!("Tried to parse bool, but bool was not found")
        }
    }

    fn parse_array(&mut self) -> JsonData {
        use JsonType::{Closing, Ignore, Opening, ValueSep};
        self.eat(Opening);

        let mut v: Vec<JsonData> = vec![];
        let mut token_type = self.get_next_token_type();
        while token_type != Closing {
            if token_type == ValueSep || token_type == Ignore {
                self.eat(token_type);
            } else {
                v.push(self.eat(token_type).unwrap());
            }
            token_type = self.get_next_token_type();
        }

        self.eat(Closing);

        JsonData::Array(v)
    }

    fn parse_object(&mut self) -> JsonData {
        use JsonType::{Closing, Ignore, NameSep, Opening, ValueSep};
        self.eat(Opening);
        if DEBUG {
            println!(
                "Opening: {:#?}, Pos: {}, Remaining: {:?}",
                self.input[self.cursor - 1],
                self.cursor - 1,
                self.dump_to_string()
            );
        }

        let mut h = HashMap::new();
        let mut token_type = self.get_next_token_type();

        while token_type != Closing {
            // skip chars that don't have a value, e.g. the `,` or space
            if token_type == ValueSep || token_type == Ignore {
                self.eat(token_type);
            }
            // we start to parse the name of the value by using our json str parser.
            // then we parse the value like normal.
            else if let JsonData::Str(name) = self.parse_str() {
                token_type = self.get_next_token_type();
                while token_type == NameSep || token_type == Ignore {
                    self.eat(token_type);
                    if DEBUG {
                        println!(
                            "{token_type:?}: {:#?}, Pos: {}, Remaining: {:?}",
                            self.input[self.cursor - 1],
                            self.cursor - 1,
                            self.dump_to_string()
                        );
                    }
                    token_type = self.get_next_token_type();
                }
                h.insert(name, self.eat(token_type).unwrap());
            }
            token_type = self.get_next_token_type();
        }
        self.eat(Closing);
        if DEBUG {
            println!(
                "Closing: {:#?}, Pos: {}, Remaining: {:?}",
                self.input[self.cursor - 1],
                self.cursor - 1,
                self.dump_to_string()
            );
        }
        JsonData::Object(h)
    }

    fn peek(&self) -> char {
        self.input[self.cursor]
    }

    fn eat(&mut self, t: JsonType) -> Option<JsonData> {
        let result = match t {
            JsonType::Null => self.parse_null(),
            JsonType::Str => self.parse_str(),
            JsonType::Number => self.parse_number(),
            JsonType::Bool => self.parse_bool(),
            JsonType::Array => self.parse_array(),
            JsonType::Object => self.parse_object(),
            JsonType::ValueSep => {
                self.cursor += 1;
                return None;
            }
            JsonType::NameSep => {
                self.cursor += 1;
                return None;
            }
            JsonType::Opening => {
                self.push_opening();
                return None;
            }
            JsonType::Closing => {
                self.pop_closing();
                return None;
            }
            JsonType::Ignore => {
                self.cursor += 1;
                return None;
            }
            #[allow(unreachable_patterns)]
            c => {
                self.print();
                panic!("`{c:?}` is invalid Json")
            }
        };
        Some(result)
    }

    fn push_opening(&mut self) {
        let item = match self.peek() {
            '{' => '}',
            '[' => ']',
            c => panic!("Unknown opening bracket: `{c}`"),
        };
        self.enclosing_stack.push(item);
        self.cursor += 1;
    }

    fn pop_closing(&mut self) {
        if let Some(back) = self.enclosing_stack.last() {
            if back == &self.peek() {
                self.enclosing_stack.pop();
                self.cursor += 1;
            } else {
                panic!("Mismatched closing bracket");
            }
        }
    }

    fn equal(&self, s: &str) -> bool {
        let mut start = self.cursor;
        let string = s.to_string();
        for c in string.chars() {
            if self.input.get(start) == Some(&c){
                start += 1;
            }
            else {
                return false;
            }
        }
        true
    }

    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    fn size(&self) -> usize {
        self.input.len() - self.cursor
    }

    fn get_next_token_type(&self) -> JsonType {
        match self.peek() {
            '{' => JsonType::Object,
            '[' => JsonType::Array,
            '"' => JsonType::Str,
            '0' => JsonType::Number,
            '1' => JsonType::Number,
            '2' => JsonType::Number,
            '3' => JsonType::Number,
            '4' => JsonType::Number,
            '5' => JsonType::Number,
            '6' => JsonType::Number,
            '7' => JsonType::Number,
            '8' => JsonType::Number,
            '9' => JsonType::Number,
            '-' => {
                if self.input[self.cursor + 1].is_ascii_digit() {
                    return JsonType::Number;
                }
                self.print();
                panic!("`-` is invalid JsonType at: {}", self.cursor)
            }
            '.' => {
                if self.input[self.cursor + 1].is_ascii_digit() {
                    return JsonType::Number;
                }
                self.print();
                panic!("`.` is invalid JsonType at: {}", self.cursor)
            }
            't' => JsonType::Bool,
            'f' => JsonType::Bool,
            'n' => JsonType::Null,
            ' ' => JsonType::Ignore,
            '\t' => JsonType::Ignore,
            '\n' => JsonType::Ignore,
            '\r' => JsonType::Ignore,
            '}' => JsonType::Closing,
            ']' => JsonType::Closing,
            ',' => JsonType::ValueSep,
            ':' => JsonType::NameSep,
            c => {
                self.print();
                panic!("`{c}` is invalid JsonType at: {}", self.cursor)
            }
        }
    }

    pub fn print(&self) {
        println!("{:?}", self.dump_to_string())
    }

    pub fn dump_to_string(&self) -> String {
        String::from_iter(self.input.iter().skip(self.cursor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_null() {
        assert_eq!(Lexer::new("null").parse(), JsonData::Null);
    }
    #[test]
    #[should_panic(expected = "Tried to parse null, but null was not found")]
    fn parse_not_null() {
        Lexer::new("nul").parse();
    }
    #[test]
    fn parse_empty_string() {
        assert_eq!(Lexer::new("\"\"").parse(), JsonData::Str(String::from("")));
    }
    #[test]
    fn parse_non_empty_string() {
        assert_eq!(
            Lexer::new("\"hej\"").parse(),
            JsonData::Str(String::from("hej"))
        );
    }
    #[test]
    fn parse_integer() {
        assert_eq!(Lexer::new("1337").parse(), JsonData::Integer(1337));
    }
    #[test]
    fn parse_float() {
        assert_eq!(Lexer::new("1337.0").parse(), JsonData::Float(1337.0));
    }
    #[test]
    fn parse_bool_true() {
        assert_eq!(Lexer::new("true").parse(), JsonData::Bool(true));
    }
    #[test]
    fn parse_bool_false() {
        assert_eq!(Lexer::new("false").parse(), JsonData::Bool(false));
    }
    #[test]
    #[should_panic(expected="Tried to parse bool, but bool was not found")]
    fn parse_not_bool_true() {
        Lexer::new("tru").parse();
    }
    #[test]
    #[should_panic(expected="Tried to parse bool, but bool was not found")]
    fn parse_not_bool_false() {
        Lexer::new("fals").parse();
    }
    #[test]
    fn parse_empty_array() {
        assert_eq!(Lexer::new("[]").parse(), JsonData::Array(vec![]));
    }
    #[test]
    fn parse_array_of_all_non_recursive_types() {
        assert_eq!(
            Lexer::new("[null, \"hej\", 1337, 1337.0, true, false]").parse(),
            JsonData::Array(vec![
                JsonData::Null,
                JsonData::Str(String::from("hej")),
                JsonData::Integer(1337),
                JsonData::Float(1337.0),
                JsonData::Bool(true),
                JsonData::Bool(false)
            ])
        );
    }
    #[test]
    fn parse_array_with_array_in() {
        assert_eq!(
            Lexer::new(
                "[null, \"hej\", \
        \
        1337, 1337.0, true, false, [null, \"hej\", 1337, true, false]]"
            )
            .parse(),
            JsonData::Array(vec![
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
            ])
        );
    }
    #[test]
    fn parse_empty_object() {
        assert_eq!(Lexer::new("{}").parse(), JsonData::Object(HashMap::new()));
    }
    #[test]
    fn parse_object_with_a_json_value_in_str() {
        assert_eq!(
            Lexer::new("{\"s1\":\"s1val\"}").parse(),
            JsonData::Object({
                let mut h = HashMap::new();
                h.insert(String::from("s1"), JsonData::Str(String::from("s1val")));
                h
            })
        );
    }
    #[test]
    fn parse_object_with_all_types_except_with_object() {
        assert_eq!(
            Lexer::new(
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
    }\
    "
            )
            .parse(),
            JsonData::Object({
                let mut h = HashMap::new();
                h.insert(String::from("string1"), JsonData::Str(String::from("string1")));
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
            })
        );
    }
    #[test]
    fn test_data_my_1() {
        assert_eq!(
            Lexer::new(include_str!("__test_data__/test_data_my_1.json")).parse(),
            JsonData::Object({
                let mut h = HashMap::new();
                h.insert(
                    "json_str_in_json".to_string(),
                    JsonData::Str(String::from("{\"hej\":null}")),
                );
                h
            })
        );
    }
    #[test]
    fn test_data_my_2() {
        assert_eq!(
            Lexer::new(include_str!("__test_data__/test_data_my_2.json")).parse(),
            JsonData::Object(HashMap::from([
                (
                    "message".to_string(),
                    JsonData::Str("simpler non-flash version\\\\".to_string())
                ),
                ("distinct".to_string(), JsonData::Bool(true))
            ]))
        );
    }
    #[test]
    fn test_data1() {
        Lexer::new(include_str!("__test_data__/test_data1.json")).parse();
    }
    #[test]
    fn test_data2() {
        Lexer::new(include_str!("__test_data__/test_data2.json")).parse();
    }
    #[test]
    fn test_data3() {
        Lexer::new(include_str!("__test_data__/test_data3.json")).parse();
    }
    #[test]
    fn test_data4() {
        Lexer::new(include_str!("__test_data__/test_data4.json")).parse();
    }
    #[test]
    fn test_data5() {
        Lexer::new(include_str!("__test_data__/test_data5.json")).parse();
    }
    #[test]
    fn test_data6() {
        Lexer::new(include_str!("__test_data__/test_data6.json")).parse();
    }
    #[test]
    fn test_data7() {
        Lexer::new(include_str!("__test_data__/test_data7.json")).parse();
    }
    #[test]
    fn test_data8() {
        Lexer::new(include_str!("__test_data__/test_data8.json")).parse();
    }
    #[test]
    fn test_data9() {
        Lexer::new(include_str!("__test_data__/test_data9.json")).parse();
    }
}
