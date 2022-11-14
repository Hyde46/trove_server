use crate::{errors::ServiceError, vars};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use base64::decode;
use pbkdf2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use rand::Rng;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789";
const API_TOKEN_LEN: usize = 30;

pub fn hash_password(password: &str) -> Result<String, ServiceError> {
    let salt = SaltString::generate(&mut OsRng);
    // Hash password to PHC string ($pbkdf2-sha256$...)
    let password_hash = Pbkdf2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    // Verify password against PHC string
    let parsed_hash = PasswordHash::new(&password_hash).unwrap();
    Ok(parsed_hash.to_string())
}

pub fn verify(hash: &str, password: &str) -> Result<bool, ServiceError> {
    let parsed_hash = PasswordHash::new(hash).unwrap();
    return Ok(Pbkdf2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok());
}

pub fn generate_api_token() -> String {
    let mut rng = rand::thread_rng();
    (0..API_TOKEN_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn decode_token(credentials: BearerAuth) -> String {
    let decoded_token_buffer: Vec<u8> = decode(credentials.token()).unwrap();
    let s = match std::str::from_utf8(&decoded_token_buffer) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    s.to_string()
}
