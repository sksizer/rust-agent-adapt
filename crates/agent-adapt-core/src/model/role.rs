use serde::{Deserialize, Serialize};

/// A role — a reusable chunk of system-prompt content that can be injected
/// into an [`crate::Agent`] or referenced from a [`crate::Skill`].
///
/// Roles exist so the same "you are an expert X" preamble isn't duplicated
/// across every agent and skill that needs it. The [`crate::PackBundle`]
/// carries the list of roles defined in a pack; runtimes that support role
/// injection (like Claude Code via the `agents/` directory) reference them
/// by id or slug at render time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Role {
    /// Human-readable name, e.g. `"Senior Rust Engineer"`.
    pub name: String,
    /// Stable kebab-case identifier used for linking.
    pub slug: String,
    /// Optional short description.
    pub description: Option<String>,
    /// The role's system-prompt body.
    pub body: String,
    /// Free-form tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
}
