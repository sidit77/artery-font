use bytemuck::{Zeroable, Pod};

pub const ARTERY_FONT_HEADER_TAG: &[u8; 16] = b"ARTERY/FONT\0\0\0\0\0";
pub const ARTERY_FONT_HEADER_MAGIC_NO: u32 = 0x4d276a5c;
pub const ARTERY_FONT_FOOTER_MAGIC_NO: u32 = 0x55ccb363;

#[cfg(not(feature = "double"))]
pub type Real = f32;

#[cfg(feature = "double")]
pub type Real = f64;

pub trait TypeCode {
    fn type_code() -> u32;
}

impl TypeCode for f32 {
    fn type_code() -> u32 {
        0x14
    }
}

impl TypeCode for f64 {
    fn type_code() -> u32 {
        0x18
    }
}

#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C)]
pub struct ArteryFontHeader {
    pub tag: [u8; 16],
    pub magic_no: u32,
    pub version: u32,
    pub flags: u32,
    pub real_type: u32,
    pub reserved: [u32; 4],
    pub metadata_format: u32,
    pub metadata_length: u32,
    pub variant_count: u32,
    pub variants_length: u32,
    pub image_count: u32,
    pub images_length: u32,
    pub appendix_count: u32,
    pub appendices_length: u32,
    pub reserved2: [u32; 8]
}

#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C)]
pub struct ArteryFontFooter {
    pub salt: u32,
    pub magic_no: u32,
    pub reserved: [u32;4],
    pub total_length: u32
}


#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C)]
pub struct FontVariantHeader {
    pub flags: u32,
    pub weight: u32,
    pub codepoint_type: u32,
    pub image_type: u32,
    pub fallback_variant: u32,
    pub fallback_glyph: u32,
    pub reserved: [u32; 6],
    pub metrics: [Real; 32],
    pub name_length: u32,
    pub metadata_length: u32,
    pub glyph_count: u32,
    pub kern_pair_count: u32,
}

#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C)]
pub struct ImageHeader {
    pub flags: u32,
    pub encoding: u32,
    pub width: u32,
    pub height: u32,
    pub channels: u32,
    pub pixel_format: u32,
    pub image_type: u32,
    pub row_length: u32,
    pub orientation: i32,
    pub child_images: u32,
    pub texture_flags: u32,
    pub reserved: [u32; 3],
    pub metadata_length: u32,
    pub data_length: u32
}

#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C)]
pub struct AppendixHeader {
    pub metadata_length: u32,
    pub data_length: u32
}
