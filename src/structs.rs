use bytemuck::{Pod, Zeroable};
use crate::enums::*;
use crate::header::Real;

#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C)]
pub struct Advance {
    pub horizontal: Real,
    pub vertical: Real
}

#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C)]
pub struct Rect {
    pub left: Real,
    pub bottom: Real,
    pub right: Real,
    pub top: Real
}

impl Rect {
    pub fn is_empty(self) -> bool{
        self.top == self.bottom || self.left == self.right
    }

    pub fn scaled(self, x: Real, y: Real) -> Self {
        Self {
            left: self.left * x,
            bottom: self.bottom * y,
            right: self.right * x,
            top: self.top * y
        }
    }
}

#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C)]
pub struct Glyph {
    pub codepoint: u32,
    pub image: u32,
    pub plane_bounds: Rect,
    pub image_bounds: Rect,
    pub advance: Advance
}

impl Glyph {
    pub fn is_drawable(self) -> bool{
        !self.plane_bounds.is_empty() && !self.image_bounds.is_empty()
    }
}

#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C)]
pub struct KernPair {
    pub codepoint1: u32,
    pub codepoint2: u32,
    pub advance: Advance
}

#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C)]
pub struct FontMetric {
    pub font_size: Real,
    pub distance_range: Real,
    pub em_size: Real,
    pub ascender: Real,
    pub descender: Real,
    pub line_height: Real,
    pub underline_y: Real,
    pub underline_thickness: Real
}

#[derive(Debug, Clone)]
pub struct FontVariant {
    pub flags: u32,
    pub weight: u32,
    pub codepoint_type: CodepointType,
    pub image_type: ImageType,
    pub fallback_variant: u32,
    pub fallback_glyph: u32,
    pub metrics: FontMetric,
    pub name: String,
    pub metadata: String,
    pub glyphs: Vec<Glyph>,
    pub kern_pairs: Vec<KernPair>
}

#[derive(Debug, Clone, Copy)]
pub struct RawBinaryFormat {
    pub row_length: u32,
    pub orientation: ImageOrientation,
}

#[derive(Debug, Clone)]
pub struct Image {
    pub flags: u32,
    pub width: u32,
    pub height: u32,
    pub channels: u32,
    pub pixel_format: PixelFormat,
    pub image_type: ImageType,
    pub child_images: u32,
    pub texture_flags: u32,
    pub metadata: String,
    pub data: Vec<u8>
}

#[derive(Debug, Clone)]
pub struct Appendix {
    pub metadata: String,
    pub data: Vec<u8>
}

#[derive(Debug, Clone)]
pub struct ArteryFont {
    pub metadata_format: MetadataFormat,
    pub variants: Vec<FontVariant>,
    pub images: Vec<Image>,
    pub appendices: Vec<Appendix>
}
