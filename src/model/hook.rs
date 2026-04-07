use serde::{Deserialize, Serialize};

/// An automation hook: "when event X fires and the matcher Y is met, run
/// command Z."
///
/// The canonical form matches Claude Code's vocabulary (e.g. event
/// `"PreToolUse"`, matcher `"Edit|Write"`) because that's the richest
/// superset. Runtimes with narrower hook models ignore fields they can't
/// express.
///
/// # Field notes
///
/// * [`Hook::matcher`] is a pipe-delimited string of canonical tool
///   names. Render impls split on `|`, translate each half, and rejoin.
/// * [`Hook::timeout_ms`] of `0` means "runtime default / no timeout."
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hook {
    /// Human-readable name.
    pub name: String,
    /// Stable kebab-case identifier.
    pub slug: String,
    /// Optional description.
    pub description: Option<String>,
    /// Lifecycle event name (e.g. `"PreToolUse"`).
    pub event: String,
    /// Optional pipe-delimited matcher of canonical tool names.
    pub matcher: Option<String>,
    /// Shell command to execute.
    pub command: String,
    /// Timeout in milliseconds. `0` means no timeout.
    pub timeout_ms: i64,
    /// Free-form tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional category.
    pub category: Option<String>,
    /// If `true`, this hook is a template.
    pub is_template: bool,
}
