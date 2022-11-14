#[macro_use]
extern crate diesel;
extern crate base64;
extern crate rand;

use actix_web::middleware::Logger;
use actix_web::{dev::ServiceRequest, web, App, Error, HttpServer};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::middleware::HttpAuthentication;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod auth;
mod errors;
mod file;
mod handlers;
mod models;
mod schema;
mod utils;
mod vars;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
    db: web::Data<Pool>,
) -> Result<ServiceRequest, Error> {
    match auth::validate_token(credentials, db) {
        Ok(res) => {
            if res {
                Ok(req)
            } else {
                Err(errors::ServiceError::AuthenticationError(String::from(
                    "Invalid token. Not found",
                ))
                .into())
            }
        }
        Err(_) => {
            Err(errors::ServiceError::AuthenticationError(String::from("Invalid token")).into())
        }
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

    let port = vars::port();
    let uri = format!("127.0.0.1:{}", port);
    // Start http server
    HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(|req, cred| {
            let db = web::Data::new(
                req.app_data::<Pool>()
                    .expect("Failed to extract DatabaseConnection from ServiceRequest")
                    .get_ref()
                    .clone(),
            );
            validator(req, cred, db)
        });
        App::new()
            .wrap(Logger::default())
            .data(pool.clone())
            .route("/info", web::get().to(handlers::info))
            .route("/register", web::post().to(handlers::register_user))
            .route("/token/new", web::get().to(handlers::create_api_token))
            .service(
                web::scope("/v1")
                    .wrap(auth)
                    .route("/trove", web::get().to(handlers::get_trove_by_profile))
                    .route("/trove", web::put().to(handlers::save_trove_by_token))
                    .route("/user", web::delete().to(handlers::delete_user_by_token))
                    .route("/token/revoke", web::get().to(handlers::revoke_api_token)),
            )
    })
    .bind(uri)?
    .run()
    .await
}
