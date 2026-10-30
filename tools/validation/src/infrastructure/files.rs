use anyhow::Result;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

pub fn json(path: &Path) -> Result<Value> {
    let text = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(text.trim_start_matches('\u{feff}'))?)
}

pub fn events(path: &Path, live: bool) -> Result<Vec<Value>> {
    let text = std::fs::read_to_string(path)?;
    let lines: Vec<_> = text.trim_start_matches('\u{feff}').lines().collect();
    let mut result = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        match serde_json::from_str(line) {
            Ok(value) => result.push(value),
            Err(_) if live && i + 1 == lines.len() => (),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(result)
}

pub fn hash(path: &Path) -> Result<String> {
    let mut file = BufReader::new(File::open(path)?);
    let mut hash = Sha256::new();
    loop {
        let buffer = file.fill_buf()?;
        if buffer.is_empty() {
            break;
        }
        hash.update(buffer);
        let consumed = buffer.len();
        file.consume(consumed);
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub fn write(path: &Path, value: &Value) -> Result<()> {
    std::fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

pub fn directory_bytes(path: &Path) -> Result<u64> {
    let mut bytes = 0;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        bytes += if entry.file_type()?.is_dir() {
            directory_bytes(&entry.path())?
        } else {
            entry.metadata()?.len()
        };
    }
    Ok(bytes)
}
