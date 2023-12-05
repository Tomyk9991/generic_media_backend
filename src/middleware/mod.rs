use std::future::ready;

use actix_web::{FromRequest, HttpMessage, HttpRequest};
use actix_web::dev::Payload;
use actix_web::error::ErrorUnauthorized;
use actix_web::web::Data;
use hmac::digest::KeyInit;
use hmac::Hmac;
use jwt::VerifyWithKey;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::model::states::app_state::AppState;

pub mod bearer_validator;
pub mod cookie_validator;

#[derive(Debug, Serialize, Deserialize, Clone)]
// must be unique to every user
// otherwise more client's will get the same jwt
pub struct TokenClaims {
    pub id: ObjectId,
}

impl FromRequest for TokenClaims {
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(app_state) = req.app_data::<Data<AppState>>() {
            if let Ok(key) = Hmac::<Sha256>::new_from_slice(app_state.jwt_secret.as_bytes()) {
                if let Some(token) = req.cookie("token") {
                    let claims: Result<TokenClaims, &str> = token.value()
                        .verify_with_key(&key)
                        .map_err(|_| "Invalid token");


                    if let Ok(claims) = claims {
                        let user_id = claims.id;

                        req.extensions_mut()
                            .insert(user_id);

                        return ready(Ok(claims));
                    }
                }
            }
        }

        ready(Err(ErrorUnauthorized("Unauthorized Token")))
    }
}