use image::{ImageBuffer, Luma};

/// Generates PNG image bytes for a QR code from a given string payload.
/// Telegram sendPhoto method requires raster/vector formats (PNG/JPG/WEBP) and rejects SVG.
pub fn generate_qr_code_png_bytes(payload: &str) -> Result<Vec<u8>, String> {
    let code = qrcode::QrCode::new(payload).map_err(|e| format!("QR Code Error: {}", e))?;
    let image: ImageBuffer<Luma<u8>, Vec<u8>> =
        code.render::<Luma<u8>>().min_dimensions(300, 300).build();

    let mut png_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    image
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("PNG Encoding Error: {}", e))?;

    Ok(png_bytes)
}
