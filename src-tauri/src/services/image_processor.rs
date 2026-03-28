use image::{DynamicImage, ImageFormat, ImageReader};
use std::path::Path;

use crate::models::media_file::{ImageInfo, ProcessResult};
use crate::models::settings::ImageProcessSettings;

pub fn get_image_info(path: &str) -> Result<ImageInfo, String> {
    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let reader = ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let format = reader.format().ok_or("Cannot detect image format")?;
    let (width, height) = reader.into_dimensions().map_err(|e| e.to_string())?;

    Ok(ImageInfo {
        width,
        height,
        format: format_to_str(format).to_string(),
        file_size: metadata.len(),
    })
}

pub fn process_image(
    file_path: &str,
    output_path: &str,
    settings: &ImageProcessSettings,
) -> Result<ProcessResult, String> {
    let total_start = std::time::Instant::now();

    let original_size = std::fs::metadata(file_path)
        .map_err(|e| e.to_string())?
        .len();

    let source_format = detect_format(file_path)?;
    let target_format = if settings.convert_enabled && settings.output_format != "original" {
        str_to_format(&settings.output_format)?
    } else {
        source_format
    };

    let needs_processing = settings.compress_enabled
        || settings.resize_enabled
        || settings.target_size.is_some()
        || target_format != source_format;

    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    if !needs_processing {
        println!("[process] FAST PATH: copy only, {:.0}KB", original_size as f64 / 1024.0);
        if file_path != output_path {
            std::fs::copy(file_path, output_path).map_err(|e| e.to_string())?;
        }
        println!("[process] total: {:?}", total_start.elapsed());
        return Ok(ProcessResult {
            output_path: output_path.to_string(),
            original_size,
            compressed_size: original_size,
            ratio: 0.0,
        });
    }

    println!("[process] START {:?} → {:?}, compress={}, resize={}, size={:.0}KB",
        source_format, target_format, settings.compress_enabled, settings.resize_enabled, original_size as f64 / 1024.0);

    // Decode
    let t = std::time::Instant::now();
    let mut img = ImageReader::open(file_path)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;
    println!("[process] decode: {:?} ({}x{})", t.elapsed(), img.width(), img.height());

    // Resize
    if settings.resize_enabled {
        let t = std::time::Instant::now();
        img = resize_image(img, settings.resize_width, settings.resize_height, settings.keep_aspect_ratio);
        println!("[process] resize: {:?} → {}x{}", t.elapsed(), img.width(), img.height());
    }

    let quality = if settings.compress_enabled { settings.quality } else { 100 };

    // Encode
    let compressed_size = if let Some(target_bytes) = settings.target_size {
        // Binary search: find highest quality that fits under target size (max ~7 iterations)
        let t = std::time::Instant::now();
        let data = compress_to_target_size(&img, target_bytes, target_format)?;
        println!("[process] target_size (≤{:.0}KB): {:?}, result={:.0}KB",
            target_bytes as f64 / 1024.0, t.elapsed(), data.len() as f64 / 1024.0);
        std::fs::write(output_path, &data).map_err(|e| e.to_string())?;
        data.len() as u64
    } else {
        let t = std::time::Instant::now();
        encode_and_save(&img, output_path, target_format, quality)?;
        let size = std::fs::metadata(output_path).map_err(|e| e.to_string())?.len();
        println!("[process] encode (q={}): {:?} → {:.0}KB", quality, t.elapsed(), size as f64 / 1024.0);
        size
    };

    println!("[process] TOTAL: {:?}", total_start.elapsed());

    let ratio = if original_size > 0 {
        1.0 - (compressed_size as f64 / original_size as f64)
    } else {
        0.0
    };

    Ok(ProcessResult {
        output_path: output_path.to_string(),
        original_size,
        compressed_size,
        ratio,
    })
}

fn resize_image(
    img: DynamicImage,
    width: Option<u32>,
    height: Option<u32>,
    keep_aspect: bool,
) -> DynamicImage {
    let (orig_w, orig_h) = (img.width(), img.height());

    let (new_w, new_h) = match (width, height) {
        (Some(w), Some(h)) => {
            if keep_aspect {
                let ratio_w = w as f64 / orig_w as f64;
                let ratio_h = h as f64 / orig_h as f64;
                let ratio = ratio_w.min(ratio_h);
                ((orig_w as f64 * ratio) as u32, (orig_h as f64 * ratio) as u32)
            } else {
                (w, h)
            }
        }
        (Some(w), None) => {
            let ratio = w as f64 / orig_w as f64;
            (w, (orig_h as f64 * ratio) as u32)
        }
        (None, Some(h)) => {
            let ratio = h as f64 / orig_h as f64;
            ((orig_w as f64 * ratio) as u32, h)
        }
        (None, None) => (orig_w, orig_h),
    };

    if new_w == orig_w && new_h == orig_h {
        return img;
    }

    img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3)
}

fn detect_format(path: &str) -> Result<ImageFormat, String> {
    // Detect by file content first, fall back to extension
    if let Ok(reader) = ImageReader::open(path)
        .and_then(|r| r.with_guessed_format())
    {
        if let Some(fmt) = reader.format() {
            return Ok(fmt);
        }
    }

    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or("Cannot detect file format")?;

    str_to_format(&ext)
}

fn format_to_str(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Png => "png",
        ImageFormat::WebP => "webp",
        ImageFormat::Gif => "gif",
        ImageFormat::Bmp => "bmp",
        ImageFormat::Tiff => "tiff",
        ImageFormat::Ico => "ico",
        _ => "unknown",
    }
}

fn str_to_format(s: &str) -> Result<ImageFormat, String> {
    match s.to_lowercase().as_str() {
        "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
        "png" => Ok(ImageFormat::Png),
        "webp" => Ok(ImageFormat::WebP),
        "gif" => Ok(ImageFormat::Gif),
        "bmp" => Ok(ImageFormat::Bmp),
        "tiff" | "tif" => Ok(ImageFormat::Tiff),
        "avif" => Ok(ImageFormat::Avif),
        other => Err(format!("Unsupported format: {}", other)),
    }
}

/// Encode and save using the fastest available codec for each format.
/// JPEG: turbojpeg (libjpeg-turbo, SIMD optimized)
/// WebP: webp crate (libwebp)
/// AVIF: image crate (pure Rust, slow)
/// Others: image crate
fn encode_and_save(
    img: &DynamicImage,
    output_path: &str,
    format: ImageFormat,
    quality: u8,
) -> Result<(), String> {
    match format {
        ImageFormat::Jpeg => encode_jpeg_turbo(img, output_path, quality),
        ImageFormat::WebP => {
            let encoder = webp::Encoder::from_image(img).map_err(|e| e.to_string())?;
            let encoded = if quality >= 100 {
                encoder.encode_lossless()
            } else {
                encoder.encode(quality as f32)
            };
            std::fs::write(output_path, &*encoded).map_err(|e| e.to_string())
        }
        ImageFormat::Avif => {
            let file = std::fs::File::create(output_path).map_err(|e| e.to_string())?;
            let encoder = image::codecs::avif::AvifEncoder::new_with_speed_quality(file, 8, quality);
            img.write_with_encoder(encoder).map_err(|e| e.to_string())
        }
        ImageFormat::Png => {
            // First save with image crate, then optimize with oxipng (lossless)
            img.save(output_path).map_err(|e| e.to_string())?;
            let opts = oxipng::Options::from_preset(3);
            oxipng::optimize(
                &oxipng::InFile::Path(std::path::PathBuf::from(output_path)),
                &oxipng::OutFile::from_path(std::path::PathBuf::from(output_path)),
                &opts,
            ).map_err(|e| e.to_string())?;
            Ok(())
        }
        _ => {
            img.save(output_path).map_err(|e| e.to_string())
        }
    }
}

/// JPEG encoding via libjpeg-turbo (3-5x faster than pure Rust)
fn encode_jpeg_turbo(img: &DynamicImage, output_path: &str, quality: u8) -> Result<(), String> {
    let rgb = img.to_rgb8();
    let (width, height) = (rgb.width() as usize, rgb.height() as usize);
    let pixels: &[u8] = rgb.as_raw().as_slice();

    let image = turbojpeg::Image {
        pixels,
        width,
        pitch: width * 3,
        height,
        format: turbojpeg::PixelFormat::RGB,
    };

    let mut compressor = turbojpeg::Compressor::new().map_err(|e| e.to_string())?;
    compressor.set_quality(quality as i32).map_err(|e| e.to_string())?;
    compressor.set_subsamp(turbojpeg::Subsamp::Sub2x2).map_err(|e| e.to_string())?;

    let jpeg_data = compressor.compress_to_vec(image).map_err(|e| e.to_string())?;
    std::fs::write(output_path, &jpeg_data).map_err(|e| e.to_string())
}

/// Returns true if the format supports quality-based lossy compression.
fn supports_quality(format: ImageFormat) -> bool {
    matches!(format, ImageFormat::Jpeg | ImageFormat::WebP | ImageFormat::Avif)
}

/// Binary search for the highest quality that produces output ≤ target_bytes.
/// Encodes to memory (no disk I/O per iteration), max ~7 iterations.
/// For lossless formats (PNG/GIF/BMP/TIFF), automatically falls back to WebP.
fn compress_to_target_size(
    img: &DynamicImage,
    target_bytes: u64,
    format: ImageFormat,
) -> Result<Vec<u8>, String> {
    // PNG: use imagequant color quantization (pngquant algorithm)
    if format == ImageFormat::Png {
        return compress_png_to_target_size(img, target_bytes);
    }

    // Other lossless formats: not supported
    if !supports_quality(format) {
        return Err(format!(
            "{} 不支持按目标大小压缩，请先转换为 JPG/WebP/PNG 等格式",
            format_to_str(format).to_uppercase()
        ));
    }
    let enc_format = format;

    // First check: quality=100, if already fits, return immediately
    let full = encode_to_vec(img, enc_format, 100)?;
    if full.len() as u64 <= target_bytes {
        println!("[target] q=100 already fits: {:.0}KB", full.len() as f64 / 1024.0);
        return Ok(full);
    }

    let mut low: u8 = 1;
    let mut high: u8 = 99;
    let mut best = encode_to_vec(img, enc_format, 1)?; // worst case fallback

    while low <= high {
        let mid = (low + high) / 2;
        let buf = encode_to_vec(img, enc_format, mid)?;
        println!("[target] q={}: {:.0}KB (target: {:.0}KB)", mid, buf.len() as f64 / 1024.0, target_bytes as f64 / 1024.0);

        if (buf.len() as u64) <= target_bytes {
            best = buf;
            low = mid + 1; // try higher quality
        } else {
            if mid == 0 { break; }
            high = mid - 1; // too big, lower quality
        }
    }

    Ok(best)
}

/// Encode image to in-memory Vec<u8> (no disk I/O).
fn encode_to_vec(
    img: &DynamicImage,
    format: ImageFormat,
    quality: u8,
) -> Result<Vec<u8>, String> {
    match format {
        ImageFormat::Jpeg => {
            let rgb = img.to_rgb8();
            let (width, height) = (rgb.width() as usize, rgb.height() as usize);
            let pixels: &[u8] = rgb.as_raw().as_slice();
            let image = turbojpeg::Image {
                pixels,
                width,
                pitch: width * 3,
                height,
                format: turbojpeg::PixelFormat::RGB,
            };
            let mut compressor = turbojpeg::Compressor::new().map_err(|e| e.to_string())?;
            compressor.set_quality(quality as i32).map_err(|e| e.to_string())?;
            compressor.set_subsamp(turbojpeg::Subsamp::Sub2x2).map_err(|e| e.to_string())?;
            compressor.compress_to_vec(image).map_err(|e| e.to_string())
        }
        ImageFormat::WebP => {
            let encoder = webp::Encoder::from_image(img).map_err(|e| e.to_string())?;
            let encoded = if quality >= 100 {
                encoder.encode_lossless()
            } else {
                encoder.encode(quality as f32)
            };
            Ok(encoded.to_vec())
        }
        _ => {
            let mut buf = std::io::Cursor::new(Vec::new());
            img.write_to(&mut buf, format).map_err(|e| e.to_string())?;
            Ok(buf.into_inner())
        }
    }
}

/// PNG target size compression via imagequant (pngquant algorithm).
/// Binary search on imagequant quality (0-100) to find best fit under target.
fn compress_png_to_target_size(
    img: &DynamicImage,
    target_bytes: u64,
) -> Result<Vec<u8>, String> {
    let rgba = img.to_rgba8();
    let width = rgba.width() as usize;
    let height = rgba.height() as usize;

    // First try lossless (oxipng optimized) to see if it already fits
    let lossless = {
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, ImageFormat::Png).map_err(|e| e.to_string())?;
        buf.into_inner()
    };
    if lossless.len() as u64 <= target_bytes {
        println!("[target-png] lossless already fits: {:.0}KB", lossless.len() as f64 / 1024.0);
        return Ok(lossless);
    }

    // Binary search on imagequant quality
    let mut low = 0u8;
    let mut high = 100u8;
    let mut best: Option<Vec<u8>> = None;

    while low <= high {
        let mid = (low + high) / 2;
        match quantize_png(&rgba, width, height, mid) {
            Ok(buf) => {
                println!("[target-png] q={}: {:.0}KB (target: {:.0}KB)", mid, buf.len() as f64 / 1024.0, target_bytes as f64 / 1024.0);
                if (buf.len() as u64) <= target_bytes {
                    best = Some(buf);
                    low = mid + 1; // try higher quality
                } else {
                    if mid == 0 { break; }
                    high = mid - 1;
                }
            }
            Err(e) => {
                println!("[target-png] q={} failed: {}", mid, e);
                if mid == 0 { break; }
                high = mid - 1;
            }
        }
    }

    best.ok_or_else(|| "无法将 PNG 压缩到目标大小，图片可能太大，请尝试缩小尺寸".to_string())
}

/// Quantize PNG to indexed color using imagequant (pngquant algorithm), encode with image crate.
fn quantize_png(rgba: &image::RgbaImage, width: usize, height: usize, quality: u8) -> Result<Vec<u8>, String> {
    let mut liq = imagequant::new();
    liq.set_quality(0, quality).map_err(|e| e.to_string())?;

    let pixels: Vec<imagequant::RGBA> = rgba.pixels()
        .map(|p| imagequant::RGBA { r: p[0], g: p[1], b: p[2], a: p[3] })
        .collect();

    let mut img = liq.new_image(pixels, width, height, 0.0).map_err(|e| e.to_string())?;
    let mut res = liq.quantize(&mut img).map_err(|e| e.to_string())?;
    res.set_dithering_level(1.0).map_err(|e| e.to_string())?;

    let (palette, indexed_pixels) = res.remapped(&mut img).map_err(|e| e.to_string())?;

    // Reconstruct RGBA image from quantized palette + indices
    let mut out_pixels: Vec<u8> = Vec::with_capacity(width * height * 4);
    for &idx in &indexed_pixels {
        let c = &palette[idx as usize];
        out_pixels.extend_from_slice(&[c.r, c.g, c.b, c.a]);
    }

    let out_img = image::RgbaImage::from_raw(width as u32, height as u32, out_pixels)
        .ok_or("Failed to create quantized image")?;

    let mut buf = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(out_img)
        .write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}
