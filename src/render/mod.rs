//! Shared rendering helpers for runtime implementations.
//!
//! Runtime adapters historically re-implemented the same YAML-frontmatter
//! / SKILL.md / manifest serialization logic inside each adapter file,
//! drifting out of sync over time. This module is the single source of
//! truth so runtime impls can collapse to ~100 lines of path + dialect
//! choices.
//!
//! # Layers
//!
//! * [`frontmatter::FrontmatterBuilder`] — low-level YAML emitter that
//!   honors the runtime's [`crate::FrontmatterDialect`].
//! * [`skill_md::render_skill_md`] — composes a [`crate::Skill`] into a
//!   complete `SKILL.md` body (frontmatter + markdown).
//! * [`agent_md::render_agent_md`] — composes an [`crate::Agent`] into
//!   a markdown file (frontmatter + system prompt body).
//! * [`hooks_json::render_hooks_json`] — emits a Claude-Code-compatible
//!   `hooks.json` file with per-runtime matcher translation.
//! * [`script_body::render_script_body`] — picks the shebang based on
//!   language and prepends it to the script body.
//! * [`manifest::render_pack_manifest`] — human-readable pack overview.

pub mod agent_md;
pub mod frontmatter;
pub mod hooks_json;
pub mod manifest;
pub mod script_body;
pub mod skill_md;

pub use agent_md::render_agent_md;
pub use frontmatter::FrontmatterBuilder;
pub use hooks_json::render_hooks_json;
pub use manifest::render_pack_manifest;
pub use script_body::{render_script_body, script_filename};
pub use skill_md::render_skill_md;
