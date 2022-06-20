use crate::parser::{parse_json, JsonData};
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::rc::Rc;
use std::{fs, io};

pub trait JsonState {}
#[derive(Debug)]
pub struct Array;
#[derive(Debug)]
pub struct Object;
#[derive(Debug)]
pub struct Value;
impl JsonState for Array {}
impl JsonState for Object {}
impl JsonState for Value {}

#[derive(Debug, PartialEq)]
pub enum JsonError {
    IncorrectType,
    KeyNotFound,
    IndexNotFound,
    InvalidJsonSyntax(String),
    FileError(io::ErrorKind),
}

#[derive(Debug)]
pub struct Json<S: JsonState> {
    data: Rc<JsonData>,
    marker: std::marker::PhantomData<S>,
}

macro_rules! expect_json_type {
    ($self:expr, $type:ident, $var1:ident, $code:block) => {
        if let JsonData::$type(inner) = $self.data.as_ref() {
            let $var1 = inner;
            $code
        } else {
            Err(JsonError::IncorrectType)
        }
    };
}

impl<S: JsonState> Display for Json<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.data)
    }
}

impl<S: JsonState> Json<S> {
    pub fn new<R: AsRef<str>>(json: R) -> Result<Self, JsonError> {
        let json = parse_json(json);
        match json {
            Ok(json_data) => Ok(Self {
                data: Rc::new(json_data),
                marker: Default::default(),
            }),
            Err(error) => Err(JsonError::InvalidJsonSyntax(format!("{}", error))),
        }
    }

    pub fn from_file<R: AsRef<Path>>(file: R) -> Result<Self, JsonError> {
        match fs::read_to_string(file.as_ref()) {
            Ok(s) => Self::new(s),
            Err(e) => Err(JsonError::FileError(e.kind())),
        }
    }
}

macro_rules! get_from_json_object {
    ($self:expr, $key:expr, $var1:ident, $code:block) => {
        expect_json_type!($self, Object, map, {
            if let Some(data) = map.get($key) {
                let $var1 = data;
                $code
            } else {
                Err(JsonError::KeyNotFound)
            }
        })
    };
}

macro_rules! create_json_of_type {
    (@create $data:expr) => {
        Json {
            data: Rc::new($data.to_owned()),
            marker: Default::default(),
        }
    };
    ($data:expr, Null) => {
        if let JsonData::Null = $data {
            Ok(create_json_of_type!(@create $data))
        } else {
            Err(JsonError::IncorrectType)
        }
    };
    ($data:expr, $type:ident) => {
        if let JsonData::$type(_) = $data {
            Ok(create_json_of_type!(@create $data))
        } else {
            Err(JsonError::IncorrectType)
        }
    };
    ($data:expr, Null, $($rest:ident),*) => {
        if let JsonData::Null = $data {
            Ok(create_json_of_type!(@create $data))
        } else {
            create_json_of_type!($data, $($rest),*)
        }
    };
    ($data:expr, $type:ident, $($rest:ident),*) => {
        if let JsonData::$type(_) = $data {
            Ok(create_json_of_type!(@create $data))
        } else {
            create_json_of_type!($data, $($rest),*)
        }
    };
}

impl Json<Object> {
    pub fn get_object(&self, key: &str) -> Result<Json<Object>, JsonError> {
        get_from_json_object!(self, key, data, { create_json_of_type!(data, Object) })
    }
    pub fn get_array(&self, key: &str) -> Result<Json<Array>, JsonError> {
        get_from_json_object!(self, key, data, { create_json_of_type!(data, Array) })
    }

    pub fn get_value(&self, key: &str) -> Result<Json<Value>, JsonError> {
        get_from_json_object!(self, key, data, {
            create_json_of_type!(data, Integer, Null, Float, Bool, Str)
        })
    }
}

macro_rules! get_from_json_array {
    ($self:expr, $index:expr, $var1:ident, $code:block) => {
        expect_json_type!($self, Array, arr, {
            if let Some(data) = arr.get($index) {
                let $var1 = data;
                $code
            } else {
                Err(JsonError::IndexNotFound)
            }
        })
    };
}

impl Json<Array> {
    pub fn is_empty(&self) -> Result<bool, JsonError> {
        expect_json_type!(self, Array, arr, { Ok(arr.is_empty()) })
    }
    pub fn len(&self) -> Result<usize, JsonError> {
        expect_json_type!(self, Array, arr, { Ok(arr.len()) })
    }
    pub fn get_object(&self, index: usize) -> Result<Json<Object>, JsonError> {
        get_from_json_array!(self, index, data, { create_json_of_type!(data, Object) })
    }
    pub fn get_array(&self, index: usize) -> Result<Json<Array>, JsonError> {
        get_from_json_array!(self, index, data, { create_json_of_type!(data, Array) })
    }
    pub fn get_value(&self, index: usize) -> Result<Json<Value>, JsonError> {
        get_from_json_array!(self, index, data, {
            create_json_of_type!(data, Integer, Float, Bool, Str, Null)
        })
    }
}

impl Json<Value> {
    pub fn is_null(&self) -> bool {
        self.data.as_ref() == &JsonData::Null
    }
    pub fn is_eof(&self) -> bool {
        self.data.as_ref() == &JsonData::Eof
    }
    pub fn get_bool(&self) -> Result<bool, JsonError> {
        expect_json_type!(self, Bool, b, { Ok(*b) })
    }
    pub fn get_string(&self) -> Result<&String, JsonError> {
        expect_json_type!(self, Str, s, { Ok(s) })
    }
    pub fn get_f64(&self) -> Result<f64, JsonError> {
        expect_json_type!(self, Float, f, { Ok(*f) })
    }
    pub fn get_i64(&self) -> Result<i64, JsonError> {
        expect_json_type!(self, Integer, i, { Ok(*i) })
    }
}

#[cfg(test)]
mod tests {
    use crate::json::{Array, Json, Object, Value};

    #[test]
    fn read_from_file_test_data1() {
        Json::<Object>::from_file("src/__test_data__/test_data1.json").unwrap();
    }
    #[test]
    fn read_from_file_test_data2() {
        Json::<Object>::from_file("src/__test_data__/test_data2.json").unwrap();
    }
    #[test]
    fn read_from_file_test_data3() {
        Json::<Array>::from_file("src/__test_data__/test_data3.json").unwrap();
    }
    #[test]
    fn read_from_file_test_data4() {
        Json::<Array>::from_file("src/__test_data__/test_data4.json").unwrap();
    }
    #[test]
    fn read_from_file_test_data5() {
        Json::<Array>::from_file("src/__test_data__/test_data5.json").unwrap();
    }
    #[test]
    fn read_from_file_test_data6() {
        Json::<Object>::from_file("src/__test_data__/test_data6.json").unwrap();
    }
    #[test]
    fn read_from_file_test_data7() {
        Json::<Object>::from_file("src/__test_data__/test_data7.json").unwrap();
    }
    #[test]
    fn read_from_file_test_data8() {
        Json::<Object>::from_file("src/__test_data__/test_data8.json").unwrap();
    }
    #[test]
    fn read_from_file_test_data9() {
        Json::<Object>::from_file("src/__test_data__/test_data9.json").unwrap();
    }
    #[test]
    fn read_from_file_test_data_my_1() {
        let json = Json::<Object>::from_file("src/__test_data__/test_data_my_1.json").unwrap();
        assert_eq!(
            &String::from("{\\\"hej\\\":null}"),
            json.get_value("json_str_in_json")
                .unwrap()
                .get_string()
                .unwrap()
        )
    }
    #[test]
    fn read_from_file_test_data_my_2() {
        let json = Json::<Object>::from_file("src/__test_data__/test_data_my_2.json").unwrap();
        assert!(json.get_value("distinct").unwrap().get_bool().unwrap());
        assert_eq!(
            Ok(&String::from("simpler non-flash version\\\\")),
            json.get_value("message").unwrap().get_string()
        );
    }

    #[test]
    fn is_eof() {
        assert!(Json::new("").unwrap().is_eof())
    }

    #[test]
    fn invalid_json_syntax_str() {
        assert!(Json::<Value>::new("\"hej").is_err())
    }

    #[test]
    fn get_plain_int() {
        assert_eq!(1337, Json::new("1337").unwrap().get_i64().unwrap())
    }
    #[test]
    fn get_plain_float() {
        assert_eq!(
            1337.1337,
            Json::new("1337.1337").unwrap().get_f64().unwrap()
        )
    }
    #[test]
    fn get_plain_string() {
        assert_eq!("hej", Json::new("\"hej\"").unwrap().get_string().unwrap())
    }
    #[test]
    fn get_plain_null() {
        assert!(Json::new("null").unwrap().is_null())
    }
    #[test]
    fn get_plain_bool() {
        assert!(Json::new("true").unwrap().get_bool().unwrap())
    }

    #[test]
    fn json_obj_sub_obj() {
        let json: Json<Object> = Json::new(
            "{
                \"test1\": 123,
                \"sub_obj\" : {
                    \"test2\":\"abc\",
                    \"testarr1\":[{\"a\":1},{\"b\":2},{\"c\":3.3}]
                }
            }",
        )
        .unwrap();
        assert_eq!(Ok(123), json.get_value("test1").unwrap().get_i64());
        let sub_obj = json.get_object("sub_obj").unwrap();
        assert_eq!(
            Ok(&String::from("abc")),
            sub_obj.get_value("test2").unwrap().get_string()
        );
        let sub_obj_arr = sub_obj.get_array("testarr1").unwrap();
        assert_eq!(
            Ok(1),
            sub_obj_arr
                .get_object(0)
                .unwrap()
                .get_value("a")
                .unwrap()
                .get_i64()
        );
        assert_eq!(
            Ok(2),
            sub_obj_arr
                .get_object(1)
                .unwrap()
                .get_value("b")
                .unwrap()
                .get_i64()
        );
        assert_eq!(
            Ok(3.3),
            sub_obj_arr
                .get_object(2)
                .unwrap()
                .get_value("c")
                .unwrap()
                .get_f64()
        );
    }

    #[test]
    fn json_get_everything_test() {
        let json: Json<Object> = Json::new(
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
        )
        .unwrap();
        let arr1 = json.get_array("arr1").unwrap();
        let arr2 = json.get_array("arr2").unwrap();
        let arr3 = json.get_array("arr3").unwrap();
        let arr3_arr = arr3.get_array(5).unwrap();
        assert_eq!(
            Ok(&String::from("string1")),
            json.get_value("string1").unwrap().get_string()
        );
        assert_eq!(
            Ok(&String::from("")),
            json.get_value("string2").unwrap().get_string()
        );
        assert!(json.get_value("null").unwrap().is_null());
        assert_eq!(Ok(1337), json.get_value("integer").unwrap().get_i64());
        assert_eq!(Ok(1337.0), json.get_value("float").unwrap().get_f64());
        assert_eq!(Ok(true), json.get_value("true").unwrap().get_bool());
        assert_eq!(Ok(false), json.get_value("false").unwrap().get_bool());
        assert!(arr1.is_empty() == Ok(true));
        assert!(arr2.get_value(0).unwrap().is_null());
        assert_eq!(
            Ok(&String::from("hej")),
            arr2.get_value(1).unwrap().get_string()
        );
        assert_eq!(Ok(1337), arr2.get_value(2).unwrap().get_i64());
        assert_eq!(Ok(true), arr2.get_value(3).unwrap().get_bool());
        assert_eq!(Ok(false), arr2.get_value(4).unwrap().get_bool());
        assert!(arr3.get_value(0).unwrap().is_null());
        assert_eq!(
            Ok(&String::from("hej")),
            arr3.get_value(1).unwrap().get_string()
        );
        assert_eq!(Ok(1337), arr3.get_value(2).unwrap().get_i64());
        assert_eq!(Ok(true), arr3.get_value(3).unwrap().get_bool());
        assert_eq!(Ok(false), arr3.get_value(4).unwrap().get_bool());
        assert!(arr3_arr.get_value(0).unwrap().is_null());
        assert_eq!(
            Ok(&String::from("hej")),
            arr3_arr.get_value(1).unwrap().get_string()
        );
        assert_eq!(Ok(1337), arr3_arr.get_value(2).unwrap().get_i64());
        assert_eq!(Ok(true), arr3_arr.get_value(3).unwrap().get_bool());
        assert_eq!(Ok(false), arr3_arr.get_value(4).unwrap().get_bool());
    }
}
