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

use log::*;

fn is_signed_up(params: &Value, conn: &mut PgConnection) -> Result<bool, Box<dyn Error>> {
    let login = params["login"].as_str().unwrap();
    trace!("Is signed up method: login = {}", &login);
    match users::table.filter(users::login.eq(&login)).first::<User>(conn) {
        Ok(_) => {
            trace!("Is signed up: {} is already signed up", &login);
            return Ok(true);
        },
        Err(DieselError::NotFound) => {
            trace!("Is signed up: {} is not signed up yet", &login);
            return Ok(false);
        },
        Err(err) => {
            error!("Is signed up: Error: {}", &err);
            return Err(Box::new(err));
        }
    }
}

pub fn sign_up(params: &Value, conn: &mut PgConnection) -> Result<(), Box<dyn Error>> {
    let is_signed_up = is_signed_up(params, conn)?;
    if is_signed_up {
        return Err(Box::new(RequestError::Auth(AuthError::AlreadySignedUp)));
    }

    let login = params["login"].as_str().unwrap();
    let password = params["password"].as_str().unwrap();

    trace!("Sign up: login = {}, password = {}", &login, &password);

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
        login: login,
        password: &hex::encode(hashed_password),
        salt: &hex::encode(salt)
    };

    trace!("Sign up: new user: {:?}", new_user);
    
    diesel::insert_into(users::table)
        .values(new_user)
        .execute(conn)?;

    Ok(())
}

pub fn verify_user_credentials(params: &Value, conn: &mut PgConnection) -> Result<Option<i32>, Box<dyn Error>> {
    let login = params["login"].as_str().unwrap();
    let password = params["password"].as_str().unwrap();

    trace!("Verify user credentials: login = {}, password = {}", login, password);

    let user = users::table.filter(users::login.eq(login)).first::<User>(conn)?;

    trace!("Verify user credentials: user = {:?}", &user);

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
        trace!("Verify user credentials: success");
        Ok(Some(user.id))
    } else {
        trace!("Verify user credentials: failed");
        Ok(None)
    }
}

// fn get_salt(params: &Map<String, Value>, conn: &mut PgConnection) -> String {
//     users::table.filter(users::login.eq(login)).select(users::salt).first(conn);
// }