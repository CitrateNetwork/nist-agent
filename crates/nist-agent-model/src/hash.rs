//! NIST SI-7 — System and Information Integrity.
//!
//! RFC §8.2 names the integrity verification: the harness MUST
//! recompute the SHA-256 of every model file at load time and
//! refuse to proceed on mismatch. This module is the single
//! verification surface; both the embedded resolver and the doctor
//! pre-flight check (S-7 check #7) bind here.

use crate::ModelError;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Compute SHA-256 of the file at `path` and compare against the
/// manifest's declared hex-encoded digest. Returns `Ok(())` on
/// match; returns `ModelError::HashMismatch` on divergence; returns
/// `ModelError::Io` on read failure.
///
/// Reading is buffered with a 64 KB chunk to bound memory for large
/// GGUF files (Gemma 4 E2B Q4_K_M is ~1.5 GB).
pub fn verify_sha256(path: &Path, expected_hex: &str) -> Result<(), ModelError> {
    let file =
        File::open(path).map_err(|e| ModelError::Io(format!("open {}: {e}", path.display())))?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| ModelError::Io(format!("read {}: {e}", path.display())))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let actual = hex::encode(hasher.finalize());
    let expected = expected_hex.trim().trim_start_matches("0x").to_lowercase();
    if actual != expected {
        return Err(ModelError::HashMismatch { expected, actual });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn matches_known_sha256_of_empty_file() {
        // sha256("") = e3b0c442... — the canonical empty hash.
        let f = NamedTempFile::new().expect("tempfile");
        verify_sha256(
            f.path(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        )
        .expect("empty file matches known empty sha256");
    }

    #[test]
    fn matches_known_sha256_of_hello_world() {
        // sha256("hello world\n") = a948904f...
        let mut f = NamedTempFile::new().expect("tempfile");
        f.write_all(b"hello world\n").expect("write");
        verify_sha256(
            f.path(),
            "a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447",
        )
        .expect("hello-world fixture matches known sha256");
    }

    #[test]
    fn mismatch_returns_typed_error() {
        let f = NamedTempFile::new().expect("tempfile");
        let err = verify_sha256(
            f.path(),
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        )
        .expect_err("mismatch must error");
        match err {
            ModelError::HashMismatch { expected, actual } => {
                assert_eq!(
                    expected,
                    "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                );
                assert_eq!(
                    actual,
                    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                );
            }
            other => panic!("expected HashMismatch, got {other:?}"),
        }
    }

    #[test]
    fn accepts_0x_prefix_and_uppercase() {
        let f = NamedTempFile::new().expect("tempfile");
        verify_sha256(
            f.path(),
            "0xE3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
        )
        .expect("uppercase + 0x prefix should normalize");
    }
}
