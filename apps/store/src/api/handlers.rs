//! API Request Handlers
//!
//! HTTP request handlers for all API endpoints.
//! Thin adapter layer between HTTP and business logic.

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};

use crate::api::{responses::ApiResponse, AppState};
use crate::media::{self, types::{MediaMetadata, UploadResponse, DeleteMediaResponse}};
use crate::slideshow::{self, types::{SlideshowSummary, CreateSlideshowRequest, Slideshow, UpdateSlideshowRequest}};

pub async fn health_handler() -> Json<ApiResponse<()>> {
    Json(ApiResponse::success(()))
}

/// # Errors
///
/// Returns an error if the database operation fails.
pub async fn list_slideshows(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<SlideshowSummary>>>, (StatusCode, String)> {
    slideshow::list_slideshows(&state.pool)
        .await
        .map(|slideshows| Json(ApiResponse::success(slideshows)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// # Errors
///
/// Returns an error if the database operation fails.
pub async fn create_slideshow(
    State(state): State<AppState>,
    Json(request): Json<CreateSlideshowRequest>,
) -> Result<Json<ApiResponse<Slideshow>>, (StatusCode, String)> {
    slideshow::create_slideshow(&state.pool, request)
        .await
        .map(|slideshow| Json(ApiResponse::success(slideshow)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// # Errors
///
/// Returns an error if the database operation fails or the slideshow is not found.
pub async fn get_slideshow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Slideshow>>, (StatusCode, String)> {
    match slideshow::get_slideshow(&state.pool, &id).await {
        Ok(slideshow) => Ok(Json(ApiResponse::success(slideshow))),
        Err(e) if e.to_string().contains("not found") => {
            Err((StatusCode::NOT_FOUND, e.to_string()))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// # Errors
///
/// Returns an error if the database operation fails or the slideshow is not found.
pub async fn update_slideshow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateSlideshowRequest>,
) -> Result<Json<ApiResponse<Slideshow>>, (StatusCode, String)> {
    match slideshow::update_slideshow(&state.pool, &id, request).await {
        Ok(slideshow) => Ok(Json(ApiResponse::success(slideshow))),
        Err(e) if e.to_string().contains("not found") => {
            Err((StatusCode::NOT_FOUND, e.to_string()))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// # Errors
///
/// Returns an error if the database operation fails or the slideshow is not found.
pub async fn delete_slideshow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, String)> {
    match slideshow::delete_slideshow(&state.pool, &id).await {
        Ok(()) => {
            let response = serde_json::json!({
                "message": "Slideshow deleted successfully",
                "slideshow_id": id
            });
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) if e.to_string().contains("not found") => {
            Err((StatusCode::NOT_FOUND, e.to_string()))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// # Errors
///
/// Returns an error if the media directory cannot be read.
pub async fn list_media(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<MediaMetadata>>>, (StatusCode, String)> {
    media::list_media(&state.media_dir)
        .await
        .map(|files| Json(ApiResponse::success(files)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// # Errors
///
/// Returns an error if multipart parsing fails or the upload operation fails.
pub async fn upload_media(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<UploadResponse>>, (StatusCode, String)> {
    let mut files = Vec::new();

    while let Some(field) = multipart.next_field().await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("multipart error: {e}")))?
    {
        let filename = field.file_name()
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing filename in multipart field".to_string()))?
            .to_string();

        let data = field.bytes().await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("failed to read field bytes: {e}")))?
            .to_vec();

        files.push((filename, data));
    }

    media::upload_media(&state.media_dir, files)
        .await
        .map(|response| Json(ApiResponse::success(response)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// # Errors
///
/// Returns an error if the media file is not found or cannot be deleted.
pub async fn delete_media(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<ApiResponse<DeleteMediaResponse>>, (StatusCode, String)> {
    match media::delete_media(&state.media_dir, &filename).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) if e.to_string().contains("not found") => {
            Err((StatusCode::NOT_FOUND, e.to_string()))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// # Errors
///
/// Returns an error if the media file is not found or cannot be read.
pub async fn get_media_file(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    use axum::http::header;
    use axum::{body::Body, response::IntoResponse};
    use tokio::fs::File;
    use tokio_util::io::ReaderStream;

    let path = media::get_media_path(&state.media_dir, &filename)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                (StatusCode::NOT_FOUND, e.to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        })?;

    let file = File::open(&path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mime = mime_guess::from_path(&path)
        .first_or_octet_stream()
        .to_string();

    Ok(([(header::CONTENT_TYPE, mime)], body).into_response())
}
