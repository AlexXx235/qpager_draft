use std::net::TcpListener;
use std::thread::spawn;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};

use tungstenite::accept;
use tungstenite::protocol::Message;

use server::db::connection::{establish_connection, generate_session_token};
use server::db::queries as db_queries;

use serde_json as JSON;
use JSON::Value;

use qpager_lib::{Request, Method, Params, Responce};

fn main () {
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    let tokens = Arc::new(Mutex::new(<HashMap<String, i32>>::new()));
    
    for stream in server.incoming() {
        let tokens = Arc::clone(&tokens);

        spawn (move || {
            let mut conn = establish_connection();
            let mut websocket = accept(stream.unwrap()).unwrap();
            loop {
                let msg = websocket.read_message().unwrap();

                if msg.is_text() {
                    println!("{}", msg.to_text().unwrap());
                    let msg: Request = JSON::from_str(msg.to_text().unwrap()).unwrap();

                    match msg.method {
                        Method::SignUp => { 
                            db_queries::sign_up(&msg.params, &mut conn).unwrap()
                        },
                        Method::LogIn => {
                            let (verified, id) = db_queries::verify_user_credentials(&msg.params, &mut conn).unwrap();
                            log::debug!("verified = {}, id = {}", verified, id);
                            if verified {
                                let token = generate_session_token();
                                log::debug!("token: {}", token);

                                let mut tokens = tokens.lock().unwrap();
                                tokens.insert(token.clone(), id);

                                let mut params = Params::new();
                                params.insert(String::from("token"), Value::String(token));

                                let responce = Responce {
                                    result: true,
                                    params: params
                                };

                                websocket.write_message(Message::Text(JSON::to_string(&responce).unwrap())).unwrap();
                            }
                        },
                        _ => continue
                    }
                }
            }
        });
    }
}
