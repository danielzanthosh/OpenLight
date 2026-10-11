use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use image::{imageops::FilterType, DynamicImage, ImageFormat, RgbImage};
use std::io::Cursor;
use xcap::Monitor;

#[derive(Debug, Clone)]
pub struct ScreenInfo {
    pub index: usize,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

pub struct ScreenManager {
    pub max_dimension: u32,
    pub default_monitor: usize,
}

impl ScreenManager {
    pub fn new(max_dimension: u32, default_monitor: usize) -> Self {
        Self {
            max_dimension,
            default_monitor,
        }
    }

    pub fn list_monitors(&self) -> Vec<ScreenInfo> {
        let mut list = Vec::new();
        if let Ok(monitors) = Monitor::all() {
            for (idx, m) in monitors.into_iter().enumerate() {
                list.push(ScreenInfo {
                    index: idx + 1,
                    x: m.x(),
                    y: m.y(),
                    width: m.width(),
                    height: m.height(),
                    is_primary: m.is_primary(),
                });
            }
        }
        if list.is_empty() {
            list.push(ScreenInfo {
                index: 1,
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
                is_primary: true,
            });
        }
        list
    }

    pub fn capture(
        &self,
        monitor_idx: Option<usize>,
    ) -> Result<(DynamicImage, String, (u32, u32), ScreenInfo), Box<dyn std::error::Error + Send + Sync>> {
        let target_idx = monitor_idx.unwrap_or(self.default_monitor);
        let monitors = self.list_monitors();
        let screen_info = monitors
            .iter()
            .find(|m| m.index == target_idx)
            .cloned()
            .unwrap_or_else(|| monitors[0].clone());

        // Attempt hardware/OS screen capture via xcap
        let img = match Monitor::all().ok().and_then(|ms| ms.into_iter().nth(target_idx.saturating_sub(1))) {
            Some(monitor) => match monitor.capture_image() {
                Ok(rgba) => DynamicImage::ImageRgba8(rgba),
                Err(_) => Self::generate_fallback_image(screen_info.width, screen_info.height),
            },
            None => Self::generate_fallback_image(screen_info.width, screen_info.height),
        };

        // Resize down to max_dimension for efficient LLM vision tokens
        let orig_w = img.width();
        let orig_h = img.height();
        let scale = if orig_w.max(orig_h) > self.max_dimension {
            self.max_dimension as f32 / orig_w.max(orig_h) as f32
        } else {
            1.0
        };

        let scaled_w = (orig_w as f32 * scale) as u32;
        let scaled_h = (orig_h as f32 * scale) as u32;

        let resized = if scale < 1.0 {
            img.resize(scaled_w, scaled_h, FilterType::Lanczos3)
        } else {
            img
        };

        // Encode to JPEG base64
        let mut buf = Cursor::new(Vec::new());
        resized.write_to(&mut buf, ImageFormat::Jpeg)?;
        let b64 = BASE64.encode(buf.into_inner());

        Ok((resized, b64, (scaled_w, scaled_h), screen_info))
    }

    fn generate_fallback_image(width: u32, height: u32) -> DynamicImage {
        let w = if width > 0 { width } else { 1920 };
        let h = if height > 0 { height } else { 1080 };
        let rgb = RgbImage::from_pixel(w, h, image::Rgb([35, 39, 46]));
        DynamicImage::ImageRgb8(rgb)
    }

    pub fn scale_to_screen_coords(
        x: u32,
        y: u32,
        scaled_dim: (u32, u32),
        screen_info: &ScreenInfo,
    ) -> (i32, i32) {
        let (sw, sh) = scaled_dim;
        let norm_x = x as f32 / sw as f32;
        let norm_y = y as f32 / sh as f32;
        let real_x = screen_info.x + (norm_x * screen_info.width as f32) as i32;
        let real_y = screen_info.y + (norm_y * screen_info.height as f32) as i32;
        (real_x, real_y)
    }
}
