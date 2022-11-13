use crate::{errors::ServiceError, vars};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use argonautica::{Hasher, Verifier};
use base64::decode;
use rand::Rng;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789";
const API_TOKEN_LEN: usize = 30;

pub fn hash_password(password: &str) -> Result<String, ServiceError> {
    Hasher::default()
        .with_password(password)
        .with_secret_key(vars::secret_key().as_str())
        .hash()
        .map_err(|_| ServiceError::AuthenticationError(String::from("Could not hash password")))
}

pub fn verify(hash: &str, password: &str) -> Result<bool, ServiceError> {
    Verifier::default()
        .with_hash(hash)
        .with_password(password)
        .with_secret_key(vars::secret_key().as_str())
        .verify()
        .map_err(|_| ServiceError::AuthenticationError(String::from("Unauthorized")))
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
