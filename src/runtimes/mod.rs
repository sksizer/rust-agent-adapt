//! Concrete [`crate::CodingAgentRuntime`] implementations for every
//! supported runtime.
//!
//! Each submodule is a thin adapter: it declares the runtime's
//! [`crate::RuntimePaths`] and [`crate::FrontmatterDialect`] as
//! `&'static` values, then implements the peer capability traits by
//! delegating to the shared helpers in [`crate::render`] and
//! [`crate::mcp::render`].
//!
//! # What each adapter contributes
//!
//! * **Paths** — where skills, agents, hooks, scripts, and MCP config
//!   live in the runtime's on-disk layout (e.g. `.claude/skills/` for
//!   Claude Code, `.gemini/extensions/` for Gemini CLI).
//! * **Frontmatter dialect** — whether field names are kebab or snake
//!   case, which fields to omit, etc.
//! * **Capability opt-in** — runtimes that don't support a given asset
//!   type (e.g. `npm_package` has no hooks) simply don't implement the
//!   matching trait, and composition functions skip that asset type for
//!   them via trait bounds.
//! * **MCP format** — JSON vs TOML, `url` vs `httpUrl`, etc. Delegates
//!   to [`crate::mcp::render`].
//!
//! Call [`all`] to get a heterogeneous list of every built-in runtime
//! (as `Box<dyn CodingAgentRuntime>`), or [`for_id`] to look one up by
//! [`crate::RuntimeId`].

pub mod amp;
pub mod claude_code;
pub mod codex_cli;
pub mod gemini_cli;
pub mod npm_package;
pub mod opencode;
pub mod standard;

pub use amp::Amp;
pub use claude_code::ClaudeCode;
pub use codex_cli::CodexCli;
pub use gemini_cli::GeminiCli;
pub use npm_package::NpmPackage;
pub use opencode::OpenCode;

use crate::{CodingAgentRuntime, RuntimeId};

/// Return every built-in runtime as a boxed trait object, in a stable
/// order (matching [`RuntimeId`] variant declaration order).
///
/// Useful for "render to every runtime" style operations, or for
/// discovery UIs.
pub fn all() -> Vec<Box<dyn CodingAgentRuntime>> {
    vec![
        Box::new(ClaudeCode),
        Box::new(GeminiCli),
        Box::new(CodexCli),
        Box::new(OpenCode),
        Box::new(Amp),
        Box::new(NpmPackage),
    ]
}

/// Look up a runtime by [`RuntimeId`]. Returns `None` only if a new
/// variant is added to the enum and this function isn't updated in the
/// same commit (enforced by the exhaustive match).
pub fn for_id(id: RuntimeId) -> Option<Box<dyn CodingAgentRuntime>> {
    Some(match id {
        RuntimeId::ClaudeCode => Box::new(ClaudeCode),
        RuntimeId::GeminiCli => Box::new(GeminiCli),
        RuntimeId::CodexCli => Box::new(CodexCli),
        RuntimeId::OpenCode => Box::new(OpenCode),
        RuntimeId::Amp => Box::new(Amp),
        RuntimeId::NpmPackage => Box::new(NpmPackage),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_six_runtimes() {
        assert_eq!(all().len(), 6);
    }

    #[test]
    fn for_id_returns_matching_runtime() {
        assert_eq!(for_id(RuntimeId::ClaudeCode).unwrap().id(), RuntimeId::ClaudeCode);
        assert_eq!(for_id(RuntimeId::GeminiCli).unwrap().id(), RuntimeId::GeminiCli);
        assert_eq!(for_id(RuntimeId::CodexCli).unwrap().id(), RuntimeId::CodexCli);
        assert_eq!(for_id(RuntimeId::OpenCode).unwrap().id(), RuntimeId::OpenCode);
        assert_eq!(for_id(RuntimeId::Amp).unwrap().id(), RuntimeId::Amp);
        assert_eq!(for_id(RuntimeId::NpmPackage).unwrap().id(), RuntimeId::NpmPackage);
    }
}
