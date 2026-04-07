//! Data model and runtime adapters for authoring coding-agent assets once
//! and exporting them to every major runtime.
//!
//! `agent-adapt` ships as a single crate with the following layers:
//!
//! * **Domain models** ([`Skill`], [`Agent`], [`Hook`], [`Script`],
//!   [`Role`], [`Pack`], [`PackBundle`]) — pure authoring content.
//! * **Naming** ([`naming`]) — kebab-case slugging, skill-name clamping,
//!   and deduplication helpers.
//! * **Virtual filesystem** ([`ExportedFile`], [`ExportedTree`]) — what
//!   capability rendering functions return. No I/O.
//! * **Runtime trait hierarchy** ([`CodingAgentRuntime`] plus peer
//!   capability traits [`SkillCapability`], [`AgentCapability`],
//!   [`HookCapability`], [`ScriptCapability`], [`mcp::McpCapability`]).
//! * **Tool vocabulary** ([`tools::ToolRegistry`]) — canonical tool names
//!   and per-runtime translation.
//! * **Shared rendering** ([`render`]) — YAML frontmatter builder,
//!   `SKILL.md` emitter, and pack manifest builder, reused across every
//!   runtime implementation.
//! * **MCP support** ([`mcp`]) — [`mcp::McpServer`] model, the
//!   [`mcp::McpCapability`] peer trait, and format-specific renderers.
//! * **Legacy MCP installer** ([`providers`]) — the original v0.1
//!   merge-into-existing-config installer. Still exported for backwards
//!   compatibility; new code should use [`mcp`] + an install helper.

pub mod error;
pub mod mcp;
pub mod model;
pub mod naming;
pub mod output;
pub mod providers;
pub mod render;
pub mod runtime;
pub mod tools;

pub use error::Error;
pub use model::{Agent, Hook, Pack, PackBundle, Role, Script, ScriptLanguage, Skill};
pub use output::{ExportedFile, ExportedFileType, ExportedTree};
pub use runtime::{
    AgentCapability, CodingAgentRuntime, FieldNaming, FrontmatterDialect, HookCapability, RuntimeId, RuntimePaths,
    Scope, ScriptCapability, SkillCapability,
};
pub use tools::{ToolEntry, ToolRegistry};

// Backwards-compat re-exports for v0.1 consumers:
pub use mcp::{McpServer, McpTransport};
pub use providers::{AgentConfigProvider, all_providers, install_to_all};

/// Convenience `Result` alias using the crate [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
