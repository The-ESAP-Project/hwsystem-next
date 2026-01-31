//! ⚠️ DEPRECATED: 此模块已被 `infer` crate 替代
//!
//! 请使用 `infer::get()` 进行文件类型检测，而不是 `validate_magic_bytes()`。
//! 保留此模块仅供参考，未来版本可能移除。

/// 验证文件内容的魔术字节是否与扩展名匹配
///
/// # Arguments
/// * `data` - 文件内容的前几个字节
/// * `extension` - 文件扩展名（包含点号，如 ".png"）
///
/// # Returns
/// * `true` - 魔术字节匹配或该类型不需要验证
/// * `false` - 魔术字节不匹配
pub fn validate_magic_bytes(data: &[u8], extension: &str) -> bool {
    if data.is_empty() {
        return false;
    }

    match extension.to_lowercase().as_str() {
        // 图片格式
        ".png" => data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
        ".jpg" | ".jpeg" => data.starts_with(&[0xFF, 0xD8, 0xFF]),
        ".gif" => data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a"),
        ".webp" => data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP",
        ".bmp" => data.starts_with(b"BM"),
        ".ico" => data.starts_with(&[0x00, 0x00, 0x01, 0x00]),

        // 文档格式
        ".pdf" => data.starts_with(b"%PDF"),
        ".doc" | ".xls" | ".ppt" => {
            // MS Office 旧格式 (OLE Compound Document)
            data.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1])
        }
        ".docx" | ".xlsx" | ".pptx" => {
            // MS Office 新格式 (ZIP-based OOXML)
            data.starts_with(&[0x50, 0x4B, 0x03, 0x04])
        }

        // 压缩格式
        ".zip" => data.starts_with(&[0x50, 0x4B, 0x03, 0x04]),
        ".rar" => data.starts_with(b"Rar!"),
        ".7z" => data.starts_with(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]),
        ".gz" | ".gzip" => data.starts_with(&[0x1F, 0x8B]),
        ".tar" => {
            // tar 文件在偏移 257 处有 "ustar" 标识，但前几个字节可能是任意的
            // 简化处理：允许 tar 文件
            true
        }

        // 文本格式 - 不检查魔术字节
        ".txt" | ".md" | ".json" | ".xml" | ".html" | ".css" | ".js" | ".ts" | ".csv" => true,

        // 未知格式 - 默认拒绝
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_magic() {
        let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(validate_magic_bytes(&png_header, ".png"));
        assert!(validate_magic_bytes(&png_header, ".PNG"));
        assert!(!validate_magic_bytes(&png_header, ".jpg"));
    }

    #[test]
    fn test_jpeg_magic() {
        let jpeg_header = [0xFF, 0xD8, 0xFF, 0xE0];
        assert!(validate_magic_bytes(&jpeg_header, ".jpg"));
        assert!(validate_magic_bytes(&jpeg_header, ".jpeg"));
        assert!(!validate_magic_bytes(&jpeg_header, ".png"));
    }

    #[test]
    fn test_pdf_magic() {
        let pdf_header = b"%PDF-1.4";
        assert!(validate_magic_bytes(pdf_header, ".pdf"));
        assert!(!validate_magic_bytes(pdf_header, ".doc"));
    }

    #[test]
    fn test_text_files() {
        let text_content = b"Hello, World!";
        assert!(validate_magic_bytes(text_content, ".txt"));
        assert!(validate_magic_bytes(text_content, ".md"));
        assert!(validate_magic_bytes(text_content, ".json"));
    }

    #[test]
    fn test_empty_data() {
        assert!(!validate_magic_bytes(&[], ".png"));
        assert!(!validate_magic_bytes(&[], ".txt"));
    }

    #[test]
    fn test_unknown_extension() {
        let data = [0x00, 0x01, 0x02, 0x03];
        assert!(!validate_magic_bytes(&data, ".exe"));
        assert!(!validate_magic_bytes(&data, ".unknown"));
    }
}
