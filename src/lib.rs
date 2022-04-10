mod enums;
mod header;
mod util;
mod structs;
#[cfg(not(feature = "no-checksum"))]
mod crc32;

use std::io::{Read};
use anyhow::{bail, ensure, Result};
use crate::util::ReadWrapper;
use crate::header::*;

pub use crate::enums::*;
pub use crate::structs::*;

impl ArteryFont {

    #[cfg(target_endian = "little")]
    pub fn read<R: Read>(reader: R) -> Result<Self> {

        let mut reader = ReadWrapper::new(reader);

        let font_header = reader.read_struct::<ArteryFontHeader>()?;
        ensure!(font_header.tag == *ARTERY_FONT_HEADER_TAG);
        ensure!(font_header.magic_no == ARTERY_FONT_HEADER_MAGIC_NO);
        ensure!(font_header.real_type == Real::type_code());

        let metadata_format = match font_header.metadata_format {
            0 => {
                ensure!(font_header.metadata_length == 0);
                MetadataFormat::None
            },
            1 => MetadataFormat::PlainText(reader.read_string(font_header.metadata_length as usize)??),
            2 => MetadataFormat::Json(reader.read_string(font_header.metadata_length as usize)??),
            _ => bail!("Unknown metadata format!")
        };

        let prev_length = reader.bytes_read();
        let mut variants = Vec::with_capacity(font_header.variant_count as usize);
        for _ in 0..font_header.variant_count {
            let variant_header = reader.read_struct::<FontVariantHeader>()?;
            variants.push(FontVariant {
                flags: variant_header.flags,
                weight: variant_header.weight,
                codepoint_type: CodepointType::from(variant_header.codepoint_type),
                image_type: ImageType::from(variant_header.image_type),
                fallback_variant: variant_header.fallback_variant,
                fallback_glyph: variant_header.fallback_glyph,
                metrics: bytemuck::cast_slice(&variant_header.metrics[..8])[0],
                name: reader.read_string(variant_header.name_length as usize)??,
                metadata: reader.read_string(variant_header.metadata_length as usize)??,
                glyphs: reader.read_struct_array(variant_header.glyph_count as usize)?,
                kern_pairs: reader.read_struct_array(variant_header.kern_pair_count as usize)?
            });
        }
        ensure!(reader.bytes_read() - prev_length == font_header.variants_length as usize);

        let prev_length = reader.bytes_read();
        let mut images = Vec::with_capacity(font_header.image_count as usize);
        for _ in 0..font_header.image_count {
            let image_header = reader.read_struct::<ImageHeader>()?;
            let encoding = ImageEncoding::from(image_header.encoding);
            let pixel_format = PixelFormat::from(image_header.pixel_format);
            let metadata = reader.read_string(image_header.metadata_length as usize)??;
            let data = match encoding {
                #[cfg(feature = "png")]
                ImageEncoding::Png => {
                    let mut decoder = png::Decoder::new((&mut reader).take(image_header.data_length as u64));
                    decoder.set_transformations(png::Transformations::EXPAND);
                    let mut reader = decoder.read_info()?;
                    let mut buf = vec![0u8; reader.output_buffer_size()];
                    let info = reader.next_frame(&mut buf)?;
                    ensure!(info.color_type.samples() == image_header.channels as usize);
                    ensure!(info.bit_depth as usize == pixel_format.bits());
                    ensure!(info.width == image_header.width);
                    ensure!(info.height == image_header.height);
                    flip_vertically(&mut buf, info.line_size);
                    buf
                },
                ImageEncoding::RawBinary => {
                    let mut data = reader.read_struct_array(image_header.data_length as usize)?;
                    match ImageOrientation::from(image_header.orientation) {
                        ImageOrientation::BottomUp => {},
                        ImageOrientation::TopDown => flip_vertically(&mut data, image_header.row_length as usize),
                        ImageOrientation::Unknown => bail!("Unknown orientation")
                    }
                    data
                }
                ImageEncoding::UnknownEncoding => bail!("Unknown encoding"),
                _ => bail!("Encoding {:?} not supported or enabled", encoding)
            };
            reader.realign()?;
            images.push(Image {
                flags: image_header.flags,
                width: image_header.width,
                height: image_header.height,
                channels: image_header.channels,
                pixel_format,
                image_type: ImageType::from(image_header.image_type),
                child_images: image_header.child_images,
                texture_flags: image_header.texture_flags,
                metadata,
                data
            });
        }
        ensure!(reader.bytes_read() - prev_length == font_header.images_length as usize);

        let prev_length = reader.bytes_read();
        let mut appendices = Vec::with_capacity(font_header.appendix_count as usize);
        for _ in 0..font_header.appendix_count {
            let appendix_header = reader.read_struct::<AppendixHeader>()?;
            appendices.push(Appendix {
                metadata: reader.read_string(appendix_header.metadata_length as usize)??,
                data: reader.read_struct_array(appendix_header.data_length as usize)?,
            });
            reader.realign()?;
        }
        ensure!(reader.bytes_read() - prev_length == font_header.appendix_count as usize);

        let footer = reader.read_struct::<ArteryFontFooter>()?;
        ensure!(footer.magic_no == ARTERY_FONT_FOOTER_MAGIC_NO);

        #[cfg(not(feature = "no-checksum"))]
        {
            let checksum = reader.checksum();
            let footer_checksum = reader.read_struct::<u32>()?;
            ensure!(checksum == footer_checksum);
        }
        #[cfg(feature = "no-checksum")]
        let _ = reader.read_struct::<u32>()?;

        ensure!(reader.bytes_read() == footer.total_length as usize);

        Ok(Self {
            metadata_format,
            variants,
            images,
            appendices
        })
    }

    #[cfg(not(target_endian = "little"))]
    pub fn read<R: Read>(reader: R) -> Result<!> {
        bail!("big endian is not supported")
    }

}

fn flip_vertically(data: &mut [u8], bytes_per_row: usize) {
    assert_eq!(data.len() % bytes_per_row, 0);
    let mut chunks = data.chunks_exact_mut(bytes_per_row);
    while chunks
        .next()
        .and_then(|v1| chunks
            .next_back()
            .map(|v2| v1.swap_with_slice(v2)))
        .is_some() {}
}

