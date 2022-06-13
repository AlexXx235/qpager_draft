use serde::{Serialize, Deserialize};
use serde_json::{Value, Map};

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub method: Method,
    pub params: Map<String, Value>
}

#[derive(Serialize, Deserialize)]
pub enum Method {
    SignUp,
    LogIn,
}
