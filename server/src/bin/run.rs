use std::env;
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

use qpager_lib::*;

use fern::colors::{Color, ColoredLevelConfig};
use log::*;

use threadpool::ThreadPool;

use sqlx::postgres::PgListener;

use dotenv::dotenv;

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
    let (tx_result, rx_result) = channel::<(i32, RequestResult)>();
    let (tx_event, rx_event) = channel::<(i32, Event)>();

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

            match rx_result.try_recv() {
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
            dotenv().ok();

            let database_url = env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set");
            for (id, client) in clients.iter_mut() {
                match client.read_message() {
                    Ok(Message::Text(request)) => {
                        trace!("Request from client {} got: {}", id, request);
                        let request = JSON::from_str::<Request>(&request).unwrap();
                        tx_request.send((*id, request)).unwrap();
                    },
                    Err(TError::ConnectionClosed) => {
                        info!("Client {} disconnected", id);let mut conn = establish_connection();
                    },
                    Err(_) => {
                        
                    },
                    _ => {}
                }
            }
        }
    });

    // Listening for database events
    let db_listening = spawn(move || {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let mut listener = PgListener::new(database_url);
        listener.listen("new_private_message");
        listener.listen("new_chat_message");
        
        loop {
            let event = listener.recv().unwrap();

            trace!("DB event: channel = {}, payload = {}", event.channel(), event.payload());
        }
    });

    // Main logic
    let request_processing = spawn(move || {
        let pool = ThreadPool::new(num_cpus::get());
        let mut tokens = Arc::new(Mutex::new(<HashMap<String, i32>>::new()));

        loop {
            let mut conn = establish_connection();
            let tx_responce = tx_result.clone();
            
            // Block and wait
            let (client_id, request) = rx_request.recv().unwrap();
            
            trace!("Request got for processing: {:?}", &request);
            match request.method {
                Method::SignUp => {
                    pool.execute(move || {
                        match db_queries::sign_up(&request.params, &mut conn) {
                            Ok(_) => {
                                // Process responce
                                let result = RequestResult::Ok(json!({}));
                                tx_responce.send((client_id, result)).unwrap();
                            },
                            Err(err) => {
                                if err.is::<RequestError>() {
                                    let result = RequestResult::Err(*err.downcast::<RequestError>().unwrap());
                                    tx_responce.send((client_id, result)).unwrap();
                                } else {
                                    error!("Method SignUp: Error {}", &err);   
                                }
                            }
                        }
                    })
                },
                Method::LogIn => {
                    let mut tokens = Arc::clone(&tokens);

                    pool.execute(move || {
                        match db_queries::verify_user_credentials(&request.params, &mut conn) {
                            Ok(Some(id)) => {
                                trace!("LogIn method: verified successfully, id = {}", id);
                                let token = generate_session_token();
                                trace!("LogIn method: token generated: {}", token);

                                let mut tokens = tokens.lock().unwrap();
                                tokens.insert(token.clone(), id);

                                let result = RequestResult::Ok(json!({
                                    "session_token": token
                                }));

                                tx_responce.send((client_id, result)).unwrap();
                            },
                            Ok(None) => {
                                trace!("LogIn method: verification failed");
                                let result = RequestResult::Err(RequestError::Auth(AuthError::IncorrectCredentials));
                                tx_responce.send((client_id, result)).unwrap();
                            },
                            Err(err) => {
                                error!("LogIn method: Error: {}", err);
                            }
                        }
                    });
                },
                Method::Test => {}
            }

            // tx_responce.send((client_id, responce)).unwrap();
        }
    });

    listen_for_clietns.join().unwrap();
    handle_clients.join().unwrap();
    request_processing.join().unwrap();
    db_listening.join().unwrap();
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
