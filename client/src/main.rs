use std::io;

use tungstenite::client::connect;
use tungstenite::protocol::Message;

use qpager_lib::{Request, Method, Responce};

use serde_json as JSON;
use JSON::{Value, Map, json};

fn show_menu() {
    println!("1 - Open chat");
    println!("2 - Registration");
    println!("3 - Authorization");
    println!("4 - Verify session tokens");
}

fn get_user_action() -> u32 {
    let mut select = String::new();

    show_menu();

    loop {
        println!("Choose option: ");
        if let Ok(_) = io::stdin().read_line(&mut select) {
            if let Ok(num) = select.trim().parse::<u32>() {
                return num;
            } else {
                println!("Enter a number! Try again");
                continue;    
            }
        } else {
            println!("Input failed! Try again");
            continue;
        }
    }
}   

fn main () {
    let (mut socket, _) = connect("ws://localhost:9001").expect("Connection failed");
    let request = Request {
        method: Method::LogIn,
        params: json!({
            "text": "hello from client"
        })
    };
    socket.write_message(Message::Text(JSON::to_string(&request).unwrap())).unwrap();
    println!("Message sent");
    let responce = socket.read_message().unwrap();
    println!("Responce got: {}", responce.to_text().unwrap());
    socket.close(None);
    // let mut session_token = String::new();

    // loop {
    //     let choice = get_user_action();

    //     match choice {
    //         1 => continue,
            
    //         2 => {
    //             let mut params = Params::new();
    //             params.insert(String::from("login"), Value::String(String::from("alex")));
    //             params.insert(String::from("password"), Value::String(String::from("qwerty")));

    //             let request = Request {
    //                 method: Method::SignUp,
    //                 params: params
    //             };

    //             let msg = Message::Text(JSON::to_string(&request).unwrap());
    //             socket.write_message(msg);
    //         }, 

    //         3 => {
    //             let mut params = <Map<String, Value>>::new();
    //             params.insert(String::from("login"), Value::String(String::from("alex")));
    //             params.insert(String::from("password"), Value::String(String::from("qwerty")));

    //             let request = Request {
    //                 method: Method::LogIn,
    //                 params: params
    //             };

    //             let msg = Message::Text(JSON::to_string(&request).unwrap());
    //             socket.write_message(msg);
    //             let responce = socket.read_message().unwrap();
    //             let responce: Responce = JSON::from_str(&responce.to_string()).unwrap();
    //             println!("{}", responce.params["session_token"].as_str().unwrap());
    //             session_token = String::from(responce.params["session_token"].as_str().unwrap());
    //         },

    //         4 => {
    //             let mut params = <Map<String, Value>>::new();
    //             params.insert(String::from("session_token"), Value::String(session_token.clone()));

    //             let request = Request {
    //                 method: Method::Test,
    //                 params: params
    //             };

    //             let msg = Message::Text(JSON::to_string(&request).unwrap());
    //             socket.write_message(msg);
    //         },
    //         _ => continue
    //     };

    //     // socket.write_message(msg);
    //     // let responce = socket.read_message().unwrap();
    //     // println!("{}", responce);
    // }
    
}
