//! Controlled finite media, generated in Rust. No network or external encoder.
use std::{fs, path::Path};

fn chunk(id: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::from(*id);
    bytes.extend_from_slice(&u32::try_from(body.len()).unwrap().to_le_bytes());
    bytes.extend_from_slice(body);
    if !body.len().is_multiple_of(2) {
        bytes.push(0);
    }
    bytes
}
fn words(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|word| word.to_le_bytes()).collect()
}
fn list(kind: &[u8; 4], chunks: &[Vec<u8>]) -> Vec<u8> {
    let mut body = Vec::from(*kind);
    for child in chunks {
        body.extend_from_slice(child);
    }
    chunk(b"LIST", &body)
}
pub(super) fn native_video(path: &Path) {
    let avih = chunk(
        b"avih",
        &words(&[100_000, 30_720, 0, 0, 100, 0, 1, 3072, 32, 32, 0, 0, 0, 0]),
    );
    let mut stream = b"vidsDIB ".to_vec();
    stream.extend_from_slice(&words(&[0, 0, 0, 1, 10, 0, 100, 3072, u32::MAX, 0]));
    for coordinate in [0i16, 0, 32, 32] {
        stream.extend_from_slice(&coordinate.to_le_bytes());
    }
    assert_eq!(stream.len(), 56);
    let mut bitmap = words(&[40, 32, 32]);
    bitmap.extend_from_slice(&1u16.to_le_bytes());
    bitmap.extend_from_slice(&24u16.to_le_bytes());
    bitmap.extend_from_slice(&words(&[0, 3072, 0, 0, 0, 0]));
    assert_eq!(bitmap.len(), 40);
    let header = list(
        b"hdrl",
        &[
            avih,
            list(b"strl", &[chunk(b"strh", &stream), chunk(b"strf", &bitmap)]),
        ],
    );
    let frames = (0..100u8)
        .map(|index| chunk(b"00db", &[index; 3072]))
        .collect::<Vec<_>>();
    let mut body = b"AVI ".to_vec();
    body.extend_from_slice(&header);
    body.extend_from_slice(&list(b"movi", &frames));
    fs::write(path, chunk(b"RIFF", &body)).unwrap();
}
pub(super) fn browser_audio_html() -> String {
    use base64::Engine;
    let mut format = Vec::new();
    for value in [1u16, 1] {
        format.extend_from_slice(&value.to_le_bytes());
    }
    format.extend_from_slice(&words(&[8_000, 16_000]));
    for value in [2u16, 16] {
        format.extend_from_slice(&value.to_le_bytes());
    }
    let samples = (0..16_000)
        .flat_map(|index| {
            let sample =
                ((f64::from(index) * 440. * std::f64::consts::TAU / 8_000.).sin() * 4096.) as i16;
            sample.to_le_bytes()
        })
        .collect::<Vec<_>>();
    let mut wave = b"WAVE".to_vec();
    wave.extend_from_slice(&chunk(b"fmt ", &format));
    wave.extend_from_slice(&chunk(b"data", &samples));
    let data = base64::engine::general_purpose::STANDARD.encode(chunk(b"RIFF", &wave));
    format!(
        "<!doctype html><meta charset=utf-8><title>Owned audio guard fixture</title><audio autoplay src=\"data:audio/wav;base64,{data}\"></audio>"
    )
}
