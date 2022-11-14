use crate::schema::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub pw_hash: String,
    pub verified: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub email: &'a str,
    pub pw_hash: &'a str,
    pub verified: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct APIToken {
    pub id: i32,
    pub token: String,
    pub user_id: i32,
    pub revoked: bool,
    pub created_at: chrono::NaiveDateTime,
}
#[derive(Insertable, Debug)]
#[table_name = "api_token"]
pub struct NewToken<'a> {
    pub token: &'a str,
    pub user_id_fk: i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Default)]
pub struct Trove {
    pub id: i32,
    pub trove_text: String,
    pub user_id: i32,
    pub created_at: chrono::NaiveDateTime,
}
#[derive(Insertable, Debug)]
#[table_name = "trove"]
pub struct NewTrove<'a> {
    pub trove_text: &'a str,
    pub user_id_fk: i32,
    pub created_at: chrono::NaiveDateTime,
}
