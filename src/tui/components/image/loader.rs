//! Image loading functionality
//! 
//! This module handles loading images from various sources including
//! files, URLs, and raw bytes, with support for multiple formats
//! and comprehensive error handling.

use super::{ImageMetadata};
use anyhow::Result;
use image::{DynamicImage, ImageFormat, ImageReader};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use tokio::fs;

/// Image loader with support for multiple sources and formats
pub struct ImageLoader;

impl ImageLoader {
    /// Load image from file path
    pub async fn load_from_path(path: &Path) -> Result<(DynamicImage, ImageMetadata)> {
        // Read file asynchronously
        let data = fs::read(path).await?;
        let file_size = data.len() as u64;
        
        // Detect format from file extension
        let format = Self::detect_format_from_path(path)?;
        
        // Load image from bytes
        let (image, mut metadata) = Self::load_from_bytes_with_format(&data, format)?;
        metadata.file_size = Some(file_size);
        
        Ok((image, metadata))
    }
    
    /// Load image from URL
    pub async fn load_from_url(url: &str) -> Result<(DynamicImage, ImageMetadata)> {
        // Download image data
        let response = reqwest::get(url).await?;
        let data = response.bytes().await?;
        let file_size = data.len() as u64;
        
        // Try to detect format from URL
        let format = Self::detect_format_from_url(url)
            .or_else(|| Self::detect_format_from_bytes(&data));
        
        let (image, mut metadata) = if let Some(fmt) = format {
            Self::load_from_bytes_with_format(&data, fmt)?
        } else {
            Self::load_from_bytes(&data)?
        };
        
        metadata.file_size = Some(file_size);
        metadata.extra_info.insert("source".to_string(), "url".to_string());
        metadata.extra_info.insert("url".to_string(), url.to_string());
        
        Ok((image, metadata))
    }
    
    /// Load image from raw bytes
    pub fn load_from_bytes(data: &[u8]) -> Result<(DynamicImage, ImageMetadata)> {
        let reader = ImageReader::new(Cursor::new(data));
        let reader = reader.with_guessed_format()?;
        
        let format = reader.format()
            .ok_or_else(|| anyhow::anyhow!("Could not determine image format"))?;
        
        let image = reader.decode()?;
        let metadata = Self::extract_metadata(&image, format, data.len() as u64);
        
        Ok((image, metadata))
    }
    
    /// Load image from bytes with known format
    pub fn load_from_bytes_with_format(
        data: &[u8], 
        format: ImageFormat
    ) -> Result<(DynamicImage, ImageMetadata)> {
        let image = ImageReader::with_format(Cursor::new(data), format).decode()?;
        let metadata = Self::extract_metadata(&image, format, data.len() as u64);
        
        Ok((image, metadata))
    }
    
    /// Detect image format from file path
    fn detect_format_from_path(path: &Path) -> Result<ImageFormat> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow::anyhow!("No file extension found"))?;
        
        Self::format_from_extension(extension)
            .ok_or_else(|| anyhow::anyhow!("Unsupported file extension: {}", extension))
    }
    
    /// Detect image format from URL
    fn detect_format_from_url(url: &str) -> Option<ImageFormat> {
        // Extract file extension from URL
        let url_path = url.split('?').next().unwrap_or(url); // Remove query parameters
        let extension = url_path.split('.').last()?;
        Self::format_from_extension(extension)
    }
    
    /// Detect image format from file signature (magic bytes)
    fn detect_format_from_bytes(data: &[u8]) -> Option<ImageFormat> {
        if data.len() < 12 {
            return None;
        }
        
        // Check common image format signatures
        match &data[..12] {
            // PNG: 89 50 4E 47 0D 0A 1A 0A
            [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, ..] => Some(ImageFormat::Png),
            
            // JPEG: FF D8 FF
            [0xFF, 0xD8, 0xFF, ..] => Some(ImageFormat::Jpeg),
            
            // GIF87a or GIF89a
            [0x47, 0x49, 0x46, 0x38, 0x37, 0x61, ..] |
            [0x47, 0x49, 0x46, 0x38, 0x39, 0x61, ..] => Some(ImageFormat::Gif),
            
            // BMP: 42 4D
            [0x42, 0x4D, ..] => Some(ImageFormat::Bmp),
            
            // WEBP: 52 49 46 46 xx xx xx xx 57 45 42 50
            [0x52, 0x49, 0x46, 0x46, _, _, _, _, 0x57, 0x45, 0x42, 0x50] => Some(ImageFormat::WebP),
            
            _ => {
                // Check for TIFF (can start with either II or MM)
                if &data[..4] == [0x49, 0x49, 0x2A, 0x00] || &data[..4] == [0x4D, 0x4D, 0x00, 0x2A] {
                    Some(ImageFormat::Tiff)
                } else {
                    None
                }
            }
        }
    }
    
    /// Get ImageFormat from file extension
    fn format_from_extension(extension: &str) -> Option<ImageFormat> {
        match extension.to_lowercase().as_str() {
            "png" => Some(ImageFormat::Png),
            "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
            "gif" => Some(ImageFormat::Gif),
            "bmp" => Some(ImageFormat::Bmp),
            "ico" => Some(ImageFormat::Ico),
            "tiff" | "tif" => Some(ImageFormat::Tiff),
            "webp" => Some(ImageFormat::WebP),
            "pnm" | "pbm" | "pgm" | "ppm" => Some(ImageFormat::Pnm),
            "dds" => Some(ImageFormat::Dds),
            "tga" => Some(ImageFormat::Tga),
            "exr" => Some(ImageFormat::OpenExr),
            "hdr" => Some(ImageFormat::Hdr),
            "farbfeld" | "ff" => Some(ImageFormat::Farbfeld),
            "avif" => Some(ImageFormat::Avif),
            _ => None,
        }
    }
    
    /// Extract metadata from loaded image
    fn extract_metadata(
        image: &DynamicImage, 
        format: ImageFormat, 
        file_size: u64
    ) -> ImageMetadata {
        let width = image.width();
        let height = image.height();
        
        // Determine color depth and alpha channel presence
        let (color_depth, has_alpha) = match image {
            DynamicImage::ImageLuma8(_) => (8, false),
            DynamicImage::ImageLumaA8(_) => (8, true),
            DynamicImage::ImageRgb8(_) => (24, false),
            DynamicImage::ImageRgba8(_) => (32, true),
            DynamicImage::ImageLuma16(_) => (16, false),
            DynamicImage::ImageLumaA16(_) => (16, true),
            DynamicImage::ImageRgb16(_) => (48, false),
            DynamicImage::ImageRgba16(_) => (64, true),
            DynamicImage::ImageRgb32F(_) => (96, false),
            DynamicImage::ImageRgba32F(_) => (128, true),
            _ => (24, false), // Default fallback
        };
        
        let mut extra_info = HashMap::new();
        
        // Add format-specific information
        match format {
            ImageFormat::Png => {
                extra_info.insert("compression".to_string(), "lossless".to_string());
            }
            ImageFormat::Jpeg => {
                extra_info.insert("compression".to_string(), "lossy".to_string());
            }
            ImageFormat::Gif => {
                extra_info.insert("compression".to_string(), "lossless".to_string());
                extra_info.insert("animated".to_string(), "possible".to_string());
            }
            ImageFormat::WebP => {
                extra_info.insert("compression".to_string(), "both".to_string());
                extra_info.insert("animated".to_string(), "possible".to_string());
            }
            _ => {}
        }
        
        // Calculate aspect ratio
        let aspect_ratio = width as f64 / height as f64;
        extra_info.insert("aspect_ratio".to_string(), format!("{:.2}", aspect_ratio));
        
        // Calculate total pixels
        let total_pixels = width as u64 * height as u64;
        extra_info.insert("total_pixels".to_string(), format!("{}", total_pixels));
        
        // Calculate megapixels
        let megapixels = total_pixels as f64 / 1_000_000.0;
        extra_info.insert("megapixels".to_string(), format!("{:.1}MP", megapixels));
        
        ImageMetadata {
            width,
            height,
            format,
            file_size: Some(file_size),
            color_depth,
            has_alpha,
            extra_info,
        }
    }
    
    /// Check if format is supported
    pub fn is_format_supported(format: ImageFormat) -> bool {
        matches!(format, 
            ImageFormat::Png |
            ImageFormat::Jpeg |
            ImageFormat::Gif |
            ImageFormat::Bmp |
            ImageFormat::Ico |
            ImageFormat::Tiff |
            ImageFormat::WebP |
            ImageFormat::Pnm |
            ImageFormat::Dds |
            ImageFormat::Tga |
            ImageFormat::OpenExr |
            ImageFormat::Hdr |
            ImageFormat::Farbfeld |
            ImageFormat::Avif
        )
    }
    
    /// Get all supported file extensions
    pub fn supported_extensions() -> Vec<&'static str> {
        vec![
            "png", "jpg", "jpeg", "gif", "bmp", "ico", "tiff", "tif",
            "webp", "pnm", "pbm", "pgm", "ppm", "dds", "tga", "exr",
            "hdr", "farbfeld", "ff", "avif"
        ]
    }
    
    /// Check if file extension is supported
    pub fn is_extension_supported(extension: &str) -> bool {
        Self::format_from_extension(extension).is_some()
    }
    
    /// Get MIME type for image format
    pub fn format_to_mime_type(format: ImageFormat) -> &'static str {
        match format {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Gif => "image/gif",
            ImageFormat::Bmp => "image/bmp",
            ImageFormat::Ico => "image/x-icon",
            ImageFormat::Tiff => "image/tiff",
            ImageFormat::WebP => "image/webp",
            ImageFormat::Pnm => "image/x-portable-anymap",
            ImageFormat::Dds => "image/vnd.ms-dds",
            ImageFormat::Tga => "image/x-targa",
            ImageFormat::OpenExr => "image/x-exr",
            ImageFormat::Hdr => "image/vnd.radiance",
            ImageFormat::Farbfeld => "image/x-farbfeld",
            ImageFormat::Avif => "image/avif",
            _ => "application/octet-stream",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{RgbImage, Rgb};
    
    #[test]
    fn test_format_detection_from_extension() {
        assert_eq!(ImageLoader::format_from_extension("png"), Some(ImageFormat::Png));
        assert_eq!(ImageLoader::format_from_extension("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageLoader::format_from_extension("jpeg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageLoader::format_from_extension("gif"), Some(ImageFormat::Gif));
        assert_eq!(ImageLoader::format_from_extension("unknown"), None);
    }
    
    #[test]
    fn test_format_detection_from_url() {
        assert_eq!(
            ImageLoader::detect_format_from_url("https://example.com/image.png"),
            Some(ImageFormat::Png)
        );
        assert_eq!(
            ImageLoader::detect_format_from_url("https://example.com/image.jpg?param=value"),
            Some(ImageFormat::Jpeg)
        );
        assert_eq!(
            ImageLoader::detect_format_from_url("https://example.com/image"),
            None
        );
    }
    
    #[test]
    fn test_magic_bytes_detection() {
        // PNG signature
        let png_bytes = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(ImageLoader::detect_format_from_bytes(&png_bytes), Some(ImageFormat::Png));
        
        // JPEG signature
        let jpeg_bytes = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x00];
        assert_eq!(ImageLoader::detect_format_from_bytes(&jpeg_bytes), Some(ImageFormat::Jpeg));
        
        // GIF signature
        let gif_bytes = [0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(ImageLoader::detect_format_from_bytes(&gif_bytes), Some(ImageFormat::Gif));
        
        // Unknown format
        let unknown_bytes = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B];
        assert_eq!(ImageLoader::detect_format_from_bytes(&unknown_bytes), None);
    }
    
    #[test]
    fn test_supported_extensions() {
        let extensions = ImageLoader::supported_extensions();
        assert!(extensions.contains(&"png"));
        assert!(extensions.contains(&"jpg"));
        assert!(extensions.contains(&"gif"));
        assert!(!extensions.is_empty());
    }
    
    #[test]
    fn test_extension_support_check() {
        assert!(ImageLoader::is_extension_supported("png"));
        assert!(ImageLoader::is_extension_supported("jpg"));
        assert!(ImageLoader::is_extension_supported("jpeg"));
        assert!(!ImageLoader::is_extension_supported("unknown"));
    }
    
    #[test]
    fn test_format_support_check() {
        assert!(ImageLoader::is_format_supported(ImageFormat::Png));
        assert!(ImageLoader::is_format_supported(ImageFormat::Jpeg));
        assert!(ImageLoader::is_format_supported(ImageFormat::Gif));
    }
    
    #[test]
    fn test_mime_type_mapping() {
        assert_eq!(ImageLoader::format_to_mime_type(ImageFormat::Png), "image/png");
        assert_eq!(ImageLoader::format_to_mime_type(ImageFormat::Jpeg), "image/jpeg");
        assert_eq!(ImageLoader::format_to_mime_type(ImageFormat::Gif), "image/gif");
    }
    
    #[test]
    fn test_metadata_extraction() {
        // Create a test image
        let img = DynamicImage::ImageRgb8(RgbImage::from_pixel(100, 200, Rgb([255, 0, 0])));
        let metadata = ImageLoader::extract_metadata(&img, ImageFormat::Png, 1024);
        
        assert_eq!(metadata.width, 100);
        assert_eq!(metadata.height, 200);
        assert_eq!(metadata.format, ImageFormat::Png);
        assert_eq!(metadata.file_size, Some(1024));
        assert_eq!(metadata.color_depth, 24);
        assert!(!metadata.has_alpha);
        
        // Check extra info
        assert!(metadata.extra_info.contains_key("aspect_ratio"));
        assert!(metadata.extra_info.contains_key("total_pixels"));
        assert!(metadata.extra_info.contains_key("megapixels"));
    }
}