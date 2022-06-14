use std::error::Error;

use diesel::{PgConnection, RunQueryDsl, QueryDsl};
use diesel::result::{QueryResult, Error as DieselError};
use diesel::ExpressionMethods;

use super::models::*;
use super::schema::users;

use serde_json as JSON;
use JSON::{Map, Value};

use openssl::rand;
use openssl::pkcs5;
use openssl::hash::MessageDigest;

use qpager_lib::*;

fn is_signed_up(params: &Map<String, Value>, conn: &mut PgConnection) -> Result<bool, Box<dyn Error>> {
    let login = params["login"].to_string();

    match users::table.filter(users::login.eq(login)).first::<User>(conn) {
        Ok(_) => {
            return Ok(true);
        },
        Err(DieselError::NotFound) => {
            return Ok(false);
        },
        Err(err) => {
            return Err(Box::new(err));
        }
    }
}

pub fn sign_up(params: &Map<String, Value>, conn: &mut PgConnection) -> Result<(), Box<dyn Error>> {
    // match is_signed_up(params, conn) {
    //     Ok(true) => {
    //         return Err(Box::new(RequestError::Auth(AuthError::AlreadySignedUp)))
    //     }
    // }

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
    
    diesel::insert_into(users::table)
        .values(new_user)
        .execute(conn)?;

    Ok(())
}

pub fn verify_user_credentials(params: &Map<String, Value>, conn: &mut PgConnection) -> Result<(bool, i32), Box<dyn Error>> {
    let login = params["login"].to_string();
    let password = params["password"].to_string();

    let user = users::table.filter(users::login.eq(login)).first::<User>(conn)?;

    let mut hashed_password = [0; 256];

    pkcs5::pbkdf2_hmac(
        password.as_bytes(),
        &hex::decode(user.salt).unwrap(),
        1000,
        MessageDigest::sha256(),
        &mut hashed_password
    )
        .unwrap();
    
    if user.password == hex::encode(&hashed_password) {
        Ok((true, user.id))
    } else {
        Ok((false, 0))
    }
}

// fn get_salt(params: &Map<String, Value>, conn: &mut PgConnection) -> String {
//     users::table.filter(users::login.eq(login)).select(users::salt).first(conn);
// }