use diesel::prelude::*;
use super::schema::users;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub login: String,
    pub password: String,
    pub salt: String
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub login: &'a str,
    pub password: &'a str,
    pub salt: &'a str
}

#[derive(Queryable)]
pub struct Chat {
    pub id: i32,
    pub name: String,
}