//! The single source of truth: every content-facing scripting method.
//!
//! Entries are added by Phase C (reads) and Phase D (writes). KEEP THIS GROUPED BY
//! RECEIVER and in the same order as the source modules so it reads as a manifest.

use super::MethodDescriptor;

pub static REGISTRY: &[MethodDescriptor] = &[
    // Populated in Phase C (reads) and Phase D (writes).
];
