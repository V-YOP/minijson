
mod lexer;
mod parser;

use std::collections::HashMap;

#[derive(Debug)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>)
}


impl TryFrom<&str> for Json {
    type Error = parser::Error;
    fn try_from(_value: &str) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<Json> for String {
    fn from(_value: Json) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    

    // use super::*;
    
    #[test]
    fn it_works() {
    }
}
