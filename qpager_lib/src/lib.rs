use std::error::Error;
use diesel::result::Error as DieselError;

use serde::{Serialize, Deserialize};
use serde_json::{Value, Map};

pub type Params = Map<String, Value>;

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub method: Method,
    pub params: Map<String, Value>
}

#[derive(Serialize, Deserialize)]
pub struct Responce {
    pub result: bool,
    pub params: Map<String, Value>
}

#[derive(Serialize, Deserialize)]
pub enum Method {
    SignUp,
    LogIn,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RequestResult {
    Ok(()),
    Err(RequestError),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RequestError {
    Auth(AuthError),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AuthError {
    AlreadySignedUp,
}