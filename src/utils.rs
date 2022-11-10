use crate::{errors::ServiceError, vars};
use argonautica::{Hasher, Verifier};

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
        .map_err(|_| ServiceError::AuthenticationError(String::from("Could not verify password")))
}
