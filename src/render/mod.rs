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
//!   honors the runtime's [`crate::FrontmatterDialect`] (kebab vs snake
//!   case, omitted fields). Pure string construction; no YAML crate.
//! * [`skill_md::render_skill_md`] — composes a [`crate::Skill`] into a
//!   complete `SKILL.md` body (frontmatter + markdown). Calls the
//!   frontmatter builder and applies tool-name translation via
//!   [`crate::ToolRegistry`].
//! * [`manifest::render_pack_manifest`] — builds a human-readable pack
//!   overview listing every skill, agent, hook, and script in a bundle.

pub mod frontmatter;
pub mod manifest;
pub mod skill_md;

pub use frontmatter::FrontmatterBuilder;
pub use manifest::render_pack_manifest;
pub use skill_md::render_skill_md;
