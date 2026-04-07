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
//!   SKILL.md emitter, agent emitter, hooks JSON emitter, script body
//!   emitter, and pack manifest builder. Every runtime adapter reuses
//!   these instead of duplicating serialization logic.
//! * **Runtimes** ([`runtimes`]) — concrete [`CodingAgentRuntime`]
//!   implementations for Claude Code, Gemini CLI, Codex CLI, OpenCode,
//!   Amp, and generic npm packages.
//! * **Composition** ([`compose`]) — free functions that compose
//!   capability traits to render a whole [`PackBundle`] at once.
//! * **Install helpers** ([`install`]) — concrete filesystem writers
//!   on top of [`ExportedTree`] plus per-capability install shortcuts.
//! * **MCP support** ([`mcp`]) — [`mcp::McpServer`] model, the
//!   [`mcp::McpCapability`] peer trait, and format-specific renderers.

pub mod compose;
pub mod error;
pub mod install;
pub mod mcp;
pub mod model;
pub mod naming;
pub mod output;
pub mod render;
pub mod runtime;
pub mod runtimes;
pub mod tools;

pub use error::Error;
pub use mcp::{McpServer, McpTransport};
pub use model::{Agent, Hook, Pack, PackBundle, Role, Script, ScriptLanguage, Skill};
pub use output::{ExportedFile, ExportedFileType, ExportedTree};
pub use runtime::{
    AgentCapability, CodingAgentRuntime, FieldNaming, FrontmatterDialect, HookCapability, RuntimeId, RuntimePaths,
    Scope, ScopedRelative, ScriptCapability, SkillCapability,
};
pub use tools::{ToolEntry, ToolRegistry};

/// Convenience `Result` alias using the crate [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
