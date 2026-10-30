use crate::{
    application::{RenderError, ports::PngEncoding},
    domain::Size,
};
use std::io::{self, Write};

struct BoundedPng {
    bytes: Vec<u8>,
    limit: usize,
    exceeded: bool,
}

impl Write for BoundedPng {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self
            .bytes
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > self.limit)
        {
            self.exceeded = true;
            return Err(io::Error::other("PNG byte budget exceeded"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct BoundedPngEncoder;

impl PngEncoding for BoundedPngEncoder {
    fn encode_rgba(&self, size: Size, rgba: &[u8], limit: usize) -> Result<Vec<u8>, RenderError> {
        encode(size, rgba, limit)
    }
}

fn encode(size: Size, rgba: &[u8], limit: usize) -> Result<Vec<u8>, RenderError> {
    let mut output = BoundedPng {
        bytes: Vec::new(),
        limit,
        exceeded: false,
    };
    let result = encode_into(size, rgba, &mut output);
    if output.exceeded {
        return Err(RenderError::ResourceLimit);
    }
    result.map_err(|e| RenderError::PngEncoding(e.to_string()))?;
    Ok(output.bytes)
}

fn encode_into(size: Size, rgba: &[u8], output: &mut BoundedPng) -> Result<(), png::EncodingError> {
    let mut encoder = png::Encoder::new(output, size.width, size.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(rgba)?;
    writer.finish()
}
