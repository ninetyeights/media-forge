use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageProcessSettings {
    pub compress_enabled: bool,
    pub quality: u8,
    pub convert_enabled: bool,
    pub output_format: String,
    pub resize_enabled: bool,
    pub resize_width: Option<u32>,
    pub resize_height: Option<u32>,
    pub keep_aspect_ratio: bool,
    pub target_size: Option<u64>,  // bytes, iteratively compress to fit
}
