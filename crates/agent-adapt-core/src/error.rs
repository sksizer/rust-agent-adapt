//! Error type used across every `agent-adapt-*` crate.
//!
//! This is deliberately a single flat enum rather than a hierarchy. Capability
//! implementations surface domain-specific problems via [`Error::Render`] or
//! [`Error::Validation`]; filesystem helpers (in downstream crates) contribute
//! [`Error::Io`] and [`Error::Serialization`] via `#[from]`. Keeping everything
//! in one place lets callers match once at the boundary.

use std::io;

use thiserror::Error as ThisError;

/// All errors the core crate (and its capability traits) can produce.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Filesystem I/O failed. Primarily produced by the downstream install
    /// helpers, but kept in core so the [`crate::Result`] alias is usable
    /// from any layer without a second error type.
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    /// A JSON (de)serialize failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// A capability implementation could not render the requested entity —
    /// e.g. a required field was missing or the entity referenced an unknown
    /// tool. The message should be actionable.
    #[error("render error: {0}")]
    Render(String),

    /// Validation of a [`crate::Pack`] or one of its children found problems
    /// that prevent export.
    #[error("validation error: {0}")]
    Validation(String),

    /// A structural invariant was violated while reading or merging a config
    /// file — e.g. "root is not an object". The static string points at the
    /// specific invariant so tests can match on it.
    #[error("invalid format: {0}")]
    InvalidFormat(&'static str),

    /// A serialization step (non-JSON) failed. Kept open-ended because
    /// different downstream crates may use different serializers (TOML, YAML).
    #[error("serialization error: {0}")]
    Serialization(String),
}

impl Error {
    /// Convenience constructor for a render failure with a formatted message.
    pub fn render(msg: impl Into<String>) -> Self {
        Error::Render(msg.into())
    }

    /// Convenience constructor for a validation failure with a formatted message.
    pub fn validation(msg: impl Into<String>) -> Self {
        Error::Validation(msg.into())
    }
}
