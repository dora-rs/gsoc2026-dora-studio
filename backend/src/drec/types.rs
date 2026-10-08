//! Binary `.drec` recording format types.
//!
//! Copied from `/home/dora/dora/libraries/recording/src/lib.rs`
//! Pinned to the dora revision documented in plans2.0/RUST-COMPAT.md.
//!
//! We copy rather than link because dora-rs requires edition 2024 + rustc 1.88,
//! and our environment is edition 2021 + rustc 1.75.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Magic bytes at the start of every .drec file.
pub const MAGIC: &[u8; 8] = b"DORAREC\x00";

/// Magic bytes marking the start of the footer.
pub const FOOTER_MAGIC: &[u8; 8] = b"DORAEND\x00";

/// Newest `.drec` container version accepted by Studio.
///
/// dora 1.0.1 writes version 2. Its container framing is unchanged from
/// version 1; only `event_bytes` changed from bincode to postcard. Studio
/// indexes those bytes as opaque payloads, so it can safely open both.
pub const FORMAT_VERSION: u16 = 2;

/// Maximum size of a single record or YAML descriptor (OOM guard).
pub const MAX_RECORD_BYTES: u32 = 64 * 1024 * 1024; // 64 MiB

/// Header written once at the beginning of a recording.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingHeader {
    pub version: u16,
    /// Wall-clock timestamp (nanoseconds) when recording started.
    pub start_nanos: u64,
    /// UUID of the recorded dataflow.
    pub dataflow_id: Uuid,
    /// The dataflow descriptor YAML at record time.
    pub descriptor_yaml: Vec<u8>,
}

/// A single captured message in the recording stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordEntry {
    /// Node that produced this message.
    pub node_id: String,
    /// Output port / topic name.
    pub output_id: String,
    /// Nanoseconds relative to [`RecordingHeader::start_nanos`].
    pub timestamp_offset_nanos: u64,
    /// Raw message bytes (typically an Arrow RecordBatch).
    pub event_bytes: Vec<u8>,
}

/// Optional footer at the end of a cleanly-closed recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingFooter {
    pub total_messages: u64,
    pub total_bytes: u64,
}
