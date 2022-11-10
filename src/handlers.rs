use super::models::{NewUser, User};
use super::schema::users::dsl::*;
use super::Pool;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::{errors::ServiceError, utils, vars};
use actix_web::Responder;
use actix_web::{web, Error, HttpResponse};
use diesel::dsl::{delete, insert_into};
use diesel::ExpressionMethods;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[derive(Debug, Serialize, Deserialize)]
pub struct InputUser {
    pub first_name: String,
    pub last_name: String,
    pub password: String,
    pub email: String,
}

// Handler for GET /users
pub async fn get_users(db: web::Data<Pool>) -> Result<HttpResponse, Error> {
    Ok(web::block(move || get_all_users(db))
        .await
        .map(|user| HttpResponse::Ok().json(user))
        .map_err(|_| HttpResponse::InternalServerError())?)
}

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

// Handler for DELETE /users/{id}
pub async fn delete_user(
    db: web::Data<Pool>,
    user_id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    Ok(
        web::block(move || delete_single_user(db, user_id.into_inner()))
            .await
            .map(|user| HttpResponse::Ok().json(user))
            .map_err(|_| HttpResponse::InternalServerError())?,
    )
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

fn get_all_users(pool: web::Data<Pool>) -> Result<Vec<User>, diesel::result::Error> {
    let conn = pool.get().unwrap();
    let items = users.load::<User>(&conn)?;
    Ok(items)
}

fn add_single_user(
    db: web::Data<Pool>,
    item: web::Json<InputUser>,
) -> Result<User, diesel::result::Error> {
    let conn = db.get().unwrap();
    let hashed_password = utils::hash_password(&item.password).unwrap();
    let new_user = NewUser {
        first_name: &item.first_name,
        last_name: &item.last_name,
        email: &item.email,
        pw_hash: &hashed_password,
        verified: vars::verify_email(),
        created_at: chrono::Local::now().naive_local(),
    };

    let res = insert_into(users).values(&new_user).get_result(&conn)?;
    Ok(res)
}

fn delete_single_user(db: web::Data<Pool>, user_id: i32) -> Result<usize, diesel::result::Error> {
    let conn = db.get().unwrap();
    let count = delete(users.find(user_id)).execute(&conn)?;
    Ok(count)
}
