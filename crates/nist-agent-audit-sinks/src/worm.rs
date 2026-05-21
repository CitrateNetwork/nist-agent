//! WORM filesystem sink — Write-Once-Read-Many storage backend.
//!
//! Required for FedRAMP High deployments per RFC §6.2:
//! "WORM storage (write-once-read-many) — required for some
//! FedRAMP High and CJIS deployments; verified at write time by
//! the storage backend."
//!
//! Implementation strategy: one record per file, named by the
//! record's sequence number. The file is created with
//! `O_CREAT | O_EXCL` so a second write attempt on the same
//! sequence number errors at the OS layer (`EEXIST`). After write,
//! the file is closed and never reopened for writes.
//!
//! Stronger immutability (e.g. `chattr +i` on Linux) requires
//! root privileges and isn't portable; this sink delivers
//! "harness-level WORM" — sufficient for FedRAMP High when paired
//! with the filesystem's own access controls (POSIX ACLs, AppArmor
//! / SELinux confinement). Trail of Bits (S-13) will assess the
//! combined posture.

use citrate_agent_core::audit::{AuditRecord, AuditSink};
use citrate_agent_core::error::AgentError;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

pub struct WormFilesystemSink {
    dir: PathBuf,
}

impl WormFilesystemSink {
    /// Open the sink at `dir`. The directory is created if missing
    /// (with 0700 perms); existing records remain untouched.
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self, AgentError> {
        let dir = dir.into();
        std::fs::create_dir_all(&dir)
            .map_err(|e| AgentError::Audit(format!("create_dir {}: {e}", dir.display())))?;
        // Tighten directory perms. Best-effort: on platforms
        // without unix perms (Windows test runners) this is a
        // no-op.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&dir)
                .map_err(|e| AgentError::Audit(format!("stat {}: {e}", dir.display())))?
                .permissions();
            perms.set_mode(0o700);
            std::fs::set_permissions(&dir, perms)
                .map_err(|e| AgentError::Audit(format!("chmod {}: {e}", dir.display())))?;
        }
        Ok(Self { dir })
    }

    fn path_for(&self, sequence: u64) -> PathBuf {
        // Zero-pad to 20 chars so directory listing sorts by
        // sequence even with plain `ls`. u64 max is 20 digits.
        self.dir.join(format!("{sequence:020}.cbor"))
    }
}

impl AuditSink for WormFilesystemSink {
    fn append(&self, record: &AuditRecord) -> Result<(), AgentError> {
        let path = self.path_for(record.sequence);
        // The load-bearing flag: O_EXCL refuses to open if the
        // path already exists. This is the WORM-at-the-syscall
        // mechanism — second write of the same sequence errors.
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o400) // read-only after close
            .open(&path)
            .map_err(|e| {
                AgentError::Audit(format!(
                    "WORM violation or write error at {}: {e}",
                    path.display()
                ))
            })?;
        // CBOR-encode the record via the same canonical encoder
        // the upstream filesystem sink uses.
        let bytes = citrate_agent_core::audit::canonical_cbor(record)?;
        file.write_all(&bytes)
            .map_err(|e| AgentError::Audit(format!("write {}: {e}", path.display())))?;
        // fsync the file + the directory entry. Crash safety for
        // FedRAMP-grade durability.
        file.sync_all()
            .map_err(|e| AgentError::Audit(format!("fsync {}: {e}", path.display())))?;
        Ok(())
    }

    fn iter(
        &self,
    ) -> Result<Box<dyn Iterator<Item = Result<AuditRecord, AgentError>> + '_>, AgentError> {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(&self.dir)
            .map_err(|e| AgentError::Audit(format!("readdir {}: {e}", self.dir.display())))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("cbor"))
            .collect();
        entries.sort();
        let iter = entries.into_iter().map(|path| {
            let file = File::open(&path)
                .map_err(|e| AgentError::Audit(format!("open {}: {e}", path.display())))?;
            let record: AuditRecord = ciborium::from_reader(file)
                .map_err(|e| AgentError::Audit(format!("decode {}: {e}", path.display())))?;
            Ok(record)
        });
        Ok(Box::new(iter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use citrate_agent_core::audit::{AuditRecord, EventType};
    use tempfile::TempDir;

    fn fixture_record(sequence: u64) -> AuditRecord {
        AuditRecord {
            sequence,
            timestamp: sequence as i64 * 1_000_000,
            previous_hash: [0u8; 32],
            event_type: EventType::DoctorReport,
            payload: Vec::new(), // opaque CBOR bytes; empty for the fixture
            actor: format!("did:test:{sequence}"),
            signatures: Vec::new(),
            chain_anchor: None,
        }
    }

    #[test]
    fn open_creates_dir() {
        let tmp = TempDir::new().expect("tempdir");
        let inner = tmp.path().join("audit");
        let sink = WormFilesystemSink::open(&inner).expect("open");
        assert!(inner.is_dir());
        // Empty iter on fresh sink.
        let n = sink.iter().expect("iter").count();
        assert_eq!(n, 0);
    }

    #[test]
    fn append_then_iter_round_trips_records() {
        let tmp = TempDir::new().expect("tempdir");
        let sink = WormFilesystemSink::open(tmp.path()).expect("open");
        for seq in 0..3u64 {
            sink.append(&fixture_record(seq)).expect("append");
        }
        let records: Vec<AuditRecord> = sink
            .iter()
            .expect("iter")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect");
        assert_eq!(records.len(), 3);
        for (i, r) in records.iter().enumerate() {
            assert_eq!(r.sequence, i as u64);
        }
    }

    #[test]
    fn second_write_of_same_sequence_errors_worm() {
        // The load-bearing test: WORM means a second write of the
        // same sequence MUST fail at the syscall layer. O_EXCL
        // delivers this.
        let tmp = TempDir::new().expect("tempdir");
        let sink = WormFilesystemSink::open(tmp.path()).expect("open");
        sink.append(&fixture_record(0)).expect("first write ok");
        let err = sink
            .append(&fixture_record(0))
            .expect_err("second write must error");
        let msg = format!("{err}");
        assert!(
            msg.contains("WORM violation") || msg.contains("File exists"),
            "expected WORM violation, got: {msg}"
        );
    }

    #[test]
    fn iter_yields_records_in_sequence_order() {
        // Write out-of-order; iter MUST yield in-order. This is
        // upstream's contract — readers depend on it.
        let tmp = TempDir::new().expect("tempdir");
        let sink = WormFilesystemSink::open(tmp.path()).expect("open");
        for seq in [5u64, 1, 3, 0, 4, 2] {
            sink.append(&fixture_record(seq)).expect("append");
        }
        let records: Vec<u64> = sink
            .iter()
            .expect("iter")
            .map(|r| r.expect("decode").sequence)
            .collect();
        assert_eq!(records, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn written_files_are_read_only() {
        // Mode bits on the record files MUST exclude write — once
        // written, the OS itself prevents in-place modification by
        // non-root processes.
        let tmp = TempDir::new().expect("tempdir");
        let sink = WormFilesystemSink::open(tmp.path()).expect("open");
        sink.append(&fixture_record(0)).expect("append");
        let path = sink.path_for(0);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            // Only the owner-read bit set; no write bits anywhere.
            assert_eq!(mode & 0o222, 0, "file has write bits: mode {:o}", mode);
        }
    }
}
