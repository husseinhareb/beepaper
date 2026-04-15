//! Image decoding and software rendering into XRGB wallpaper pixels.

use std::path::Path;

use image::{DynamicImage, RgbaImage, imageops::FilterType};

use crate::{config::ApplyMode, error::WaylandError};

/// Rendered wallpaper pixel data ready for a shared-memory buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedImage {
    /// Output width in pixels.
    pub width: u32,
    /// Output height in pixels.
    pub height: u32,
    /// XRGB8888 bytes in native little-endian order.
    pub pixels: Vec<u8>,
}

/// A centered crop rectangle used to implement `fill`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CropRect {
    /// Left offset.
    pub x: u32,
    /// Top offset.
    pub y: u32,
    /// Crop width.
    pub width: u32,
    /// Crop height.
    pub height: u32,
}

/// Decode an image and render it for the target Wayland surface.
pub fn render_image(
    path: &Path,
    mode: ApplyMode,
    width: u32,
    height: u32,
) -> Result<RenderedImage, WaylandError> {
    let image = image::open(path)?;
    let rgba = match mode {
        ApplyMode::Fill => render_fill(&image, width, height),
    };

    Ok(RenderedImage {
        width,
        height,
        pixels: rgba_to_xrgb(&rgba),
    })
}

fn render_fill(image: &DynamicImage, width: u32, height: u32) -> RgbaImage {
    let crop = fill_crop_rect(image.width(), image.height(), width, height);
    let cropped = image.crop_imm(crop.x, crop.y, crop.width, crop.height);
    image::imageops::resize(&cropped.to_rgba8(), width, height, FilterType::Triangle)
}

/// Compute the centered crop rectangle needed to fill the destination aspect ratio.
pub fn fill_crop_rect(
    src_width: u32,
    src_height: u32,
    dst_width: u32,
    dst_height: u32,
) -> CropRect {
    let src_ratio_left = u64::from(src_width) * u64::from(dst_height);
    let src_ratio_right = u64::from(dst_width) * u64::from(src_height);

    if src_ratio_left > src_ratio_right {
        let width = ((u64::from(src_height) * u64::from(dst_width)) / u64::from(dst_height))
            .try_into()
            .expect("crop width fits in u32");
        let x = (src_width - width) / 2;

        CropRect {
            x,
            y: 0,
            width,
            height: src_height,
        }
    } else {
        let height = ((u64::from(src_width) * u64::from(dst_height)) / u64::from(dst_width))
            .try_into()
            .expect("crop height fits in u32");
        let y = (src_height - height) / 2;

        CropRect {
            x: 0,
            y,
            width: src_width,
            height,
        }
    }
}

fn rgba_to_xrgb(image: &RgbaImage) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((image.width() * image.height() * 4) as usize);

    for pixel in image.pixels() {
        let [r, g, b, a] = pixel.0;
        let alpha = u16::from(a);
        let r = ((u16::from(r) * alpha) / 255) as u8;
        let g = ((u16::from(g) * alpha) / 255) as u8;
        let b = ((u16::from(b) * alpha) / 255) as u8;

        bytes.extend_from_slice(&[b, g, r, 0x00]);
    }

    bytes
}

#[cfg(test)]
mod tests {
    use super::{CropRect, fill_crop_rect};

    #[test]
    fn fill_crop_centers_wide_images() {
        assert_eq!(
            fill_crop_rect(4000, 2000, 1000, 1000),
            CropRect {
                x: 1000,
                y: 0,
                width: 2000,
                height: 2000,
            }
        );
    }

    #[test]
    fn fill_crop_centers_tall_images() {
        assert_eq!(
            fill_crop_rect(2000, 4000, 1000, 1000),
            CropRect {
                x: 0,
                y: 1000,
                width: 2000,
                height: 2000,
            }
        );
    }

    #[test]
    fn fill_crop_keeps_matching_aspect_ratio() {
        assert_eq!(
            fill_crop_rect(1920, 1080, 1280, 720),
            CropRect {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            }
        );
    }
}
