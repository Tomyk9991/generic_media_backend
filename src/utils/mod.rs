use std::cmp::Ordering;
use std::fs::DirEntry;
use std::time::{Duration, SystemTime};

use actix_multipart::Multipart;
use actix_web::HttpRequest;
use actix_web::web::Data;
use futures_util::TryStreamExt;
use mime::Mime;
use tokio::io::AsyncWriteExt;

use crate::api::shared::UploadError;
use crate::model::states::app_state::AppState;

pub mod version;

pub fn read_files_in_directory(path: &str, reversed: bool) -> std::io::Result<Vec<DirEntry>> {
    let directory = std::fs::read_dir(path)?;

    let mut all_files = directory
        .filter_map(|file| file.ok())
        .filter(|a| a.path().is_file())
        .collect::<Vec<_>>();


    // sort by creation_time otherwise say they are equal
    all_files.sort_by(|a, b| {
        if let Ok(meta_data_a) = a.metadata() {
            if let Ok(meta_data_b) = b.metadata() {
                if let Ok(last_mod_a) = meta_data_a.modified() {
                    if let Ok(last_mod_b) = meta_data_b.modified() {
                        return if reversed {
                            last_mod_a.cmp(&last_mod_b)
                        } else {
                            last_mod_b.cmp(&last_mod_a)
                        };
                    }
                }
            }
        }

        Ordering::Equal
    });

    Ok(all_files)
}

pub fn validate_stories(path: &str) -> std::io::Result<()> {
    let directory = std::fs::read_dir(path)?;

    let all_files_older_than_24_hours = directory
        .filter_map(|file| file.ok())
        .filter(|a| a.path().is_file())
        .filter(|entry| {
            if let Ok(meta_data) = entry.metadata() {
                if let Ok(last_modified) = meta_data.modified() {
                    return is_older_than_24_hours(last_modified);
                }
            }

            true
        })
        .collect::<Vec<_>>();

    for file in &all_files_older_than_24_hours {
        std::fs::remove_file(file.path().display().to_string())?;
    }

    Ok(())
}

fn is_older_than_24_hours(target_time: SystemTime) -> bool {
    let current_time = SystemTime::now();
    let twenty_four_hours = Duration::from_secs(24 * 3600);

    if let Ok(duration) = current_time.duration_since(target_time) {
        return duration > twenty_four_hours;
    }

    false
}

pub struct UploadOptions {
    pub max_file_count: usize,
    pub max_file_size: usize,
    pub legal_file_types: Vec<Mime>,
}

pub async fn write_files_in_directory(req: &HttpRequest, mut payload: Multipart, upload_options: UploadOptions, state: &Data<AppState>, file_name_delegate: impl Fn(&Data<AppState>, &str) -> String) -> Result<(), UploadError> {
    let content_length: usize = match req.headers().get(actix_web::http::header::CONTENT_LENGTH) {
        Some(header_value) => header_value
            .to_str()
            .unwrap_or("0")
            .parse::<usize>()
            .map_err(UploadError::CorruptedHeaderLength)?,
        None => 0
    };

    let mut current_count = 0;
    if content_length > upload_options.max_file_size { return Err(UploadError::FileSizeTooBig); }

    loop {
        if current_count == upload_options.max_file_count { break; }
        if let Ok(Some(mut field)) = payload.try_next().await {
            if let Some(field_type) = field.content_type() {
                if !upload_options.legal_file_types.contains(field_type) {
                    return Err(UploadError::IllegalContentType);
                }
            }

            let file_name = field.content_disposition().get_filename().unwrap_or("Default name");
            let destination = file_name_delegate(state, file_name);
            let mut saved_file: tokio::fs::File = tokio::fs::File::create(&destination).await?;

            while let Ok(Some(chunk)) = field.try_next().await {
                if saved_file.write_all(&chunk).await.is_ok() {} else {
                    return Err(UploadError::WritingError);
                }
            }
        }

        current_count += 1;
    }

    Ok(())
}