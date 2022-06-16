use std::hash::Hash;
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{channel, TryRecvError};

use tungstenite::{accept, client};
use tungstenite::protocol::{Message, WebSocket};
use tungstenite::error::Error as TError;

use server::db::connection::{establish_connection, generate_session_token};
use server::db::queries as db_queries;

use serde_json as JSON;
use JSON::{Value, json};

use qpager_lib::{Request, Method, Params, Responce};

use fern::colors::{Color, ColoredLevelConfig};
use log::*;

fn init_logger() {
    let colors = ColoredLevelConfig::new()
        .info(Color::BrightBlue)
        .debug(Color::Magenta)
        .error(Color::Red)
        .warn(Color::Yellow);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .level(log::LevelFilter::Off)
        .level_for("run", log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply().unwrap(); 
}

fn main () {
    init_logger();
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    // let tokens = Arc::new(Mutex::new(<HashMap<String, i32>>::new()));


    let (tx_incoming_client, rx_incoming_client) = channel::<WebSocket<TcpStream>>();
    
    // Listen for new client connections
    let listen_for_clietns = spawn (move || {
        for stream in server.incoming() {
            info!("client connected");
            let mut tcp_stream = stream.unwrap();
            tcp_stream.set_nonblocking(true);
            let mut client_socket = accept(tcp_stream).unwrap();
            tx_incoming_client.send(client_socket).unwrap();
        }
    });
    
    let (tx_request, rx_request) = channel::<(i32, Request)>();
    let (tx_responce, rx_responce) = channel::<(i32, Responce)>();

    // Manage client connections
    let handle_clients = spawn(move || {
        let mut id_counter = 1;
        let mut clients = <HashMap<i32, WebSocket<TcpStream>>>::new();
        loop {
            match rx_incoming_client.try_recv() {
                Ok(client) => {
                    trace!("New client with id = {} connected!", id_counter);
                    clients.insert(id_counter, client);
                    id_counter += 1;
                },
                Err(TryRecvError::Empty) => {
                    // continue
                },
                Err(TryRecvError::Disconnected) => {
                    panic!("rx_incoming_client disconnected");
                }
            }

            match rx_responce.try_recv() {
                Ok((id, responce)) => {
                    let responce = JSON::to_string(&responce).unwrap();
                    trace!("Responce for client {} received for sending: {}", id, responce);
                    trace!("Responce for client {} sent: {}", id, responce);
                    clients.get_mut(&id).unwrap().write_message(Message::Text(responce)).unwrap();
                },
                Err(TryRecvError::Empty) => {
                    // continue
                },
                Err(TryRecvError::Disconnected) => {
                    panic!("rx_incoming_client disconnected");
                }
            }

            for (id, client) in clients.iter_mut() {
                match client.read_message() {
                    Ok(Message::Text(request)) => {
                        trace!("Request from client {} got: {}", id, request);
                        let request = JSON::from_str::<Request>(&request).unwrap();
                        tx_request.send((*id, request)).unwrap();
                    },
                    Err(TError::ConnectionClosed) => {
                        info!("Client {} disconnected", id);
                    },
                    Err(_) => {
                        
                    },
                    _ => {}
                }
            }
        }
    });

    // Main logic
    let request_processing = spawn(move || {
        loop {
            let (client_id, request) = rx_request.recv().unwrap();
            let request = JSON::to_string(&request).unwrap();
            trace!("Request from client {} received for processing: {}", client_id, request);
            trace!("Request from client {} proceed successfully: {}", client_id, request);
            let responce = Responce {
                result: true,
                params: json!({})
            };
            // trace!("Responce sent to se {}: {}", client_id, responce);
            tx_responce.send((client_id, responce)).unwrap();
        }
    });

    listen_for_clietns.join();
    handle_clients.join();
    request_processing.join();
        // let tokens = Arc::clone(&tokens);

        // spawn (move || {
        //     let mut conn = establish_connection();
        //     let mut websocket = accept(stream.unwrap()).unwrap();
        //     loop {
        //         let msg = websocket.read_message().unwrap();

        //         if msg.is_text() {
        //             log::trace!("Request got: {}", msg.to_text().unwrap());
        //             let msg: Request = JSON::from_str(msg.to_text().unwrap()).unwrap();

        //             match msg.method {
        //                 Method::SignUp => { 
        //                     db_queries::sign_up(&msg.params, &mut conn).unwrap()
        //                 },
        //                 Method::LogIn => {
        //                     let (verified, id) = db_queries::verify_user_credentials(&msg.params, &mut conn).unwrap();
        //                     log::debug!("verified = {}, id = {}", verified, id);
        //                     if verified {
        //                         let token = generate_session_token();
        //                         log::debug!("token: {}", token);

        //                         let mut tokens = tokens.lock().unwrap();
        //                         tokens.insert(token.clone(), id);

        //                         let mut params = Params::new();
        //                         params.insert(String::from("session_token"), Value::String(token));

        //                         let responce = Responce {
        //                             result: true,
        //                             params: params
        //                         };

        //                         websocket.write_message(Message::Text(JSON::to_string(&responce).unwrap())).unwrap();
        //                     }
        //                 },
        //                 Method::Test => {
        //                     let session_token = msg.params["session_token"].as_str().unwrap();
        //                     println!("{}", session_token);

        //                     let tokens = tokens.lock().unwrap();
        //                     if let Some(id) = tokens.get(session_token) {
        //                         log::info!("got token for user with id = {}", id);
        //                     } else {
        //                         log::info!("incorrect token");
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // });
    
}
