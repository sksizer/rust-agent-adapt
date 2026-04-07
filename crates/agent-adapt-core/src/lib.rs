//! Core domain models and traits for `agent-adapt`.
//!
//! This crate is the foundation shared by every other `agent-adapt-*` crate. It
//! defines the pure content shapes — [`Skill`], [`Agent`], [`Hook`], [`Script`],
//! [`Role`], [`Pack`], [`PackBundle`] — plus the [`ExportedFile`] virtual
//! filesystem, the [`CodingAgentRuntime`] trait and its peer capability traits
//! ([`SkillCapability`], [`AgentCapability`], [`HookCapability`],
//! [`ScriptCapability`]), and shared utilities ([`naming`], [`error::Error`]).
//!
//! The crate performs **no I/O**. Capability impls return virtual trees of
//! [`ExportedFile`]s; writing them to disk is the concern of a separate layer.

pub mod error;
pub mod model;
pub mod naming;
pub mod output;
pub mod runtime;

pub use error::Error;
pub use model::{Agent, Hook, Pack, PackBundle, Role, Script, ScriptLanguage, Skill};
pub use output::{ExportedFile, ExportedFileType, ExportedTree};
pub use runtime::{
    AgentCapability, CodingAgentRuntime, FrontmatterDialect, HookCapability, RuntimeId, RuntimePaths, Scope,
    ScriptCapability, SkillCapability,
};

/// Result type alias using the crate [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
