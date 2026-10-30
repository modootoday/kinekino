use super::*;
use crate::domain::snapshot_export::PixelCrop;
fn png() -> Vec<u8> {
    let mut b = Vec::new();
    {
        let mut e = png::Encoder::new(&mut b, 1, 1);
        e.set_color(png::ColorType::Rgba);
        let mut w = e.write_header().unwrap();
        w.write_image_data(&[1, 2, 3, 255]).unwrap();
        w.finish().unwrap();
    }
    b
}
#[test]
fn strict_inspection_rejects_trailing_truncated_crc_and_apng() {
    let bytes = png();
    assert_eq!(inspect_static_png(&bytes).unwrap(), (1, 1));
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_eq!(
        inspect_static_png(&trailing).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        inspect_static_png(&bytes[..bytes.len() - 1])
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    let mut crc = bytes;
    crc[29] ^= 1;
    assert_eq!(
        inspect_static_png(&crc).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let mut animated = Vec::new();
    {
        let mut e = png::Encoder::new(&mut animated, 1, 1);
        e.set_animated(1, 0).unwrap();
        let mut w = e.write_header().unwrap();
        w.write_image_data(&[1]).unwrap();
        w.finish().unwrap();
    }
    assert_eq!(
        inspect_static_png(&animated).unwrap_err().code,
        ErrorCode::Unsupported
    );
}
#[test]
fn crop_bounds_error_still_precedes_pixel_decode_error() {
    let mut bytes = png();
    let offset = bytes.windows(4).position(|x| x == b"IDAT").unwrap() + 4;
    bytes[offset] ^= 1;
    let error = raster::crop(
        &bytes,
        PixelCrop {
            x: 0,
            y: 0,
            width: 2,
            height: 1,
        },
    )
    .err()
    .unwrap();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(error.message.contains("crop") || error.message.contains("Crop"));
}
