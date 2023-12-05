use actix_web::dev::ServiceRequest;
use actix_web::{Error, HttpMessage};
use actix_web::web::Data;
use actix_web_httpauth::extractors::{AuthenticationError, bearer};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use hmac::digest::KeyInit;
use hmac::Hmac;
use jwt::VerifyWithKey;
use sha2::Sha256;
use crate::database::repositories::SelectRepository;
use crate::middleware::TokenClaims;
use crate::model::states::app_state::AppState;
use crate::model::user::{SelectUserById};


#[allow(clippy::expect_used, dead_code)]
pub async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
        if let Ok(key) = Hmac::<Sha256>::new_from_slice(jwt_secret.as_bytes()) {
            let token_string = credentials.token();

            let claims: Result<TokenClaims, &str> = token_string
                .verify_with_key(&key)
                .map_err(|_| "Invalid token");

            if let Ok(value) = claims {
                if let Some(app_state) = req.app_data::<Data<AppState>>() {
                    if app_state.db.user().select(&SelectUserById { id: value.id }).await.is_ok() {
                        // user found, everything is nice
                        req.extensions_mut().insert(value);
                        return Ok(req);
                    }
                }
            }
        }
    }

    let config = req.app_data::<bearer::Config>().cloned().unwrap_or_default().scope("");
    Err((AuthenticationError::from(config).into(), req))
}