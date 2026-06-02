use std::path::Path;

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Rgb565,
    Bgra8888,
    Rgba8888,
    Gray8,
}

impl PixelFormat {
    pub fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Rgb565 => 2,
            Self::Bgra8888 | Self::Rgba8888 => 4,
            Self::Gray8 => 1,
        }
    }

    pub fn bits_per_pixel(self) -> u32 {
        (self.bytes_per_pixel() as u32) * 8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FramebufferInfo {
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub format: PixelFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FramebufferRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl FramebufferRect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn full(width: u32, height: u32) -> Self {
        Self::new(0, 0, width, height)
    }

    pub fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    fn clipped_to(self, width: u32, height: u32) -> Option<Self> {
        if self.x >= width || self.y >= height {
            return None;
        }
        let right = self.x.saturating_add(self.width).min(width);
        let bottom = self.y.saturating_add(self.height).min(height);
        let clipped = Self::new(self.x, self.y, right - self.x, bottom - self.y);
        (!clipped.is_empty()).then_some(clipped)
    }
}

pub trait Framebuffer {
    fn info(&self) -> FramebufferInfo;
    fn pixels(&self) -> &[u8];
    fn pixels_mut(&mut self) -> &mut [u8];
    fn mark_dirty(&mut self, rect: FramebufferRect);
    fn dirty_rects(&self) -> &[FramebufferRect];
    fn take_dirty_rects(&mut self) -> Vec<FramebufferRect>;

    fn width(&self) -> u32 {
        self.info().width
    }

    fn height(&self) -> u32 {
        self.info().height
    }

    fn stride(&self) -> usize {
        self.info().stride
    }

    fn pixel_format(&self) -> PixelFormat {
        self.info().format
    }

    fn clear(&mut self, byte: u8) {
        self.pixels_mut().fill(byte);
        let info = self.info();
        self.mark_dirty(FramebufferRect::full(info.width, info.height));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FramebufferSnapshot {
    pub info: FramebufferInfo,
    pub pixels: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualFramebuffer {
    info: FramebufferInfo,
    pixels: Vec<u8>,
    dirty_rects: Vec<FramebufferRect>,
}

impl VirtualFramebuffer {
    pub const DEFAULT_WIDTH: u32 = 800;
    pub const DEFAULT_HEIGHT: u32 = 480;
    pub const DEFAULT_FORMAT: PixelFormat = PixelFormat::Rgb565;

    pub fn new(width: u32, height: u32, format: PixelFormat) -> Result<Self> {
        let stride = checked_stride(width, format)?;
        Self::with_stride(width, height, stride, format)
    }

    pub fn default_primary() -> Result<Self> {
        Self::new(
            Self::DEFAULT_WIDTH,
            Self::DEFAULT_HEIGHT,
            Self::DEFAULT_FORMAT,
        )
    }

    pub fn with_stride(
        width: u32,
        height: u32,
        stride: usize,
        format: PixelFormat,
    ) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(Error::InvalidArgument(
                "framebuffer dimensions must be nonzero".to_owned(),
            ));
        }
        let minimum_stride = checked_stride(width, format)?;
        if stride < minimum_stride {
            return Err(Error::InvalidArgument(format!(
                "framebuffer stride {stride} is smaller than minimum {minimum_stride}"
            )));
        }
        let len = stride
            .checked_mul(height as usize)
            .ok_or_else(|| Error::InvalidArgument("framebuffer byte size overflow".to_owned()))?;
        Ok(Self {
            info: FramebufferInfo {
                width,
                height,
                stride,
                format,
            },
            pixels: vec![0; len],
            dirty_rects: vec![FramebufferRect::full(width, height)],
        })
    }

    pub fn snapshot(&self) -> FramebufferSnapshot {
        FramebufferSnapshot {
            info: self.info,
            pixels: self.pixels.clone(),
        }
    }

    pub fn write_ppm(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let mut bytes = format!("P6\n{} {}\n255\n", self.info.width, self.info.height).into_bytes();
        bytes.reserve((self.info.width as usize) * (self.info.height as usize) * 3);
        for y in 0..self.info.height as usize {
            let row = &self.pixels[y * self.info.stride..];
            for x in 0..self.info.width as usize {
                let offset = x * self.info.format.bytes_per_pixel();
                let rgb = pixel_to_rgb(self.info.format, &row[offset..]);
                bytes.extend_from_slice(&rgb);
            }
        }
        std::fs::write(path, bytes).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })
    }
}

impl Default for VirtualFramebuffer {
    fn default() -> Self {
        Self::default_primary().expect("default framebuffer dimensions are valid")
    }
}

impl Framebuffer for VirtualFramebuffer {
    fn info(&self) -> FramebufferInfo {
        self.info
    }

    fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    fn mark_dirty(&mut self, rect: FramebufferRect) {
        if let Some(rect) = rect.clipped_to(self.info.width, self.info.height) {
            self.dirty_rects.push(rect);
        }
    }

    fn dirty_rects(&self) -> &[FramebufferRect] {
        &self.dirty_rects
    }

    fn take_dirty_rects(&mut self) -> Vec<FramebufferRect> {
        std::mem::take(&mut self.dirty_rects)
    }
}

fn checked_stride(width: u32, format: PixelFormat) -> Result<usize> {
    (width as usize)
        .checked_mul(format.bytes_per_pixel())
        .ok_or_else(|| Error::InvalidArgument("framebuffer stride overflow".to_owned()))
}

fn pixel_to_rgb(format: PixelFormat, pixel: &[u8]) -> [u8; 3] {
    match format {
        PixelFormat::Rgb565 => {
            let raw = u16::from_le_bytes([pixel[0], pixel[1]]);
            let r = ((raw >> 11) & 0x1f) as u8;
            let g = ((raw >> 5) & 0x3f) as u8;
            let b = (raw & 0x1f) as u8;
            [
                (u16::from(r) * 255 / 31) as u8,
                (u16::from(g) * 255 / 63) as u8,
                (u16::from(b) * 255 / 31) as u8,
            ]
        }
        PixelFormat::Bgra8888 => [pixel[2], pixel[1], pixel[0]],
        PixelFormat::Rgba8888 => [pixel[0], pixel[1], pixel[2]],
        PixelFormat::Gray8 => [pixel[0], pixel[0], pixel[0]],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_framebuffer_uses_stride_and_tracks_initial_dirty_rect() {
        let fb = VirtualFramebuffer::new(320, 240, PixelFormat::Rgb565).unwrap();

        assert_eq!(fb.info().width, 320);
        assert_eq!(fb.info().height, 240);
        assert_eq!(fb.info().stride, 640);
        assert_eq!(fb.pixels().len(), 640 * 240);
        assert_eq!(fb.dirty_rects(), &[FramebufferRect::full(320, 240)]);
    }

    #[test]
    fn dirty_rects_are_clipped_to_the_framebuffer_bounds() {
        let mut fb = VirtualFramebuffer::new(10, 8, PixelFormat::Gray8).unwrap();
        let _ = fb.take_dirty_rects();

        fb.mark_dirty(FramebufferRect::new(8, 6, 20, 20));
        fb.mark_dirty(FramebufferRect::new(10, 0, 1, 1));

        assert_eq!(fb.dirty_rects(), &[FramebufferRect::new(8, 6, 2, 2)]);
    }

    #[test]
    fn clear_marks_the_full_framebuffer_dirty() {
        let mut fb = VirtualFramebuffer::new(4, 3, PixelFormat::Bgra8888).unwrap();
        let _ = fb.take_dirty_rects();

        fb.clear(0xaa);

        assert!(fb.pixels().iter().all(|byte| *byte == 0xaa));
        assert_eq!(fb.dirty_rects(), &[FramebufferRect::full(4, 3)]);
    }

    #[test]
    fn rejects_stride_smaller_than_one_scanline() {
        let err = VirtualFramebuffer::with_stride(10, 8, 9, PixelFormat::Rgb565).unwrap_err();

        assert!(err.to_string().contains("stride"));
    }
}
