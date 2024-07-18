use axum::{extract::Multipart, extract::State};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::AppState;

#[derive(Clone)]
pub struct UploadState {
    files: Arc<Mutex<HashMap<String, File>>>,
}

impl UploadState {
    pub fn new() -> Self {
        UploadState {
            files: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Deserialize)]
struct ChunkInfo {
    chunk: usize,
    chunks: usize,
}

pub async fn handle_upload(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<String, String> {
    let upload_dir = Path::new("uploads");
    if (!upload_dir.exists()) {
        fs::create_dir_all(upload_dir).map_err(|e| e.to_string())?;
    }

    let upload_state = app_state.upload_state.lock().await;
    let mut files = upload_state.files.lock().await;

    while let Some(mut field) = multipart.next_field().await.map_err(|e| e.to_string())? {
        let name = field.name().unwrap_or("").to_string();
        let file_name = field.file_name().unwrap_or("").to_string();

        if name == "file" && !file_name.is_empty() {
            let chunk_info: ChunkInfo =
                serde_json::from_str(&field.text().await.map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;

            let chunk_data = field.bytes().await.map_err(|e| e.to_string())?;

            let mut file = if chunk_info.chunk == 0 {
                // First chunk, create new file
                let path = upload_dir.join(&file_name);
                File::create(path).map_err(|e| e.to_string())?
            } else {
                // Subsequent chunk, get existing file
                files
                    .get_mut(&file_name)
                    .ok_or("File not found")?
                    .try_clone()
                    .map_err(|e| e.to_string())?
            };

            file.write_all(&chunk_data).map_err(|e| e.to_string())?;

            if chunk_info.chunk == chunk_info.chunks - 1 {
                // Last chunk, close the file
                files.remove(&file_name);
            } else {
                // Not last chunk, keep file open
                files.insert(file_name, file);
            }
        }
    }

    Ok("Upload successful".to_string())
}
