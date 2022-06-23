use core::panic;
use std::io::{self, stdin, Read, ErrorKind};
use std::net::TcpStream;
use std::error::Error;

use tungstenite::stream::MaybeTlsStream;
use tungstenite::{client::connect, WebSocket};
use tungstenite::protocol::Message;

use qpager_lib::{Request, Method, RequestResult, RequestError, MethodResult};

use serde_json as JSON;
use serde::de::Deserialize;
use JSON::{Value, Map, json};

fn main () {
    let (mut socket, _) = connect("ws://localhost:9001").expect("Connection failed");

    //SignUp
    // let request = Request {
    //     request_id: 1,
    //     method: Method::SignUp {
    //         login: String::from("alex"),
    //         password: String::from("qwerty")
    //     },
    //     session_token: None
    // };
    // socket.write_message(Message::Text(JSON::to_string(&request).unwrap())).unwrap();
    // println!("Message sent");
    // let responce = socket.read_message().unwrap();
    // println!("Responce got: {}", responce.to_text().unwrap());

    // LogIn
    let request = Request {
        request_id: 2,
        method: Method::LogIn {
            login: String::from("alex"),
            password: String::from("qwerty")
        },
        session_token: None
    };
    socket.write_message(Message::Text(JSON::to_string(&request).unwrap())).unwrap();
    println!("Message sent");
    let responce = socket.read_message().unwrap();
    println!("Responce got: {}", responce.to_text().unwrap());
    let response = JSON::from_str::<RequestResult>(responce.to_text().unwrap()).unwrap();
    let session_token = if let MethodResult::LogIn{ session_token } = response.result.unwrap() {
        session_token
    } else {
        panic!("Wrong response");
    };

    // SendPrivateMessage
    let request = Request {
        request_id: 3,
        method: Method::SendPrivateMessage {
            message: String::from("hello user"),
            receiver_login: String::from("user")
        },
        session_token: Some(session_token.clone())
    };
    socket.write_message(Message::Text(JSON::to_string(&request).unwrap())).unwrap();
    println!("Message sent");
    let responce = socket.read_message().unwrap();
    println!("Responce got: {}", responce.to_text().unwrap());

    // GetPrivateChatMessages
    let request = Request {
        request_id: 4,
        method: Method::GetPrivateChatMessages {
            second_user_login: String::from("user")
        },
        session_token: Some(session_token.clone())
    };

    socket.write_message(Message::Text(JSON::to_string(&request).unwrap())).unwrap();
    println!("Message sent");
    
    socket.close(None);
    
}
