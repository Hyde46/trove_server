use crate::errors::ServiceError;
use crate::handlers::{db_get_user_by_api_token, db_is_token_revoked};
use crate::Pool;
use actix_web::web;
use actix_web_httpauth::extractors::bearer::BearerAuth;

pub fn validate_token(token: BearerAuth, pool: web::Data<Pool>) -> Result<bool, ServiceError> {
    match db_is_token_revoked(pool.clone(), token.clone()) {
        Ok(is_revoked) => {
            if is_revoked {
                return Err(ServiceError::AuthenticationError(String::from(
                    "Token has revoked access",
                )));
            }
        }
        Err(_) => return Err(ServiceError::AuthenticationError(String::from("No token"))),
    }
    match db_get_user_by_api_token(pool, token) {
        Ok(_) => {}
        Err(_) => {
            return Err(ServiceError::AuthenticationError(String::from(
                "No user for token",
            )))
        }
    }
    Ok(true)
}
