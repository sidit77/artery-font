# artery-font
A pure Rust parser for Artery Atlas font files.
An Artery Atlas font file (*.arfont) wraps together the atlas bitmap(s), which can be compressed e.g. in PNG format, the layout of the atlas, as well as the font's and the individual glyphs' metrics and positioning data, including kerning pairs.

Artery Atlas font files can be generated using the [Multi-channel signed distance field atlas generator](https://github.com/Chlumsky/msdf-atlas-gen).

This is a port of the [C++ Reference Implementation](https://github.com/Chlumsky/artery-font-format).

Currently only PNG and RawBinary are supported as images.

## Example
```rust
let arfont = ArteryFont::read(&include_bytes!("../data/test.arfont")[..]).unwrap();
let image = arfont.images.first().unwrap();
let variant = arfont.variants.first().unwrap();
assert_eq!(variant.image_type, ImageType::Msdf);
assert_eq!(variant.codepoint_type, CodepointType::Unicode);
let line_height = variant.metrics.line_height;
```
See the [full Example](https://github.com/sidit77/artery-font/blob/main/example/src/main.rs)

## Cargo features
* `double`: Configures this library to use `f64` instead of `f32` for floating point values. Needs to match the exporter.
* `no-checksum`: Disables checksum calculation and verification. Note: this flag only affects this library and has no effect on the embedded image loading crates.
* `png`: enables support for png compression

## License
MIT License