use actix_files::NamedFile;
use actix_multipart::Multipart;

use actix_web::{get, HttpRequest, HttpResponse, post};
use actix_web::web::{Data, Json, Path, Query};
use serde::Deserialize;
use uuid::Uuid;
use crate::api::shared::{GETError, UploadError};
use crate::database::repositories::SelectRepository;
use crate::middleware::TokenClaims;
use crate::model::states::app_state::AppState;
use crate::model::user::{SelectUserById, SelectUserByName};
use crate::utils::{read_files_in_directory, UploadOptions, validate_stories, write_files_in_directory};


#[derive(Debug, Deserialize)]
pub struct QueryInfo {
    //pagination
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[get("/{user_name}")]
pub async fn list(user_name: Path<String>, state: Data<AppState>, query: Query<QueryInfo>, claims: TokenClaims) -> actix_web::Result<Json<Vec<String>>, GETError> {
    let query = query.into_inner();
    let user_name = user_name.into_inner();

    let user = state.db.user().select(&SelectUserByName { username: &user_name }).await?;
    let requesting_user = state.db.user().select(&SelectUserById { id: claims.id }).await?;
    let are_friends = state.db.friendship().select(&SelectUserById { id: claims.id })
        .await?
        .iter()
        .filter(|friendship| friendship.friend_a == user.id || friendship.friend_b == user.id)
        .count() > 0;

    if !are_friends && user.id != claims.id && !requesting_user.is_admin() {
        return Err(GETError::Unauthorized);
    }

    let path = format!("{}{}", state.data_directory, user_name);
    let all_files = read_files_in_directory(&path, false)?;

    return Ok(Json(
        all_files
            .iter()
            .skip(query.offset.unwrap_or(0))
            .take(query.limit.unwrap_or(all_files.len()))
            .map(|dir| dir.file_name().into_string())
            .filter_map(|f| f.ok())
            .collect::<Vec<String>>()
    ));
}

#[get("stories/{user_name}")]
pub async fn stories(user_name: Path<String>, state: Data<AppState>, claims: TokenClaims) -> Result<Json<Vec<String>>, GETError> {
    let user_name = user_name.into_inner();
    let user = state.db.user().select(&SelectUserByName { username: &user_name }).await?;
    let requesting_user = state.db.user().select(&SelectUserById { id: claims.id }).await?;

    let are_friends = state.db.friendship().select(&SelectUserById { id: claims.id })
        .await?
        .iter()
        .filter(|friendship| friendship.friend_a == user.id || friendship.friend_b == user.id)
        .count() > 0;

    if !are_friends && user.id != claims.id && !requesting_user.is_admin() {
        return Err(GETError::Unauthorized);
    }

    let path = format!("{}{}/stories",  state.data_directory, user.name);
    // create or do nothing, when created
    std::fs::create_dir_all(&path)?;

    let all_files = read_files_in_directory(&path, true)?;

    return Ok(Json(
        all_files
            .iter()
            .map(|dir| dir.file_name().into_string())
            .filter_map(|f| f.ok())
            .collect::<Vec<String>>()
    ));
}

#[get("stories/{user_name}/{path}")]
pub async fn story(user_name: Path<(String, String)>, state: Data<AppState>, claims: TokenClaims) -> actix_web::Result<NamedFile, GETError> {
    let (user_name, media_file_name) = user_name.into_inner();
    let user = state.db.user().select(&SelectUserByName { username: &user_name }).await?;
    let requesting_user = state.db.user().select(&SelectUserById { id: claims.id }).await?;

    let are_friends = state.db.friendship().select(&SelectUserById { id: claims.id })
        .await?
        .iter()
        .filter(|friendship| friendship.friend_a == user.id || friendship.friend_b == user.id)
        .count() > 0;

    if !are_friends && user.id != claims.id && !requesting_user.is_admin() {
        return Err(GETError::Unauthorized);
    }

    let path = format!("{}{}/stories",  state.data_directory, user.name);
    // create or do nothing, when created
    std::fs::create_dir_all(&path)?;


    let all_files = read_files_in_directory(&path, false)?;
    let all_files = all_files
        .iter()
        .map(|dir| dir.file_name().into_string())
        .filter_map(|f| f.ok())
        .collect::<Vec<String>>();

    if all_files.contains(&media_file_name) {
        Ok(NamedFile::open(format!("{}{}/stories/{}",  state.data_directory, user.name, media_file_name))?)
    } else {
        Err(GETError::CantRead)
    }
}

#[post("stories/{user_name}")]
pub async fn upload_story(user_name: Path<String>, payload: Multipart, req: HttpRequest, claims: TokenClaims, state: Data<AppState>) -> Result<HttpResponse, UploadError> {
    let user_name = user_name.into_inner();

    let user = state.db.user().select(&SelectUserByName { username: &user_name }).await?;
    let requesting_user = state.db.user().select(&SelectUserById { id: claims.id }).await?;

    if user.id != claims.id && !requesting_user.is_admin() {
        return Err(UploadError::Unauthorized);
    }

    let upload_options = UploadOptions {
        max_file_count: 5,
        max_file_size: 100_000_000, // 100mb
        legal_file_types: vec![
            mime::IMAGE_JPEG,
            mime::IMAGE_PNG,
            #[allow(clippy::unwrap_used)]
                "video/mp4".parse().unwrap(),
            #[allow(clippy::unwrap_used)]
                "video/quicktime".parse().unwrap()
        ]
    };

    write_files_in_directory(&req, payload, upload_options, &state, |inner_state, file_name| {
        format!("{}{}/stories/{}_{}", inner_state.data_directory, user.name, Uuid::new_v4(), file_name)
    }).await?;

    validate_stories(&format!("{}{}/stories", state.data_directory, user.name))?;

    Ok(HttpResponse::Ok().into())
}

#[post("/{user_name}")]
pub async fn upload(payload: Multipart, req: HttpRequest, user_name: Path<String>, claims: TokenClaims, state: Data<AppState>) -> Result<HttpResponse, UploadError> {
    let user_name = user_name.into_inner();

    let user = state.db.user().select(&SelectUserByName { username: &user_name }).await?;
    let requesting_user = state.db.user().select(&SelectUserById { id: claims.id }).await?;

    if user.id != claims.id && !requesting_user.is_admin() {
        return Err(UploadError::Unauthorized);
    }

    let upload_options = UploadOptions {
        max_file_count: 100,
        max_file_size: 100_000_000, // 100mb
        legal_file_types: vec![
            mime::IMAGE_JPEG,
            mime::IMAGE_PNG,
            #[allow(clippy::unwrap_used)]
            "video/mp4".parse().unwrap(),
            #[allow(clippy::unwrap_used)]
            "video/quicktime".parse().unwrap()
        ]
    };

    write_files_in_directory(&req, payload, upload_options, &state, |inner_state, file_name| {
        format!("{}{}/{}_{}", inner_state.data_directory, user.name, Uuid::new_v4(), file_name)
    }).await?;

    Ok(HttpResponse::Ok().into())
}