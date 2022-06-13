use tungstenite::Message;

use diesel::{PgConnection, RunQueryDsl, QueryDsl};
use diesel::result::Error as DbError;
use diesel::ExpressionMethods;

use super::models::*;
use super::schema::users;

use serde_json as JSON;
use JSON::{Map, Value};

use openssl::rand;
use openssl::pkcs5;
use openssl::hash::MessageDigest;

// trait DbQuery {
//     fn query(params: Map<String, Value>, conn: PgConnection) -> Result<_, _>;
// }

// pub struct IsSignedUp {}

// impl DbQuery for IsSignedUp {
//     fn query(params: Map<String, Value>, conn: PgConnection) -> Result<_, _> {
//         Ok(())
//     }
// }

pub fn is_signed_up(params: Map<String, Value>, conn: &mut PgConnection) -> bool {
    let login = params["login"].to_string();

    let user = users::table
        .filter(users::login.eq(login))
        .first::<User>(conn)
        .unwrap();
    println!("{}", user.login);
    true
}

pub fn sign_up(params: Map<String, Value>, conn: &mut PgConnection) -> Result<(), DbError>{
    let login = params["login"].to_string();
    let password = params["password"].to_string();

    let mut salt = [0; 32];
    rand::rand_bytes(&mut salt).unwrap();

    let mut hashed_password = [0; 256];

    pkcs5::pbkdf2_hmac(
        password.as_bytes(),
        &salt,
        1000,
        MessageDigest::sha256(),
        &mut hashed_password
    )
        .unwrap();

    let new_user = NewUser {
        login: login.as_str(),
        password: &hex::encode(hashed_password),
        salt: &hex::encode(salt)
    };
    
    let inserted_rows_count = diesel::insert_into(users::table)
        .values(new_user)
        .execute(conn)?;

    println!("{}", inserted_rows_count);

    Ok(())
}