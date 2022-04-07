use num_enum::TryFromPrimitive;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MetadataFormat {
    None,
    PlainText(String),
    Json(String)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub enum CodepointType {
    Unspecified = 0,
    Unicode = 1,
    Indexed = 2,
    Iconographic = 14
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub enum ImageType {
    None = 0,
    SrgbImage = 1,
    LinearMask = 2,
    MaskedSrgbImage = 3,
    Sdf = 4,
    Psdf = 5,
    Msdf = 6,
    Mtsdf = 7,
    MixedContent = 255
}