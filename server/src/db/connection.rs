use std::env;

use diesel::prelude::*;
use diesel::pg::PgConnection;

use dotenv::dotenv;

use openssl::rand;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn generate_session_token() -> String {
    let mut token = [0; 32];
    rand::rand_bytes(&mut token).unwrap();

    hex::encode(&token)
}