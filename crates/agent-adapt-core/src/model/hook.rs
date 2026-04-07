use serde::{Deserialize, Serialize};

/// An automation hook: "when event X fires and the tool/pattern matches Y,
/// run command Z."
///
/// Hooks map onto Claude Code's `hooks.json` format most directly; other
/// runtimes may transform the matcher or event names when rendering. The
/// canonical form stored here matches Claude Code's vocabulary (e.g. event
/// `"PreToolUse"`, matcher `"Edit|Write"`) because that's the richest
/// superset — runtimes with narrower hook models ignore fields they can't
/// express.
///
/// # Field notes
///
/// * [`Hook::event`] is a free-form string rather than an enum so new event
///   types can be added without a crate release. Runtimes validate.
/// * [`Hook::matcher`] is a pipe-delimited string of canonical tool names
///   (e.g. `"Edit|Write|MultiEdit"`). Render impls split on `|`, translate
///   each half, and rejoin.
/// * [`Hook::timeout_ms`] of `0` means "no timeout / runtime default."
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hook {
    /// Human-readable name.
    pub name: String,
    /// Stable kebab-case identifier.
    pub slug: String,
    /// Optional description.
    pub description: Option<String>,
    /// Lifecycle event name (e.g. `"PreToolUse"`, `"PostToolUse"`).
    pub event: String,
    /// Optional pipe-delimited matcher of canonical tool names.
    pub matcher: Option<String>,
    /// The shell command to execute when the hook fires.
    pub command: String,
    /// Timeout in milliseconds. `0` means "no timeout / runtime default."
    pub timeout_ms: i64,
    /// Free-form tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional category for grouping.
    pub category: Option<String>,
    /// If `true`, this hook is a template to be cloned.
    pub is_template: bool,
}
