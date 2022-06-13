use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::accept;
use diesel::prelude::*;

use server::db::connection::establish_connection;
use server::db::schema::chats::dsl::*;
use server::db::models::*;

use serde_json as JSON;

use qpager_lib::{Request, Method};
use server::db::queries as db_queries;

fn main () {
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    
    for stream in server.incoming() {
        spawn (|| {
            let mut conn = establish_connection();
            let mut websocket = accept(stream.unwrap()).unwrap();

            loop {
                let msg = websocket.read_message().unwrap();

                if msg.is_text() {
                    let msg: Request = JSON::from_str(msg.to_text().unwrap()).unwrap();

                    match msg.method {
                        Method::SignUp => { 
                            db_queries::sign_up(msg.params, &mut conn).unwrap()
                        },
                        _ => continue
                    }
                }
            }
        });
    }
}
