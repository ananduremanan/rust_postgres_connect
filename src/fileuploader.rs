use axum::{extract::Multipart, extract::State};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
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

pub async fn handle_upload(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<String, String> {
    let upload_dir = Path::new("uploads");
    if !upload_dir.exists() {
        fs::create_dir_all(upload_dir).map_err(|e| e.to_string())?;
    }

    let upload_state = app_state.upload_state.lock().await;
    let mut files = upload_state.files.lock().await;

    let mut file_name = String::new();
    let mut chunk_index = 0;
    let mut total_chunks = 0;
    let mut chunk_data = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| e.to_string())? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                file_name = field.file_name().unwrap_or("").to_string();
                chunk_data = field.bytes().await.map_err(|e| e.to_string())?.to_vec();
            }
            "chunkIndex" => {
                chunk_index = field
                    .text()
                    .await
                    .map_err(|e| e.to_string())?
                    .parse::<usize>()
                    .map_err(|e| e.to_string())?;
            }
            "totalChunks" => {
                total_chunks = field
                    .text()
                    .await
                    .map_err(|e| e.to_string())?
                    .parse::<usize>()
                    .map_err(|e| e.to_string())?;
            }
            _ => {}
        }
    }

    if !file_name.is_empty() {
        let path = upload_dir.join(&file_name);
        let mut file = if chunk_index == 0 {
            File::create(&path).map_err(|e| e.to_string())?
        } else {
            OpenOptions::new()
                .write(true)
                .append(true)
                .open(&path)
                .map_err(|e| e.to_string())?
        };

        file.write_all(&chunk_data).map_err(|e| e.to_string())?;

        if chunk_index == total_chunks - 1 {
            // Last chunk, close the file
            files.remove(&file_name);
        } else {
            // Not last chunk, keep file open
            files.insert(file_name, file);
        }
    }

    Ok("Chunk uploaded successfully".to_string())
}
