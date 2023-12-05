use actix_web::dev::{ServiceRequest};
use actix_web::{Error};
use actix_web::error::ErrorUnauthorized;
use actix_web::web::{Data};
use crate::database::repositories::SelectRepository;
use crate::middleware::TokenClaims;
use crate::model::states::app_state::AppState;
use crate::model::user::SelectUserById;


pub async fn validator(req: ServiceRequest, claims: TokenClaims) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    if let Some(app_state) = req.app_data::<Data<AppState>>() {
        if app_state.db.user().select(&SelectUserById { id: claims.id }).await.is_ok() {
            return Ok(req);
        }
    }

    Err((ErrorUnauthorized("Unauthorized"), req))
}