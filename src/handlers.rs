use super::schema;
use diesel::prelude::*;
use bytes::Bytes;
use std::str;

use super::file::save_file;
use super::models::{NewUser, User};
use super::utils::{generate_api_token, verify, encode_text, decode_text};
use super::Pool;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::models::{APIToken, NewToken, Trove, NewTrove};
use crate::{errors::ServiceError, utils, vars};
use actix_multipart::Multipart;
use actix_web::{web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::dsl::{delete, insert_into};
use diesel::{ExpressionMethods, OptionalExtension};
use schema::api_token::dsl::*;
use schema::users::dsl::*;
use schema::trove::dsl::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::vec::Vec;
use utils::decode_token;

#[derive(Debug, Serialize, Deserialize)]
pub struct InputUser {
    pub password: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputAuthUser {
    pub password: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputApiToken {
    pub token: String,
    pub user_id: i32,
}

// Handler for GET /info
pub async fn info() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json("Hello troveserver!"))
}

// Handler for put /trove
//pub async fn save_trove(db: web::Data<Pool>, item: web::Json<InputAuthUser>) -> Result<HttpResponse, Error> {
//
//}
// Handler for Get /token
pub async fn create_api_token(
    db: web::Data<Pool>,
    item: web::Json<InputAuthUser>,
) -> Result<HttpResponse, Error> {
    // Get User ID from DB
    let user_email = item.email.clone();
    let db_clone = db.clone();
    let user = web::block(move || db_get_user_by_email(db_clone, &user_email))
        .await
        .map_err(|_| HttpResponse::InternalServerError())?;

    if verify(&user.pw_hash, &item.password)? {
        Ok(web::block(move || db_add_api_token(db, user.id))
            .await
            .map(|t| HttpResponse::Created().json(t))
            .map_err(|_| HttpResponse::InternalServerError())?)
    } else {
        Err(ServiceError::AuthenticationError(String::from("Err during authentication")).into())
    }
}

//Handler for GET /token/revoke
pub async fn revoke_api_token(
    db: web::Data<Pool>,
    auth: BearerAuth,
) -> Result<HttpResponse, Error> {
    Ok(web::block(move || db_update_api_token(db, auth))
        .await
        .map(|_| HttpResponse::Created().json("Revoked the access with the API key"))
        .map_err(|_| HttpResponse::InternalServerError())?)
}

// Handler for GET /trove
pub async fn get_trove_by_profile(
    db: web::Data<Pool>,
    auth: BearerAuth,
) -> Result<HttpResponse, Error> {
    // Can unwrap here, since auth middle wear already checks if a user for exists for a given token
    let user = db_get_user_by_api_token(db.clone(), auth).unwrap();
    Ok(
        web::block(move || db_get_latest_trove_by_user_id(db, user.id))
            .await
            .map(|t| HttpResponse::Ok().json(decode_text(t.unwrap_or_default().trove_text)))
            .map_err(|_| HttpResponse::InternalServerError())?,
    )
}

// Handler for PUT /trove

// Handler for GET /users
pub async fn save_trove_by_token(
    db: web::Data<Pool>,
    auth: BearerAuth,
    trove_data: web::Bytes,
) -> Result<HttpResponse, Error> {
    let user = db_get_user_by_api_token(db.clone(), auth).unwrap();
    Ok(web::block(move || db_add_trove_text(db, user.id, str::from_utf8(&trove_data).unwrap()))
        .await
        .map(|_| HttpResponse::Created().json("Saved trove!"))
        .map_err(|_| HttpResponse::InternalServerError())?)
}

#[allow(dead_code)]
// Handler for GET /users/{id}
pub async fn get_user_by_id(
    db: web::Data<Pool>,
    user_id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    Ok(
        web::block(move || db_get_user_by_id(db, user_id.into_inner()))
            .await
            .map(|user| HttpResponse::Ok().json(user))
            .map_err(|_| HttpResponse::InternalServerError())?,
    )
}

#[allow(dead_code)]
pub async fn get_user_by_token(
    auth: BearerAuth,
    db: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    Ok(web::block(move || db_get_user_by_api_token(db, auth))
        .await
        .map(|user| HttpResponse::Ok().json(user))
        .map_err(|_| HttpResponse::InternalServerError())?)
}

// Handler for DELETE /users/{id}
pub async fn delete_user_by_token(
    db: web::Data<Pool>,
    auth: BearerAuth,
) -> Result<HttpResponse, Error> {
    let user = db_get_user_by_api_token(db.clone(), auth).unwrap();
    Ok(web::block(move || delete_single_user(db, user.id))
        .await
        .map(|_| HttpResponse::Ok().json("Deleted user"))
        .map_err(|_| HttpResponse::InternalServerError())?)
}

// Handler for POST /register
pub async fn register_user(
    db: web::Data<Pool>,
    item: web::Json<InputUser>,
) -> Result<HttpResponse, Error> {
    let user_email = item.email.clone();
    let db_clone = db.clone();
    let is_unique = web::block(move || db_count_user_email(db_clone, &user_email))
        .await
        .map(|c| c == 0)
        .map_err(|_| HttpResponse::InternalServerError())?;
    if is_unique {
        Ok(web::block(move || add_single_user(db, item))
            .await
            .map(|user| HttpResponse::Created().json(user))
            .map_err(|_| HttpResponse::InternalServerError())?)
    } else {
        Err(ServiceError::AuthenticationError(String::from("Email already in use")).into())
    }
}

fn db_get_user_by_id(pool: web::Data<Pool>, user_id: i32) -> Result<User, diesel::result::Error> {
    let conn = pool.get().unwrap();
    users.find(user_id).get_result::<User>(&conn)
}

fn db_get_latest_trove_by_user_id(pool: web::Data<Pool>, user_id: i32) -> Result<Option<Trove>, diesel::result::Error> {
    let conn = pool.get().unwrap();
    trove.filter(schema::trove::user_id_fk.eq(user_id)).order_by(schema::trove::id.desc()).first(&conn).optional()
}

fn db_get_user_by_email(
    pool: web::Data<Pool>,
    user_email: &str,
) -> Result<User, diesel::result::Error> {
    let conn = pool.get().unwrap();
    // Distinct, since user email cant be duplicate. Just to make sure to only return one user
    let user = users
        .filter(email.eq(user_email))
        .distinct()
        .get_result(&conn)?;
    Ok(user)
}

pub fn db_get_user_by_api_token(
    pool: web::Data<Pool>,
    requested_api_token: BearerAuth,
) -> Result<User, diesel::result::Error> {
    let conn = pool.get().unwrap();
    let s = decode_token(requested_api_token);
    let user_api_token: APIToken = api_token.filter(token.eq(s)).distinct().get_result(&conn)?;
    db_get_user_by_id(pool, user_api_token.user_id)
}

pub fn db_is_token_revoked(
    pool: web::Data<Pool>,
    requested_api_token: BearerAuth,
) -> Result<bool, diesel::result::Error> {
    // Should be a inner join on api token with users on `user_id_fk`, but have not read up on diesel enough yet
    let conn = pool.get().unwrap();
    let s = decode_token(requested_api_token);
    let user_api_token: Option<APIToken> = api_token
        .filter(token.eq(s).and(revoked.eq(true)))
        .distinct()
        .get_result(&conn)
        .optional()
        .unwrap();
    if user_api_token.is_some() {
        return Ok(true);
    }
    Ok(false)
}

pub fn db_update_api_token(
    pool: web::Data<Pool>,
    requested_api_token: BearerAuth,
) -> Result<bool, diesel::result::Error> {
    // Should be a inner join on api token with users on `user_id_fk`, but have not read up on diesel enough yet
    let conn = pool.get().unwrap();
    let s = utils::decode_token(requested_api_token);
    let token_option: Option<APIToken> = api_token
        .filter(token.eq(s))
        .distinct()
        .get_result(&conn)
        .optional()
        .unwrap();
    if let Some(token_id) = token_option {
        let _ = diesel::update(api_token.filter(schema::api_token::dsl::id.eq(token_id.id)))
            .set(schema::api_token::revoked.eq(true))
            .execute(&conn);
    };
    Ok(true)
}

fn db_count_user_email(
    pool: web::Data<Pool>,
    user_email: &str,
) -> Result<usize, diesel::result::Error> {
    let conn = pool.get().unwrap();
    let user_count = users
        .filter(email.eq(user_email))
        .count()
        .get_result::<i64>(&conn)?;
    Ok(user_count as usize)
}

#[allow(dead_code)]
fn get_all_users(pool: web::Data<Pool>) -> Result<Vec<User>, diesel::result::Error> {
    let conn = pool.get().unwrap();
    let items = users.load::<User>(&conn)?;
    Ok(items)
}

fn db_add_api_token(db: web::Data<Pool>, user_id: i32) -> Result<APIToken, diesel::result::Error> {
    let conn = db.get().unwrap();
    let new_token = NewToken {
        token: &generate_api_token(),
        user_id_fk: user_id,
        created_at: chrono::Local::now().naive_local(),
    };
    let res = insert_into(api_token)
        .values(&new_token)
        .get_result(&conn)?;
    Ok(res)
}

fn add_single_user(
    db: web::Data<Pool>,
    item: web::Json<InputUser>,
) -> Result<User, diesel::result::Error> {
    let conn = db.get().unwrap();
    let hashed_password = utils::hash_password(&item.password).unwrap();
    let new_user = NewUser {
        email: &item.email,
        pw_hash: &hashed_password,
        verified: vars::verify_email(),
        subscribed: false,
        created_at: chrono::Local::now().naive_local(),
        last_payment: chrono::Local::now().naive_local(),
        admin: false,
    };

    let res = insert_into(users).values(&new_user).get_result(&conn)?;
    Ok(res)
}

fn db_add_trove_text(db: web::Data<Pool>, user_id: i32, trove_data: &str) -> Result<Trove, diesel::result::Error> {
    let conn = db.get().unwrap();
    let new_trove = NewTrove {
        trove_text: &encode_text(trove_data.to_string())[..],
        user_id_fk: user_id,
        created_at: chrono::Local::now().naive_local(),
    };
    let res = insert_into(trove).values(&new_trove).get_result(&conn)?;
    Ok(res)
}

fn delete_single_user(db: web::Data<Pool>, user_id: i32) -> Result<usize, diesel::result::Error> {
    let conn = db.get().unwrap();
    let count = delete(users.find(user_id)).execute(&conn)?;
    Ok(count)
}
