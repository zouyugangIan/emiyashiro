//! Shared utility functions for file operations, compression, and checksums.

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};

/// Compresses data using Gzip.
///
/// # Arguments
/// * `data` - The raw byte slice to compress.
/// * `level` - The compression level (0-9).
pub fn compress_data(data: &[u8], level: u32) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
    encoder.write_all(data)?;
    encoder.finish()
}

/// Decompresses data using Gzip.
///
/// # Arguments
/// * `data` - The compressed byte slice.
pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// Checks if data is Gzip compressed by looking for the magic number.
/// The Gzip magic number is `0x1f 0x8b`.
pub fn is_compressed(data: &[u8]) -> bool {
    data.starts_with(&[0x1f, 0x8b])
}

/// Calculates a checksum for a slice of bytes using DefaultHasher.
///
/// # Arguments
/// * `data` - The byte slice to hash.
pub fn calculate_checksum(data: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
