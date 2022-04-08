
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MetadataFormat {
    None,
    PlainText(String),
    Json(String)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CodepointType {
    Unspecified,
    Unicode,
    Indexed,
    Iconographic
}

impl Default for CodepointType {
    fn default() -> Self {
        Self::Unspecified
    }
}

impl From<u32> for CodepointType {
    fn from(i: u32) -> Self {
        match i {
            1 => Self::Unicode,
            2 => Self::Indexed,
            14 => Self::Iconographic,
            _ => Self::default()
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ImageType {
    None,
    SrgbImage,
    LinearMask,
    MaskedSrgbImage,
    Sdf,
    Psdf,
    Msdf,
    Mtsdf,
    MixedContent
}

impl Default for ImageType {
    fn default() -> Self {
        Self::None
    }
}

impl From<u32> for ImageType {
    fn from(i: u32) -> Self {
        match i {
            1 => Self::SrgbImage,
            2 => Self::LinearMask,
            3 => Self::MaskedSrgbImage,
            4 => Self::Sdf,
            5 => Self::Psdf,
            6 => Self::Msdf,
            7 => Self::Mtsdf,
            255 => Self::MixedContent,
            _ => Self::default()
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ImageEncoding {
    UnknownEncoding,
    RawBinary,
    Bmp,
    Tiff,
    Png,
    Tga
}

impl Default for ImageEncoding {
    fn default() -> Self {
        Self::UnknownEncoding
    }
}

impl From<u32> for ImageEncoding {
    fn from(i: u32) -> Self {
        match i {
            1 => Self::RawBinary,
            4 => Self::Bmp,
            5 => Self::Tiff,
            8 => Self::Png,
            9 => Self::Tga,
            _ => Self::default()
        }
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PixelFormat {
    Unknown,
    Boolean1,
    Unsigned8,
    Float32
}

impl Default for PixelFormat {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<u32> for PixelFormat {
    fn from(i: u32) -> Self {
        match i {
            1 => Self::Boolean1,
            8 => Self::Unsigned8,
            32 => Self::Float32,
            _ => Self::default()
        }
    }
}

impl PixelFormat {
    pub fn bits(self) -> usize {
        match self {
            PixelFormat::Unknown => 0,
            PixelFormat::Boolean1 => 1,
            PixelFormat::Unsigned8 => 8,
            PixelFormat::Float32 => 32,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ImageOrientation {
    BottomUp = -1,
    Unknown,
    TopDown = 1,
}

impl Default for ImageOrientation {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<i32> for ImageOrientation {
    fn from(i: i32) -> Self {
        match i {
            -1 => Self::BottomUp,
             1 => Self::TopDown,
             _ => Self::default()
        }
    }
}
