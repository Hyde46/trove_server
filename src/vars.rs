use dotenv::dotenv;
use std::env::var;

pub fn database_url() -> String {
    dotenv().ok();
    var("DATABASE_URL").expect("DATABASE_URL must be set")
}

pub fn secret_key() -> String {
    dotenv().ok();
    var("SECRET_KEY").unwrap_or_else(|_| "0123".repeat(8))
}

pub fn verify_email() -> bool {
    dotenv().ok();
    let env_var = var("VERIFY_USER").unwrap_or_else(|_| "true".to_string());
    env_var.parse::<bool>().unwrap()
}

pub fn port() -> String {
    dotenv().ok();
    var("ACTIX_PORT").unwrap_or_else(|_| "8080".repeat(8))
}
