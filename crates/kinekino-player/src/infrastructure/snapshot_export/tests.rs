use super::*;
use crate::{application::snapshot_export::export, domain::snapshot_export::PixelCrop};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn encode(
    color: png::ColorType,
    depth: png::BitDepth,
    pixels: &[u8],
    palette: bool,
    animated: bool,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, 4, 2);
        encoder.set_color(color);
        encoder.set_depth(depth);
        if palette {
            encoder.set_palette(vec![0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110]);
            encoder.set_trns(vec![255, 128, 64, 0]);
        }
        if animated {
            encoder.set_animated(1, 0).unwrap();
        }
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(pixels).unwrap();
        writer.finish().unwrap();
    }
    bytes
}
fn decode(bytes: &[u8]) -> (png::OutputInfo, Vec<u8>) {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().unwrap();
    let mut output = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut output).unwrap();
    reader.finish().unwrap();
    output.truncate(info.buffer_size());
    (info, output)
}
fn rect() -> PixelCrop {
    PixelCrop {
        x: 1,
        y: 0,
        width: 2,
        height: 2,
    }
}
fn assert_pixels(bytes: &[u8]) {
    let result = raster::crop(bytes, rect()).unwrap();
    assert_eq!((result.width, result.height), (4, 2));
    let (source, source_pixels) = decode(bytes);
    let (output, output_pixels) = decode(&result.bytes);
    assert_eq!((output.width, output.height), (2, 2));
    assert_eq!(output.color_type, source.color_type);
    assert_eq!(output.bit_depth, source.bit_depth);
    let pixel_bytes = source.line_size / 4;
    let expected = (0..2usize)
        .flat_map(|row| {
            source_pixels
                [row * source.line_size + pixel_bytes..row * source.line_size + 3 * pixel_bytes]
                .iter()
                .copied()
        })
        .collect::<Vec<_>>();
    assert_eq!(output_pixels, expected);
}

#[test]
fn crops_actual_eight_and_sixteen_bit_samples_without_rounding_or_alpha_changes() {
    for color in [
        png::ColorType::Grayscale,
        png::ColorType::GrayscaleAlpha,
        png::ColorType::Rgb,
        png::ColorType::Rgba,
    ] {
        for depth in [png::BitDepth::Eight, png::BitDepth::Sixteen] {
            let sample_bytes = if depth == png::BitDepth::Sixteen {
                2
            } else {
                1
            };
            let pixels = (0..8 * color.samples() * sample_bytes)
                .map(|n| (n * 37 % 256) as u8)
                .collect::<Vec<_>>();
            assert_pixels(&encode(color, depth, &pixels, false, false));
        }
    }
}
#[test]
fn crops_palette_transparency_and_subbyte_grayscale_as_equivalent_expanded_pixels() {
    assert_pixels(&encode(
        png::ColorType::Indexed,
        png::BitDepth::Two,
        &[0x1b, 0xe4],
        true,
        false,
    ));
    assert_pixels(&encode(
        png::ColorType::Grayscale,
        png::BitDepth::One,
        &[0x50, 0xa0],
        false,
        false,
    ));
}
#[test]
fn rejects_animation_instead_of_silently_exporting_its_first_frame() {
    let bytes = encode(
        png::ColorType::Grayscale,
        png::BitDepth::Eight,
        &[1; 8],
        false,
        true,
    );
    assert_eq!(
        raster::crop(&bytes, rect()).err().unwrap().code,
        ErrorCode::Unsupported
    );
}
#[test]
fn rejects_truncation_crc_trailing_data_and_out_of_bounds() {
    let bytes = encode(
        png::ColorType::Grayscale,
        png::BitDepth::Eight,
        &[1; 8],
        false,
        false,
    );
    assert!(raster::crop(&bytes[..bytes.len() - 1], rect()).is_err());
    let mut damaged = bytes.clone();
    damaged[29] ^= 1; // IHDR CRC.
    assert!(raster::crop(&damaged, rect()).is_err());
    let mut damaged_data = bytes.clone();
    let mut offset = 8usize;
    loop {
        let length = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        if &bytes[offset + 4..offset + 8] == b"IDAT" {
            damaged_data[offset + 8 + length] ^= 1;
            break;
        }
        offset += length + 12;
    }
    assert!(raster::crop(&damaged_data, rect()).is_err());
    let mut damaged_end = bytes.clone();
    *damaged_end.last_mut().unwrap() ^= 1; // IEND CRC must also be consumed.
    assert!(raster::crop(&damaged_end, rect()).is_err());
    let mut trailing = bytes.clone();
    trailing.extend_from_slice(b"unrelated trailing bytes");
    assert!(raster::crop(&trailing, rect()).is_err());
    assert!(raster::crop(&bytes, PixelCrop { x: 3, ..rect() }).is_err());
    assert!(
        raster::crop(
            &bytes,
            PixelCrop {
                x: u32::MAX,
                ..rect()
            }
        )
        .is_err()
    );
}

#[test]
fn rejects_shared_pixel_and_decoded_byte_budgets_even_for_a_tiny_crop() {
    for (width, height, color, depth, count) in [
        (
            4000,
            2200,
            png::ColorType::Grayscale,
            png::BitDepth::Eight,
            4000 * 2200,
        ),
        (
            3000,
            1500,
            png::ColorType::Rgba,
            png::BitDepth::Sixteen,
            3000 * 1500 * 8,
        ),
    ] {
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, width, height);
            encoder.set_color(color);
            encoder.set_depth(depth);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&vec![0; count]).unwrap();
            writer.finish().unwrap();
        }
        assert_eq!(
            raster::crop(&bytes, rect()).err().unwrap().code,
            ErrorCode::ResourceExhausted
        );
    }
}

struct Fixture {
    root: PathBuf,
    source: Vec<u8>,
    request: ExportSnapshot,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "snapshot-export-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let source = encode(
            png::ColorType::Rgb,
            png::BitDepth::Eight,
            &(0..24).collect::<Vec<u8>>(),
            false,
            false,
        );
        let source_path = root.join("source.png");
        fs::write(&source_path, &source).unwrap();
        let hash = content_digest::sha256(&mut Cursor::new(&source), MAX_FRAME_PNG_BYTES)
            .unwrap()
            .1;
        let request = ExportSnapshot {
            source_path: source_path.to_string_lossy().into(),
            expected_source_sha256: hash,
            output_path: root.join("crop.png").to_string_lossy().into(),
            crop: rect(),
        };
        Self {
            root,
            source,
            request,
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn actual_file_export_preserves_original_and_publishes_verified_crop_without_overwrite() {
    let fixture = Fixture::new();
    let result = export(&PngSnapshotExport, &fixture.request).unwrap();
    assert_eq!(
        fs::read(&fixture.request.source_path).unwrap(),
        fixture.source
    );
    let output = fs::read(&result.output_path).unwrap();
    assert_eq!(output, raster::crop(&fixture.source, rect()).unwrap().bytes);
    assert_eq!(result.source_sha256, fixture.request.expected_source_sha256);
    assert_eq!(
        result.output_sha256,
        content_digest::sha256(&mut Cursor::new(&output), MAX_FRAME_PNG_BYTES)
            .unwrap()
            .1
    );
    assert_eq!(result.output_bytes, output.len() as u64);
    assert!(!result.metadata_preserved);
    assert_eq!(
        export(&PngSnapshotExport, &fixture.request)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(fs::read(&result.output_path).unwrap(), output);
    assert_eq!(
        fs::read(&fixture.request.source_path).unwrap(),
        fixture.source
    );
}
#[test]
fn changed_source_same_path_alias_and_invalid_input_publish_nothing() {
    let mut fixture = Fixture::new();
    fixture.request.expected_source_sha256 = "0".repeat(64);
    assert_eq!(
        export(&PngSnapshotExport, &fixture.request)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert!(!Path::new(&fixture.request.output_path).exists());
    fixture.request.expected_source_sha256 =
        content_digest::sha256(&mut Cursor::new(&fixture.source), MAX_FRAME_PNG_BYTES)
            .unwrap()
            .1;
    fixture.request.output_path = fixture.root.join("alias.png").to_string_lossy().into();
    fs::hard_link(&fixture.request.source_path, &fixture.request.output_path).unwrap();
    assert_eq!(
        export(&PngSnapshotExport, &fixture.request)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        fs::read(&fixture.request.source_path).unwrap(),
        fixture.source
    );
}

#[test]
fn oversized_input_is_rejected_without_output_or_source_rewrite() {
    let fixture = Fixture::new();
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(&fixture.request.source_path)
        .unwrap();
    file.set_len(MAX_FRAME_PNG_BYTES + 1).unwrap();
    assert_eq!(
        export(&PngSnapshotExport, &fixture.request)
            .unwrap_err()
            .code,
        ErrorCode::ResourceExhausted
    );
    assert!(!Path::new(&fixture.request.output_path).exists());
    assert_eq!(
        fs::metadata(&fixture.request.source_path).unwrap().len(),
        MAX_FRAME_PNG_BYTES + 1
    );
}

#[test]
fn corrupted_source_with_matching_hash_is_not_published() {
    let mut fixture = Fixture::new();
    let mut damaged = fixture.source.clone();
    damaged[29] ^= 1;
    fs::write(&fixture.request.source_path, &damaged).unwrap();
    fixture.request.expected_source_sha256 =
        content_digest::sha256(&mut Cursor::new(&damaged), MAX_FRAME_PNG_BYTES)
            .unwrap()
            .1;
    assert_eq!(
        export(&PngSnapshotExport, &fixture.request)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert!(!Path::new(&fixture.request.output_path).exists());
    assert_eq!(fs::read(&fixture.request.source_path).unwrap(), damaged);
}

#[test]
fn ancillary_crc_failure_after_image_data_with_matching_hash_is_not_published() {
    let mut fixture = Fixture::new();
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, 4, 2);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&[1; 8]).unwrap();
        writer
            .write_chunk(png::chunk::tEXt, b"Comment\0source metadata")
            .unwrap();
        writer.finish().unwrap();
    }
    fs::write(&fixture.request.source_path, &bytes).unwrap();
    fixture.request.expected_source_sha256 =
        content_digest::sha256(&mut Cursor::new(&bytes), MAX_FRAME_PNG_BYTES)
            .unwrap()
            .1;
    let normal = export(&PngSnapshotExport, &fixture.request).unwrap();
    assert!(!normal.metadata_preserved);
    assert_eq!(fs::read(&fixture.request.source_path).unwrap(), bytes);
    fs::remove_file(&fixture.request.output_path).unwrap();

    let mut offset = 8usize;
    loop {
        let length = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        if &bytes[offset + 4..offset + 8] == b"tEXt" {
            bytes[offset + 8 + length] ^= 1; // Only the ancillary chunk CRC changes.
            break;
        }
        assert_ne!(
            &bytes[offset + 4..offset + 8],
            b"IEND",
            "Fixture must contain tEXt"
        );
        offset += length + 12;
    }
    fs::write(&fixture.request.source_path, &bytes).unwrap();
    fixture.request.expected_source_sha256 =
        content_digest::sha256(&mut Cursor::new(&bytes), MAX_FRAME_PNG_BYTES)
            .unwrap()
            .1;
    assert_eq!(
        export(&PngSnapshotExport, &fixture.request)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert!(!Path::new(&fixture.request.output_path).exists());
    assert_eq!(fs::read(&fixture.request.source_path).unwrap(), bytes);
}

#[test]
fn invalid_zlib_adler32_with_valid_chunk_crc_and_matching_hash_is_not_published() {
    let mut fixture = Fixture::new();
    let normal = encode(
        png::ColorType::Grayscale,
        png::BitDepth::Eight,
        &[1; 8],
        false,
        false,
    );
    let mut compressed = Vec::new();
    let mut offset = 8usize;
    loop {
        let length = u32::from_be_bytes(normal[offset..offset + 4].try_into().unwrap()) as usize;
        if &normal[offset + 4..offset + 8] == b"IDAT" {
            compressed.extend_from_slice(&normal[offset + 8..offset + 8 + length]);
        }
        if &normal[offset + 4..offset + 8] == b"IEND" {
            break;
        }
        offset += length + 12;
    }
    assert!(compressed.len() >= 6);
    *compressed.last_mut().unwrap() ^= 1; // Only the zlib trailer Adler32 changes.
    let mut damaged = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut damaged, 4, 2);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        // The existing writer computes a fresh valid PNG CRC over the altered zlib stream.
        writer.write_chunk(png::chunk::IDAT, &compressed).unwrap();
        writer.finish().unwrap();
    }
    // Pinned defaults still decode these samples: this fixture isolates ignored Adler32,
    // rather than damage to the PNG chunk CRC or the deflate payload.
    assert_eq!(decode(&damaged).1, vec![1; 8]);
    fs::write(&fixture.request.source_path, &damaged).unwrap();
    fixture.request.expected_source_sha256 =
        content_digest::sha256(&mut Cursor::new(&damaged), MAX_FRAME_PNG_BYTES)
            .unwrap()
            .1;
    assert_eq!(
        export(&PngSnapshotExport, &fixture.request)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert!(!Path::new(&fixture.request.output_path).exists());
    assert_eq!(fs::read(&fixture.request.source_path).unwrap(), damaged);
}
