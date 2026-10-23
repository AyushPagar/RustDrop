use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path},
    http::header,
    response::{Html, IntoResponse},
    routing::get,
    Json,
    Router,
};
use serde::Serialize;
use std::{env, path::PathBuf, sync::atomic::{AtomicU64, Ordering}, time::{SystemTime, UNIX_EPOCH}};
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize)]
struct SharedFile {
    name: String,
    size: u64,
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = env::var("RUSTDROP_BIND").unwrap_or_else(|_| "0.0.0.0:3000".to_owned());
    let app = Router::new()
        .route("/", get(home))
        .route("/static/style.css", get(stylesheet))
        .route("/static/app.js", get(javascript))
        .route("/api/files", get(list_files).post(upload_files))
        .route("/files/{name}", get(download_file))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024));
    let listener = tokio::net::TcpListener::bind(&address).await?;

    println!("RustDrop is listening on {address} for devices on your local network.");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn home() -> Html<&'static str> {
    Html(include_str!("../templates/index.html"))
}

async fn stylesheet() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../static/style.css"),
    )
}

async fn javascript() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        include_str!("../static/app.js"),
    )
}

fn upload_dir() -> PathBuf {
    env::var_os("RUSTDROP_DATA_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("uploads"))
}

async fn list_files() -> Result<Json<Vec<SharedFile>>, (axum::http::StatusCode, String)> {
    let directory = upload_dir();
    tokio::fs::create_dir_all(&directory).await.map_err(internal_error)?;
    let mut entries = tokio::fs::read_dir(directory).await.map_err(internal_error)?;
    let mut files = Vec::new();
    while let Some(entry) = entries.next_entry().await.map_err(internal_error)? {
        let metadata = entry.metadata().await.map_err(internal_error)?;
        if metadata.is_file() {
            let stored_name = entry.file_name().to_string_lossy().into_owned();
            let name = original_name(&stored_name).to_owned();
            files.push(SharedFile { name, size: metadata.len(), url: format!("/files/{stored_name}") });
        }
    }
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(Json(files))
}

async fn upload_files(mut multipart: Multipart) -> Result<Json<Vec<SharedFile>>, (axum::http::StatusCode, String)> {
    let directory = upload_dir();
    tokio::fs::create_dir_all(&directory).await.map_err(internal_error)?;
    while let Some(field) = multipart.next_field().await.map_err(bad_request)? {
        let Some(filename) = field.file_name().map(str::to_owned) else { continue };
        let filename_path = PathBuf::from(&filename);
        let safe_name = filename_path.file_name().and_then(|name| name.to_str()).unwrap_or("file");
        let safe_name: String = safe_name.chars().map(|ch| if ch.is_ascii_alphanumeric() || " ._-()".contains(ch) { ch } else { '_' }).collect();
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        let stored_name = format!("{stamp}-{}-{safe_name}", NEXT_ID.fetch_add(1, Ordering::Relaxed));
        let path = directory.join(stored_name);
        let mut file = tokio::fs::File::create(&path).await.map_err(internal_error)?;
        let mut field = field;
        loop {
            match field.chunk().await {
                Ok(Some(chunk)) => file.write_all(&chunk).await.map_err(internal_error)?,
                Ok(None) => break,
                Err(error) => {
                    drop(file);
                    let _ = tokio::fs::remove_file(path).await;
                    return Err(bad_request(error));
                }
            }
        }
    }
    list_files().await
}

async fn download_file(Path(name): Path<String>) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    if name.is_empty() || !name.chars().all(|ch| ch.is_ascii_alphanumeric() || " ._-()".contains(ch)) {
        return Err((axum::http::StatusCode::NOT_FOUND, "File not found".to_owned()));
    }
    let path = upload_dir().join(&name);
    let file = tokio::fs::File::open(path).await.map_err(|_| (axum::http::StatusCode::NOT_FOUND, "File not found".to_owned()))?;
    let size = file.metadata().await.map_err(internal_error)?.len();
    let display_name = original_name(&name).replace('"', "");
    let mut response = Body::from_stream(ReaderStream::new(file)).into_response();
    response.headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/octet-stream"));
    response.headers_mut().insert(header::CONTENT_LENGTH, header::HeaderValue::from(size));
    let disposition = header::HeaderValue::from_str(&format!("attachment; filename=\"{display_name}\""))
        .map_err(|error| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    response.headers_mut().insert(header::CONTENT_DISPOSITION, disposition);
    Ok(response)
}

fn original_name(stored_name: &str) -> &str {
    stored_name.split_once('-').and_then(|(_, rest)| rest.split_once('-').map(|(_, name)| name)).unwrap_or(stored_name)
}

fn internal_error(error: std::io::Error) -> (axum::http::StatusCode, String) {
    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn bad_request(error: axum::extract::multipart::MultipartError) -> (axum::http::StatusCode, String) {
    (axum::http::StatusCode::BAD_REQUEST, error.to_string())
}
