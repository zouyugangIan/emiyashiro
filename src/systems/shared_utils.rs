//! Shared utility functions for file operations, compression, and checksums.

use atomicwrites::{AtomicFile, OverwriteBehavior};
use flate2::read::GzDecoder;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{self, Cursor, Read, Write};
use std::path::Path;

const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];
const ZSTD_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];

/// Compresses data using Zstd.
///
/// # Arguments
/// * `data` - The raw byte slice to compress.
/// * `level` - The compression level (0-19).
pub fn compress_data(data: &[u8], level: u32) -> Result<Vec<u8>, std::io::Error> {
    let zstd_level = level.min(19) as i32;
    zstd::stream::encode_all(Cursor::new(data), zstd_level)
}

/// Decompresses data using supported compression formats.
///
/// # Arguments
/// * `data` - The compressed byte slice.
pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    if data.starts_with(&ZSTD_MAGIC) {
        return zstd::stream::decode_all(Cursor::new(data));
    }

    if data.starts_with(&GZIP_MAGIC) {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        return Ok(decompressed);
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Unknown compressed format",
    ))
}

/// Decodes a file payload into UTF-8 JSON string.
/// Supports plain JSON, Zstd, and legacy Gzip payloads.
pub fn decode_file_payload(file_data: &[u8]) -> Result<String, std::io::Error> {
    let decoded = if is_compressed(file_data) {
        decompress_data(file_data)?
    } else {
        file_data.to_vec()
    };

    String::from_utf8(decoded)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

/// Checks if data is compressed by looking for known magic numbers.
pub fn is_compressed(data: &[u8]) -> bool {
    data.starts_with(&GZIP_MAGIC) || data.starts_with(&ZSTD_MAGIC)
}

/// Calculates a checksum for a slice of bytes using BLAKE3.
///
/// # Arguments
/// * `data` - The byte slice to hash.
pub fn calculate_checksum(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Calculates checksum using legacy `DefaultHasher` for backward compatibility.
pub fn calculate_legacy_checksum(data: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Atomically writes file bytes to target path.
pub fn atomic_write_file(path: &Path, data: &[u8]) -> Result<(), std::io::Error> {
    let atomic_file = AtomicFile::new(path, OverwriteBehavior::AllowOverwrite);
    atomic_file
        .write(|file| file.write_all(data))
        .map_err(|e| io::Error::other(e.to_string()))
}
