mod enums;
mod crc32;

use std::fmt::{Debug};
use std::io::{Read};
use anyhow::{bail, ensure, Result};
use zerocopy::{FromBytes, AsBytes};
use crate::enums::{CodepointType, ImageEncoding, ImageOrientation, ImageType, MetadataFormat, PixelFormat};
use std::io::Result as IoResult;
use crate::crc32::Hasher;

type UtfResult<T> = std::result::Result<T, std::string::FromUtf8Error>;

const ARTERY_FONT_HEADER_TAG: &[u8; 16] = b"ARTERY/FONT\0\0\0\0\0";
// const ARTERY_FONT_HEADER_VERSION: u32 = 1;
const ARTERY_FONT_HEADER_MAGIC_NO: u32 = 0x4d276a5c;
const ARTERY_FONT_FOOTER_MAGIC_NO: u32 = 0x55ccb363;

type REAL = f32;

trait Real: AsBytes + FromBytes {
    fn type_code() -> u32;
}

impl Real for f32 {
    fn type_code() -> u32 {
        0x14
    }
}

impl Real for f64 {
    fn type_code() -> u32 {
        0x18
    }
}

#[derive(Debug, Copy, Clone, FromBytes, AsBytes)]
#[repr(C)]
struct ArteryFontHeader {
    tag: [u8; 16],
    magic_no: u32,
    version: u32,
    flags: u32,
    real_type: u32,
    reserved: [u32; 4],

    metadata_format: u32,
    metadata_length: u32,
    variant_count: u32,
    variants_length: u32,
    image_count: u32,
    images_length: u32,
    appendix_count: u32,
    appendices_length: u32,
    reserved2: [u32; 8]
}

#[derive(Debug, Copy, Clone, FromBytes, AsBytes)]
#[repr(C)]
struct ArteryFontFooter {
    salt: u32,
    magic_no: u32,
    reserved: [u32;4],
    total_length: u32
}


#[derive(Debug, Copy, Clone, FromBytes, AsBytes)]
#[repr(C)]
struct FontVariantHeader {
    flags: u32,
    weight: u32,
    codepoint_type: u32,
    image_type: u32,
    fallback_variant: u32,
    fallback_glyph: u32,
    reserved: [u32; 6],
    metrics: [REAL; 32],
    name_length: u32,
    metadata_length: u32,
    glyph_count: u32,
    kern_pair_count: u32,
}

#[derive(Debug, Copy, Clone, FromBytes, AsBytes)]
#[repr(C)]
struct ImageHeader {
    flags: u32,
    encoding: u32,
    width: u32,
    height: u32,
    channels: u32,
    pixel_format: u32,
    image_type: u32,
    row_length: u32,
    orientation: i32,
    child_images: u32,
    texture_flags: u32,
    reserved: [u32; 3],
    metadata_length: u32,
    data_length: u32
}

#[derive(Debug, Copy, Clone, FromBytes, AsBytes)]
#[repr(C)]
struct AppendixHeader {
    metadata_length: u32,
    data_length: u32
}

struct ReadWrapper<R> {
    inner: R,
    total_length: usize,
    checksum: Hasher
}

impl<R: Read> ReadWrapper<R> {

    fn new(inner: R) -> Self {
        Self {
            inner,
            total_length: 0,
            checksum: Hasher::new()
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> IoResult<()> {
        self.inner.read_exact(buf)?;
        self.total_length += buf.len();
        self.checksum.update(buf);
        Ok(())
    }

    fn read_struct<S: AsBytes + FromBytes>(&mut self) -> IoResult<S> {
        let mut result = S::new_zeroed();
        self.read(result.as_bytes_mut())?;
        Ok(result)
    }

    fn read_struct_array<S: AsBytes + FromBytes + Clone>(&mut self, len: usize) -> IoResult<Vec<S>> {
        let mut vec = vec![S::new_zeroed(); len];
        self.read(vec.as_bytes_mut())?;
        Ok(vec)
    }

    fn realign(&mut self) -> IoResult<()> {
        let mut dump = [0u8; 4];
        if self.total_length & 0x03 != 0 {
            let len = 0x04 - (self.total_length & 0x03);
            self.read(&mut dump[..len])?;
        }
        Ok(())
    }

    fn read_string(&mut self, len: usize) -> IoResult<UtfResult<String>> {
        if len > 0 {
            let mut buf = vec![0u8; len + 1];
            self.read(buf.as_mut_slice())?;
            self.realign()?;
            buf.pop();
            Ok(String::from_utf8(buf))
        } else {
            Ok(Ok(String::new()))
        }
    }

}

#[derive(Debug, Copy, Clone, FromBytes, AsBytes)]
#[repr(C)]
pub struct Advance {
    pub vertical: REAL,
    pub horizontal: REAL
}

#[derive(Debug, Copy, Clone, FromBytes, AsBytes)]
#[repr(C)]
pub struct Rect {
    pub left: REAL,
    pub bottom: REAL,
    pub right: REAL,
    pub top: REAL
}

#[derive(Debug, Copy, Clone, FromBytes, AsBytes)]
#[repr(C)]
pub struct Glyph {
    pub codepoint: u32,
    pub image: u32,
    pub plane_bounds: Rect,
    pub image_bounds: Rect,
    pub advance: Advance
}

#[derive(Debug, Copy, Clone, FromBytes, AsBytes)]
#[repr(C)]
pub struct KernPair {
    pub codepoint1: u32,
    pub codepoint2: u32,
    pub advance: Advance
}

#[derive(Debug, Copy, Clone, FromBytes)]
#[repr(C)]
pub struct FontMetric {
    pub font_size: REAL,
    pub distance_range: REAL,
    pub em_size: REAL,
    pub ascender: REAL,
    pub descender: REAL,
    pub line_height: REAL,
    pub underline_y: REAL,
    pub underline_thickness: REAL
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
    pub encoding: ImageEncoding,
    pub width: u32,
    pub height: u32,
    pub channels: u32,
    pub pixel_format: PixelFormat,
    pub image_type: ImageType,
    pub raw_binary_format: RawBinaryFormat,
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

impl ArteryFont {

    #[cfg(target_endian = "little")]
    pub fn read<R: Read>(reader: R) -> Result<Self> {


        let mut reader = ReadWrapper::new(reader);

        let font_header = reader.read_struct::<ArteryFontHeader>()?;
        ensure!(font_header.tag == *ARTERY_FONT_HEADER_TAG);
        ensure!(font_header.magic_no == ARTERY_FONT_HEADER_MAGIC_NO);
        ensure!(font_header.real_type == REAL::type_code());

        let metadata_format = match font_header.metadata_format {
            0 => {
                ensure!(font_header.metadata_length == 0);
                MetadataFormat::None
            },
            1 => MetadataFormat::PlainText(reader.read_string(font_header.metadata_length as usize)??),
            2 => MetadataFormat::Json(reader.read_string(font_header.metadata_length as usize)??),
            _ => bail!("Unknown metadata format!")
        };

        let prev_length = reader.total_length;
        let mut variants = Vec::with_capacity(font_header.variant_count as usize);
        for _ in 0..font_header.variant_count {
            let variant_header = reader.read_struct::<FontVariantHeader>()?;
            variants.push(FontVariant {
                flags: variant_header.flags,
                weight: variant_header.weight,
                codepoint_type: CodepointType::try_from(variant_header.codepoint_type)?,
                image_type: ImageType::try_from(variant_header.image_type)?,
                fallback_variant: variant_header.fallback_variant,
                fallback_glyph: variant_header.fallback_glyph,
                metrics: FontMetric::read_from_prefix(variant_header.metrics.as_bytes()).unwrap(),
                name: reader.read_string(variant_header.name_length as usize)??,
                metadata: reader.read_string(variant_header.metadata_length as usize)??,
                glyphs: reader.read_struct_array(variant_header.glyph_count as usize)?,
                kern_pairs: reader.read_struct_array(variant_header.kern_pair_count as usize)?
            });
        }
        ensure!(reader.total_length - prev_length == font_header.variants_length as usize);

        let prev_length = reader.total_length;
        let mut images = Vec::with_capacity(font_header.image_count as usize);
        for _ in 0..font_header.image_count {
            let image_header = reader.read_struct::<ImageHeader>()?;
            images.push(Image {
                flags: image_header.flags,
                encoding: ImageEncoding::try_from(image_header.encoding)?,
                width: image_header.width,
                height: image_header.height,
                channels: image_header.channels,
                pixel_format: PixelFormat::try_from(image_header.pixel_format)?,
                image_type: ImageType::try_from(image_header.image_type)?,
                raw_binary_format: RawBinaryFormat {
                    row_length: image_header.row_length,
                    orientation: ImageOrientation::try_from(image_header.orientation).unwrap_or(ImageOrientation::BottomUp)
                },
                child_images: image_header.child_images,
                texture_flags: image_header.texture_flags,
                metadata: reader.read_string(image_header.metadata_length as usize)??,
                data: reader.read_struct_array(image_header.data_length as usize)?
            });
            reader.realign()?;
        }
        ensure!(reader.total_length - prev_length == font_header.images_length as usize);

        let prev_length = reader.total_length;
        let mut appendices = Vec::with_capacity(font_header.appendix_count as usize);
        for _ in 0..font_header.appendix_count {
            let appendix_header = reader.read_struct::<AppendixHeader>()?;
            appendices.push(Appendix {
                metadata: reader.read_string(appendix_header.metadata_length as usize)??,
                data: reader.read_struct_array(appendix_header.data_length as usize)?,
            });
            reader.realign()?;
        }
        ensure!(reader.total_length - prev_length == font_header.appendix_count as usize);

        let footer = reader.read_struct::<ArteryFontFooter>()?;
        ensure!(footer.magic_no == ARTERY_FONT_FOOTER_MAGIC_NO);
        let checksum = std::mem::take(&mut reader.checksum).finalize();
        let footer_checksum = reader.read_struct::<u32>()?;
        ensure!(checksum == footer_checksum);
        ensure!(reader.total_length == footer.total_length as usize);

        Ok(Self {
            metadata_format,
            variants,
            images,
            appendices
        })
    }

}