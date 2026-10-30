//! Read-only fixed PCM lineage. No resampling, neural execution or video-time inference.
use crate::{infrastructure as io, sprite_metadata::guards};
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Pcm {
    pub(super) start: u64,
    pub(super) bytes: u64,
}
fn read(file: &mut File, bytes: &mut [u8]) -> Result<(), String> {
    file.read_exact(bytes).map_err(|e| e.to_string())
}
pub(super) fn layout(file: &mut File) -> Result<Pcm, String> {
    let length = file.metadata().map_err(|e| e.to_string())?.len();
    if !(44..=64 * 1024 * 1024).contains(&length) {
        return Err("bounded RIFF WAV required".into());
    }
    file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    let mut header = [0; 12];
    read(file, &mut header)?;
    if &header[..4] != b"RIFF"
        || &header[8..] != b"WAVE"
        || u64::from(u32::from_le_bytes(header[4..8].try_into().unwrap())) + 8 != length
    {
        return Err("invalid RIFF identity/length".into());
    }
    let mut position = 12;
    let mut format = false;
    let mut data = None;
    let mut count = 0;
    while position < length {
        count += 1;
        if count > 64 || length - position < 8 {
            return Err("invalid bounded WAV chunk table".into());
        }
        file.seek(SeekFrom::Start(position))
            .map_err(|e| e.to_string())?;
        let mut chunk = [0; 8];
        read(file, &mut chunk)?;
        let bytes = u64::from(u32::from_le_bytes(chunk[4..].try_into().unwrap()));
        let start = position + 8;
        let end = start
            .checked_add(bytes)
            .and_then(|end| end.checked_add(bytes % 2))
            .ok_or("chunk overflow")?;
        if end > length {
            return Err("chunk exceeds RIFF file".into());
        }
        match &chunk[..4] {
            b"fmt " => {
                if format || !(16..=40).contains(&bytes) {
                    return Err("duplicate or unsupported PCM format".into());
                }
                let mut fmt = [0; 16];
                read(file, &mut fmt)?;
                if u16::from_le_bytes(fmt[0..2].try_into().unwrap()) != 1
                    || u16::from_le_bytes(fmt[2..4].try_into().unwrap()) != 1
                    || u32::from_le_bytes(fmt[4..8].try_into().unwrap()) != 16000
                    || u32::from_le_bytes(fmt[8..12].try_into().unwrap()) != 32000
                    || u16::from_le_bytes(fmt[12..14].try_into().unwrap()) != 2
                    || u16::from_le_bytes(fmt[14..16].try_into().unwrap()) != 16
                {
                    return Err("expected mono 16kHz signed 16-bit PCM".into());
                }
                format = true;
            }
            b"data" => {
                if data.is_some() || bytes == 0 || !bytes.is_multiple_of(2) {
                    return Err("invalid or duplicate PCM data".into());
                }
                data = Some(Pcm { start, bytes });
            }
            _ => {}
        }
        position = end;
    }
    if !format {
        return Err("missing PCM format".into());
    }
    data.ok_or("missing PCM data".into())
}
pub(super) fn compare(
    full: &mut File,
    whole: Pcm,
    clip: &mut File,
    selected: Pcm,
    start_sample: u64,
) -> Result<(), String> {
    let offset = start_sample
        .checked_mul(2)
        .ok_or("sample offset overflow")?;
    if offset
        .checked_add(selected.bytes)
        .is_none_or(|end| end > whole.bytes)
    {
        return Err("clip exceeds whole PCM samples".into());
    }
    full.seek(SeekFrom::Start(whole.start + offset))
        .map_err(|e| e.to_string())?;
    clip.seek(SeekFrom::Start(selected.start))
        .map_err(|e| e.to_string())?;
    let mut remaining = selected.bytes;
    let mut left = [0; 8192];
    let mut right = [0; 8192];
    while remaining > 0 {
        let count = remaining.min(8192) as usize;
        read(full, &mut left[..count])?;
        read(clip, &mut right[..count])?;
        if let Some(offset) = left[..count]
            .iter()
            .zip(&right[..count])
            .position(|(a, b)| a != b)
        {
            return Err(format!(
                "clip PCM differs from original sample slice at byte {}",
                selected.bytes - remaining + offset as u64
            ));
        }
        remaining -= count as u64;
    }
    Ok(())
}
pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() != 2 {
        return Err("expected sprite-pcm-lineage ROOT NEW_RECEIPT_DIR".into());
    }
    let root = io::ordinary(Path::new(&args[0]))?;
    let output = PathBuf::from(&args[1]);
    if !io::ordinary(output.parent().ok_or("output parent missing")?)?
        .starts_with(root.join("runtime/verification"))
    {
        return Err("private new receipt directory required".into());
    }
    let folder = root.join("runtime/ai-spike/assets/sprite-clips-r1");
    let full = folder.join("full-mono-16k.wav");
    let clip = folder.join("latin-17.wav");
    let _full = guards(&full)?;
    let _clip = guards(&clip)?;
    let full_id = io::identity(&full)?;
    let clip_id = io::identity(&clip)?;
    if fs::metadata(&full).map_err(|e| e.to_string())?.len() != 20156486
        || fs::metadata(&clip).map_err(|e| e.to_string())?.len() != 272044
    {
        return Err("fixed PCM input length mismatch".into());
    }
    let full_sha = io::hash(&full)?;
    let clip_sha = io::hash(&clip)?;
    if full_sha != "872987BC1C6296B33241E833F5A315E8BD656B2D3ED05D14034231CAD4AE51A4"
        || clip_sha
            != "D12EC627660E5B3DE88579296D45B6F3CF368C7FE6E2419EA3ACBEb643C33270"
                .to_ascii_uppercase()
    {
        return Err("fixed PCM content identity mismatch".into());
    }
    fs::create_dir(&output).map_err(|e| e.to_string())?;
    io::write_new(&output.join("INPUT.txt"),format!("full={}\nfull_sha256={full_sha}\nfull_identity={full_id}\nclip={}\nclip_sha256={clip_sha}\nclip_identity={clip_id}\n",full.display(),clip.display()).as_bytes())?;
    let mut original = File::open(&full).map_err(|e| e.to_string())?;
    let mut selected = File::open(&clip).map_err(|e| e.to_string())?;
    let whole = layout(&mut original)?;
    let part = layout(&mut selected)?;
    if whole.bytes / 2 != 10078204 || part.bytes / 2 != 136000 {
        return Err("fixed PCM sample count mismatch".into());
    }
    compare(&mut original, whole, &mut selected, part, 272000)?;
    let unchanged = io::identity(&full)? == full_id && io::identity(&clip)? == clip_id;
    io::write_new(&output.join("RETURN.txt"),format!("full_data_offset={}\nfull_samples={}\nclip_data_offset={}\nclip_samples={}\nstart_sample=272000\nsample_rate=16000\nselected_pcm_bytes_equal=true\nguarded_identity_unchanged={unchanged}\nvideo_pts_mapping_verified=false\nactual9_binding=false\nquality_approved=false\n",whole.start,whole.bytes/2,part.start,part.bytes/2).as_bytes())?;
    if !unchanged {
        return Err("PCM guarded identity changed".into());
    }
    println!(
        "fixed existing PCM clip matches original sample slice; video PTS mapping remains unverified"
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    fn wav(samples: &[i16], junk: bool) -> Vec<u8> {
        let mut result = b"RIFF\0\0\0\0WAVEfmt ".to_vec();
        result.extend(16u32.to_le_bytes());
        result.extend(1u16.to_le_bytes());
        result.extend(1u16.to_le_bytes());
        result.extend(16000u32.to_le_bytes());
        result.extend(32000u32.to_le_bytes());
        result.extend(2u16.to_le_bytes());
        result.extend(16u16.to_le_bytes());
        if junk {
            result.extend(b"JUNK");
            result.extend(1u32.to_le_bytes());
            result.extend([99, 0]);
        }
        result.extend(b"data");
        result.extend((samples.len() as u32 * 2).to_le_bytes());
        for sample in samples {
            result.extend(sample.to_le_bytes());
        }
        let size = result.len() as u32 - 8;
        result[4..8].copy_from_slice(&size.to_le_bytes());
        result
    }
    fn file(bytes: &[u8]) -> File {
        let path = std::env::temp_dir().join(format!(
            "sprite-pcm-test-{}-{}.wav",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        io::write_new(&path, bytes).unwrap();
        File::open(path).unwrap()
    }
    #[test]
    fn chunk_layout_and_sample_slice_are_independent_of_wav_headers() {
        let mut full = file(&wav(&[1, 2, -3, 4, 5], true));
        let whole = layout(&mut full).unwrap();
        let mut clip = file(&wav(&[-3, 4], false));
        let part = layout(&mut clip).unwrap();
        assert_ne!(whole.start, part.start);
        compare(&mut full, whole, &mut clip, part, 2).unwrap();
        assert!(compare(&mut full, whole, &mut clip, part, 1).is_err());
        assert!(compare(&mut full, whole, &mut clip, part, 4).is_err());
        assert!(compare(&mut full, whole, &mut clip, part, u64::MAX).is_err());
    }
    #[test]
    fn rejects_wrong_format_and_truncated_chunk_without_guessing_samples() {
        let mut wrong = wav(&[1, 2], false);
        wrong[24..28].copy_from_slice(&8000u32.to_le_bytes());
        assert!(layout(&mut file(&wrong)).is_err());
        let mut bad = wav(&[1, 2], false);
        bad[40..44].copy_from_slice(&9999u32.to_le_bytes());
        assert!(layout(&mut file(&bad)).is_err());
        assert!(layout(&mut file(b"RIFF")).is_err());
    }
}
