use std::fmt::{Display, Formatter};
use actix_web::{get, HttpResponse, Responder, ResponseError};
use actix_web::cookie::Cookie;
use actix_web::cookie::time::Duration;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web_httpauth::extractors::basic::BasicAuth;
use argon2::password_hash::Error;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use hmac::digest::{InvalidLength, KeyInit};
use hmac::Hmac;
use jwt::SignWithKey;
use sha2::Sha256;
use crate::api::shared::GETError;
use crate::database::repositories::SelectRepository;
use crate::middleware::TokenClaims;
use crate::migrations::DatabaseMigration;
use crate::migrations::user_migration::UserMigration;
use crate::model::states::app_state::AppState;
use crate::model::user::{SelectUserById, SelectUserByName};
use crate::utils::version::Version;

#[derive(Debug)]
pub enum AuthError {
    InvalidLength,
    Unauthorized,
}

impl Display for AuthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            AuthError::InvalidLength => String::from("Invalid length provided"),
            AuthError::Unauthorized => String::from("Unauthorized"),
        })
    }
}

impl From<InvalidLength> for AuthError {
    fn from(_: InvalidLength) -> Self {
        AuthError::InvalidLength
    }
}

impl From<jwt::Error> for AuthError {
    fn from(_: jwt::Error) -> Self {
        AuthError::InvalidLength
    }
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(_: Error) -> Self { AuthError::Unauthorized }
}


impl ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthError::InvalidLength => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::Unauthorized => StatusCode::UNAUTHORIZED
        }
    }
}


#[get("/authCookie")]
async fn cookie_auth(state: Data<AppState>, credentials: BasicAuth) -> Result<impl Responder, AuthError> {
    let jwt_secret: Hmac<Sha256> = Hmac::new_from_slice(
        state.jwt_secret.as_bytes()
    )?;

    let user_name = credentials.user_id();
    let password = credentials.password();

    if let Some(pass) = password {
        if let Ok(found_user) = state.db.user().select(&SelectUserByName { username: user_name }).await {
            let parsed_hash = PasswordHash::new(&found_user.password_hash)?;

            // verified. correct password
            if Argon2::default().verify_password(pass.as_bytes(), &parsed_hash).is_ok() {
                let claims = TokenClaims { id: found_user.id };
                let token_str = claims.sign_with_key(&jwt_secret)?;

                let cookie = Cookie::build("token", token_str)
                    .max_age(Duration::days(730))
                    .http_only(true)
                    .finish();

                return Ok(HttpResponse::Ok()
                    .cookie(cookie)
                    .finish()
                );
            }
        }
    }

    Err::<HttpResponse, AuthError>(AuthError::Unauthorized)
}

#[get("/authCookie/revalidate")]
async fn cookie_revalidate(app_state: Data<AppState>, claims: TokenClaims) -> Result<impl Responder, AuthError> {
    if app_state.db.user().select(&SelectUserById { id: claims.id }).await.is_ok() {
        return Ok(HttpResponse::Ok().into());
    }

    Err::<HttpResponse, AuthError>(AuthError::Unauthorized)
}

#[get("/logout")]
pub async fn logout_cookie(_: TokenClaims) -> Result<impl Responder, AuthError> {
    let cookie = Cookie::build("token", "")
        .http_only(false)
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(cookie)
        .finish()
    )
}

#[get("/migrate")]
pub async fn migrate(app_state: Data<AppState>) -> Result<impl Responder, GETError> {
    let user_migration = UserMigration { version: Version { version: 1.0 } };

    match user_migration.migrate(&app_state.db).await {
        Ok(_) => {}
        Err(err) => {
            log::error!("{err:?}");
            std::process::exit(1);
        }
    }

    Ok(HttpResponse::Ok())
}
