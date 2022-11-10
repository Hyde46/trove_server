#[macro_use]
extern crate diesel;

use actix_web::{dev::ServiceRequest, middleware, web, App, Error, HttpServer};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::middleware::HttpAuthentication;
use argonautica::{Hasher, Verifier};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod auth;
mod errors;
mod handlers;
mod models;
mod schema;
mod utils;
mod vars;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, Error> {
    let config = req
        .app_data::<Config>()
        .map(|data| data.get_ref().clone())
        .unwrap_or_else(Default::default);
    match auth::validate_token(credentials.token()) {
        Ok(res) => {
            if res {
                Ok(req)
            } else {
                Err(AuthenticationError::from(config).into())
            }
        }
        Err(_) => Err(AuthenticationError::from(config).into()),
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=debug");

    let manager = ConnectionManager::<PgConnection>::new(vars::database_url());
    let pool: Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    // Start http server
    HttpServer::new(move || {
        //let auth = HttpAuthentication::bearer(validator);
        App::new()
            //.wrap(auth)
            .data(pool.clone())
            .route("/register", web::post().to(handlers::register_user))
        //.route("/users", web::get().to(handlers::get_users))
        //.route("/users/{id}", web::get().to(handlers::get_user_by_id))
        //.route("/users/{id}", web::delete().to(handlers::delete_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
