use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub file_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessResult {
    pub output_path: String,
    pub original_size: u64,
    pub compressed_size: u64,
    pub ratio: f64,
}
