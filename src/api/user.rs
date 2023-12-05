use std::fs::File;
use std::io::{BufReader, Write};
use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{get, HttpRequest, HttpResponse, post, put, Responder};
use actix_web::web::{Data, Json, Path};
use futures_util::{TryFutureExt};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use crate::api::shared::{GETError, UploadError};
use crate::api::shared::list::List;

use crate::database::repositories::{InsertRepository, SelectRepository, UpdateRepository};
use crate::middleware::TokenClaims;
use crate::model::friend::Friend;
use crate::model::friendship::Friendship;
use crate::model::states::app_state::AppState;
use crate::model::user::{CreateUser, CreateUserError, SelectUserById, SelectUserByName, UpdateUser};
use crate::utils::{UploadOptions, write_files_in_directory};

#[derive(Serialize)]
struct UserNoPassword {
    username: String,
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    amount_posts: usize,
    description: String,
    is_bot: bool
}

#[derive(Debug, Deserialize)]
pub struct UpdateDescription {
    description: String
}


#[post("/user")]
async fn create_user(app_state: Data<AppState>, body: Json<CreateUser>) -> Result<impl Responder, CreateUserError> {
    let create_user: CreateUser = body.into_inner();
    let repo = app_state.db.user();
    let created_user = repo.insert(create_user).await?;

    // todo check if name collides with naming policy of operating system
    let _ = std::fs::create_dir_all(format!("{}{}/information", app_state.data_directory, created_user.name));

    Ok(Json(UserNoPassword { username: created_user.name }))
}


#[get("/whoAmI")]
pub async fn who_am_i(app_state: Data<AppState>, claims: TokenClaims) -> Result<impl Responder, GETError> {
    if let Ok(user) = app_state.db.user().select(&SelectUserById { id: claims.id }).await {
        return Ok(Json(user.name));
    }

    Err(GETError::UserNotFound)
}

#[put("{user_name}/list")]
pub async fn put_list(user_name: Path<String>, body: Json<List>, state: Data<AppState>, claims: TokenClaims) -> Result<HttpResponse, UploadError> {
    let user_name = user_name.into_inner();
    let send_list: List = body.into_inner();

    let user = state.db.user().select(&SelectUserByName { username: &user_name }).await?;
    let requesting_user = state.db.user().select(&SelectUserById { id: claims.id }).await?;

    if user.id != claims.id && !requesting_user.is_admin() {
        return Err(UploadError::Unauthorized);
    }

    let destination = format!("{}{}/information/list.json", state.data_directory, user.name);

    let list_as_string = serde_json::ser::to_string_pretty(&send_list)
        .map_err(|_| UploadError::WritingError)?;

    let mut saved_file: tokio::fs::File = tokio::fs::File::create(&destination).await?;
    saved_file.write_all(list_as_string.as_bytes()).map_err(|_| UploadError::WritingError).await?;


    Ok(HttpResponse::Ok().into())
}

#[get("{user_name}/list")]
pub async fn list(user_name: Path<String>, state: Data<AppState>, claims: TokenClaims) -> Result<Json<List>, GETError> {
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

    let path = format!("{}{}/information/list.json", state.data_directory, user.name);

    let file = File::open(&path);

    if file.is_err() {
        match File::create(&path) {
            Ok(mut file) => { let _ = file.write_all(b"{\"entries\": []}"); },
            Err(err) => {
                log::error!("{}", err);
                return Err(GETError::CantRead);
            }
        }
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    if let Ok(list) = serde_json::from_reader(reader) {
        return Ok(Json(list));
    }

    Err(GETError::CantRead)
}

#[get("/{user_name}/information")]
pub async fn full_profile_information(user_name: Path<String>, state: Data<AppState>) -> actix_web::Result<Json<UserProfile>, GETError> {
    let user_name = user_name.into_inner();

    let user = state.db.user().select(&SelectUserByName { username: &user_name }).await?;
    let amount_posts = count(&user_name, &state)?;

    Ok(Json(UserProfile {
        amount_posts,
        description: user.description,
        is_bot: user.is_bot
    }))
}

#[put("/{user_name}/information")]
pub async fn put_user_information(new_description: Json<UpdateDescription>, user_name: Path<String>, claims: TokenClaims, state: Data<AppState>) -> Result<Json<UserProfile>, UploadError> {
    let user_name = user_name.into_inner();

    let user = state.db.user().select(&SelectUserByName { username: &user_name }).await?;
    let requesting_user = state.db.user().select(&SelectUserById { id: claims.id }).await?;

    if user.id != claims.id && !requesting_user.is_admin() {
        return Err(UploadError::Unauthorized);
    }

    let updated_user = state.db.user().update(&UpdateUser { target_id: user.id, new_description: new_description.description.clone() }).await?;
    let amount_posts = count(&user_name, &state)?;

    Ok(Json(UserProfile {
        amount_posts,
        description: updated_user.description,
        is_bot: user.is_bot
    }))
}

#[post("/{user_name}/avatar")]
pub async fn upload_avatar(payload: Multipart, req: HttpRequest, user_name: Path<String>, claims: TokenClaims, state: Data<AppState>) -> Result<HttpResponse, UploadError> {
    let user_name = user_name.into_inner();

    let user = state.db.user().select(&SelectUserByName { username: &user_name }).await?;
    let requesting_user = state.db.user().select(&SelectUserById { id: claims.id }).await?;

    if user.id != claims.id && !requesting_user.is_admin() {
        return Err(UploadError::Unauthorized);
    }

    let upload_options = UploadOptions {
        max_file_count: 1,
        max_file_size: 30_000_000, // 30mb
        legal_file_types: vec![mime::IMAGE_JPEG, mime::IMAGE_PNG]
    };

    write_files_in_directory(&req, payload, upload_options, &state, |inner_state, _| {
        format!("{}{}/information/avatar.jpeg", inner_state.data_directory, user.name)
    }).await?;


    Ok(HttpResponse::Ok().into())
}

#[post("/{user_a}/friendship/{user_b}")]
pub async fn post_friendship(users: Path<(String, String)>, state: Data<AppState>, claims: TokenClaims) -> actix_web::Result<HttpResponse, UploadError> {
    let users = users.into_inner();

    let user_a = state.db.user().select(&SelectUserByName { username: &users.0 }).await?;
    let user_b = state.db.user().select(&SelectUserByName { username: &users.1 }).await?;
    let requesting_user = state.db.user().select(&SelectUserById { id: claims.id }).await?;

    if user_a.id != claims.id && !requesting_user.is_admin() {
        return Err(UploadError::Unauthorized);
    }

    let friendship = Friendship { friend_a: user_a.id, friend_b: user_b.id };
    state.db.friendship().insert(friendship).await?;

    Ok(HttpResponse::Ok().into())
}


#[get("/{user_name}/friends")]
pub async fn get_friends(user_name: Path<String>, state: Data<AppState>, claims: TokenClaims) -> actix_web::Result<Json<Vec<Friend>>, GETError> {
    let user = state.db.user().select(&SelectUserByName { username: &user_name.into_inner() }).await?;
    let requesting_user = state.db.user().select(&SelectUserById { id: claims.id }).await?;

    if user.id != claims.id && !requesting_user.is_admin() {
        return Err(GETError::Unauthorized);
    }

    let friendships = state.db.friendship().select(&SelectUserById { id: user.id }).await?;

    let friend_ids = friendships
        .iter()
        .map(|friendship| if friendship.friend_a == user.id { friendship.friend_b } else { friendship.friend_a })
        .collect::<Vec<_>>();

    let mut friends = vec![];

    for friend in friend_ids {
        // potentially do this in parallel
        let user = state.db.user().select(&SelectUserById { id: friend }).await?;
        friends.push(Friend {
            user_name: user.name.clone(),
            profile_image: format!("user/{}/avatar", &user.name),
            description: user.description,
            is_bot: user.is_bot,
            amount_posts: count(&user.name, &state)?
        });
    }

    Ok(Json(friends))
}


#[get("/{user_name}/avatar")]
pub async fn avatar(user_name: Path<String>, state: Data<AppState>) -> actix_web::Result<NamedFile, GETError> {
    let user_name = user_name.into_inner();
    let user = state.db.user().select(&SelectUserByName { username: &user_name }).await?;

    Ok(NamedFile::open(format!("{}{}/information/avatar.jpeg", state.data_directory, user.name))?)
}

fn count(user_name: &str, app_state: &Data<AppState>) -> Result<usize, GETError> {
    if let Ok(paths) = std::fs::read_dir(format!("{}{}", &app_state.data_directory, user_name)) {
        return Ok(paths
            .filter_map(|file| file.ok())
            .filter(|a| a.path().is_file())
            .count()
        )
    }

    Err(GETError::CantRead)
}
