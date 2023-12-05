use std::str::FromStr;
use actix_web::get;
use actix_web::web::{Json};
use crate::api::shared::GETError;
use crate::middleware::TokenClaims;
use crate::utils::read_files_in_directory;
use crate::utils::version::Version;

#[get("/changelog")]
pub async fn changelog(claims: TokenClaims) -> Result<Json<Vec<String>>, GETError> {
    let version = get_highest_version()?;
    let path = format!("./changelog/Version {}", version.version);

    // so it doesnt get optimized out
    // claims is needed for authentication
    let _ = claims.id;

    if let Ok(content) = std::fs::read_to_string(path) {
        return Ok(Json(content.lines().map(|a| a.to_string()).collect::<Vec<_>>()));
    }

    Ok(Json(vec![]))
}

#[get("/changelog/version")]
pub async fn changelog_version(claims: TokenClaims) -> Result<Json<f32>, GETError> {
    let _ = claims.id;
    let version = get_highest_version()?;

    Ok(Json(version.version))
}

fn get_highest_version() -> Result<Version, GETError> {
    let changelog_path = "./changelog";
    let latest_version = read_files_in_directory(changelog_path, false)?
        .iter()
        .map(|dir| dir.file_name().into_string())
        .filter_map(|f| f.ok())
        .map(|file_name| Version::from_str(&file_name))
        .filter_map(|f| f.ok())
        .max();

    if let Some(last) = latest_version {
        Ok(last.clone())
    } else {
        Err(GETError::CantRead)
    }
}
