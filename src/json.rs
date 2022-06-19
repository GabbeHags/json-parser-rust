use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::parser::{JsonData, parse_json};

trait JsonState {}
struct Array;
struct Object;
struct Value;
impl JsonState for Array {}
impl JsonState for Object {}
impl JsonState for Value {}

struct Json<S: JsonState> {
    data: Rc<JsonData>,
    marker: std::marker::PhantomData<S>
}

impl<S: JsonState> Display for Json<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.data)

    }
}

impl<S: JsonState> Json<S> {
    pub fn new<R: AsRef<str>>(json: R) -> Self{
        if let Ok(json_data) = parse_json(json) {
            Self {
                data: Rc::new(json_data),
                marker: Default::default()
            }
        } else {
            todo!()
        }
    }
}

impl Json<Object> {
    pub fn get_object(&self, key: &str) -> Json<Object> {
        if let JsonData::Object(map) = self.data.as_ref() {
            if let Some(data) = map.get(key) {
                match data {
                    JsonData::Object(_) => {
                        Json {
                            data: Rc::new(data.to_owned()),
                            marker: Default::default()
                        }
                    }
                    _ => {
                        todo!()
                    }
                }
            }
            else {
                todo!()
            }
        }
        else {
            todo!()
        }
    }
    pub fn get_array(&self, key: &str) -> Json<Array> {
        if let JsonData::Object(map) = self.data.as_ref() {
            if let Some(data) = map.get(key) {
                match data {
                    JsonData::Array(_) => {
                        Json {
                            data: Rc::new(data.to_owned()),
                            marker: Default::default()
                        }
                    }
                    _ => {
                        todo!()
                    }
                }
            }
            else {
                todo!()
            }
        }
        else {
            todo!()
        }
    }
    pub fn get_value(&self, key: &str) -> Json<Value> {
        if let JsonData::Object(map) = self.data.as_ref() {
            if let Some(data) = map.get(key) {
                match data {
                    JsonData::Object(_) => {
                        todo!()
                    }
                    JsonData::Array(_) => {
                        todo!()
                    }
                    _ => {
                        Json {
                            data: Rc::new(data.to_owned()),
                            marker: Default::default()
                        }
                    }
                }
            }
            else {
                todo!()
            }
        }
        else {
            todo!()
        }
    }
}

impl Json<Array> {
    pub fn get_object(&self, index: usize) -> Json<Object> {
        if let JsonData::Array(arr) = self.data.as_ref() {
            if let Some(data) = arr.get(index) {
                match data {
                    JsonData::Object(_) => {
                        Json {
                            data: Rc::new(data.to_owned()),
                            marker: Default::default()
                        }
                    }
                    _ => {
                        todo!()
                    }
                }
            }
            else {
                todo!()
            }
        }
        else {
            todo!()
        }
    }
    pub fn get_array(&self, index: usize) -> Json<Array> {
        if let JsonData::Array(arr) = self.data.as_ref() {
            if let Some(data) = arr.get(index) {
                match data {
                    JsonData::Array(_) => {
                        Json {
                            data: Rc::new(data.to_owned()),
                            marker: Default::default()
                        }
                    }
                    _ => {
                        todo!()
                    }
                }
            }
            else {
                todo!()
            }
        }
        else {
            todo!()
        }
    }
    pub fn get_value(&self, index: usize) -> Json<Value> {
        if let JsonData::Array(arr) = self.data.as_ref() {
            if let Some(data) = arr.get(index) {
                match data {
                    JsonData::Object(_) => {
                        todo!()
                    }
                    JsonData::Array(_) => {
                        todo!()
                    }
                    _ => {
                        Json {
                            data: Rc::new(data.to_owned()),
                            marker: Default::default()
                        }
                    }
                }
            }
            else {
                todo!()
            }
        }
        else {
            todo!()
        }
    }
}

impl Json<Value> {
    pub fn is_null(&self) -> bool {
        self.data.as_ref() == &JsonData::Null
    }
    pub fn get_bool(&self) -> bool{
        if let JsonData::Bool(b) = self.data.as_ref() {
            *b
        } else {
            todo!()
        }
    }
    pub fn get_string(&self) -> &String{
        if let JsonData::Str(s) = self.data.as_ref() {
            s
        }
        else {
            todo!()
        }
    }
    pub fn get_f64(&self) -> f64{
        if let JsonData::Float(f) = self.data.as_ref() {
            *f
        } else {
            todo!()
        }
    }
    pub fn get_i64(&self) -> i64{
        if let JsonData::Integer(i) = self.data.as_ref() {
            *i
        } else {
            todo!()
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::json::{Json, Object, Value};

    #[test]
    fn get_plain_int() {
        let json: Json<Value> = Json::new("1337");
        println!("{}",json);
    }

    #[test]
    fn test() {
        let json: Json<Object> = Json::new("\
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
    }");
        println!("{}", json.get_array("arr2").get_value(3).get_bool());
    }
}