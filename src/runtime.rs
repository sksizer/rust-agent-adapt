//! Runtime trait hierarchy: [`CodingAgentRuntime`] as the base, and one
//! peer sub-trait per capability ([`SkillCapability`], [`AgentCapability`],
//! [`HookCapability`], [`ScriptCapability`]).
//!
//! MCP support is another peer capability in [`crate::mcp::McpCapability`]
//! with the same shape — deliberately not privileged in this module so
//! runtimes that don't care about MCP never implement it.
//!
//! # Why separate traits per capability
//!
//! Not every runtime supports every asset type — `npm_package` has no
//! hooks, some runtimes handle MCP differently. Per-capability traits let
//! a runtime express exactly what it supports, and let composition
//! functions bound themselves on only the capabilities they actually use.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Agent, ExportedFile, Hook, Result, Script, Skill};

/// Closed enum of every runtime the crate knows about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeId {
    /// Anthropic's Claude Code CLI.
    ClaudeCode,
    /// Google's Gemini CLI.
    GeminiCli,
    /// OpenAI's Codex CLI.
    CodexCli,
    /// SST's OpenCode.
    OpenCode,
    /// Sourcegraph's Amp.
    Amp,
    /// Generic npm package layout.
    NpmPackage,
}

/// Project scope vs user scope for path resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Paths relative to a project root.
    Project,
    /// Paths relative to the user's home directory.
    User,
}

/// Convention bundle describing where a runtime stores its asset types.
/// A path that differs between project scope and user scope.
///
/// Runtimes that use the same directory layout in both scopes (most of
/// them) store the same [`PathBuf`] in both fields. Runtimes that don't
/// (or that only support one scope) can still represent it with
/// distinct values.
#[derive(Debug, Clone, PartialEq)]
pub struct ScopedRelative {
    /// Path relative to a project root (e.g. `.claude/skills`).
    pub project: PathBuf,
    /// Path relative to a user's home directory (e.g. `.claude/skills`).
    pub user: PathBuf,
}

impl ScopedRelative {
    /// Construct a [`ScopedRelative`] where the same path is used for
    /// both scopes — the common case.
    pub fn same(path: impl Into<PathBuf>) -> Self {
        let p = path.into();
        Self { project: p.clone(), user: p }
    }

    /// Return the path that applies to the given scope.
    pub fn for_scope(&self, scope: Scope) -> &PathBuf {
        match scope {
            Scope::Project => &self.project,
            Scope::User => &self.user,
        }
    }
}

/// Per-asset path conventions for a runtime.
///
/// Each field is either a [`ScopedRelative`] (for asset types the
/// runtime supports) or `None` (for ones it doesn't — e.g. the
/// `NpmPackage` runtime has no hooks or MCP config).
///
/// Paths are **relative** — either to a project root (for
/// [`Scope::Project`]) or to a user home directory (for [`Scope::User`]).
/// Anchoring them against an absolute root is the install layer's job.
///
/// Note that asset types do not share a common prefix: Codex CLI, for
/// example, stores skills under `.agents/skills/` (the Anthropic Agent
/// Skills shared convention) but its MCP config under `.codex/config.toml`.
/// Representing every path explicitly removes that ambiguity.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimePaths {
    /// Directory where [`Skill`]s are installed.
    pub skills_dir: ScopedRelative,
    /// Directory where [`Agent`] personas are installed.
    pub agents_dir: ScopedRelative,
    /// Directory where [`Script`]s are installed.
    pub scripts_dir: ScopedRelative,
    /// Hooks configuration file, if this runtime has one.
    pub hooks_file: Option<ScopedRelative>,
    /// MCP servers configuration file, if this runtime has one.
    pub mcp_config_file: Option<ScopedRelative>,
}

impl RuntimePaths {
    /// Resolved skills directory, anchored against `anchor`.
    pub fn skills_dir_for(&self, anchor: &Path, scope: Scope) -> PathBuf {
        anchor.join(self.skills_dir.for_scope(scope))
    }

    /// Resolved agents directory, anchored against `anchor`.
    pub fn agents_dir_for(&self, anchor: &Path, scope: Scope) -> PathBuf {
        anchor.join(self.agents_dir.for_scope(scope))
    }

    /// Resolved scripts directory, anchored against `anchor`.
    pub fn scripts_dir_for(&self, anchor: &Path, scope: Scope) -> PathBuf {
        anchor.join(self.scripts_dir.for_scope(scope))
    }

    /// Resolved hooks file path, if this runtime has one.
    pub fn hooks_path_for(&self, anchor: &Path, scope: Scope) -> Option<PathBuf> {
        self.hooks_file.as_ref().map(|sr| anchor.join(sr.for_scope(scope)))
    }

    /// Resolved MCP config file path, if this runtime has one.
    pub fn mcp_config_path_for(&self, anchor: &Path, scope: Scope) -> Option<PathBuf> {
        self.mcp_config_file.as_ref().map(|sr| anchor.join(sr.for_scope(scope)))
    }
}

/// Casing convention for YAML frontmatter field names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldNaming {
    /// `allowed-tools`, `user-invocable`, etc.
    Kebab,
    /// `allowed_tools`, `user_invocable`, etc.
    Snake,
}

/// Per-runtime YAML frontmatter quirks.
#[derive(Debug, Clone, PartialEq)]
pub struct FrontmatterDialect {
    /// Whether field names are kebab-case or snake_case.
    pub field_naming: FieldNaming,
    /// Canonical (snake_case) field names that should be unconditionally
    /// omitted from the rendered frontmatter.
    pub omit_fields: &'static [&'static str],
    /// If `true`, emit `user-invocable: true` explicitly rather than
    /// relying on the spec default.
    pub emit_user_invocable_default: bool,
}

/// The base runtime trait.
///
/// `Send + Sync` because runtime impls are typically zero-sized unit
/// structs passed around freely.
pub trait CodingAgentRuntime: Send + Sync {
    /// Stable machine identifier for this runtime.
    fn id(&self) -> RuntimeId;

    /// Human-readable display name.
    fn display_name(&self) -> &'static str;

    /// Path conventions.
    fn paths(&self) -> &RuntimePaths;

    /// YAML frontmatter dialect.
    fn frontmatter_dialect(&self) -> &FrontmatterDialect;
}

/// Runtimes that can render [`Skill`]s.
pub trait SkillCapability: CodingAgentRuntime {
    /// Render a single skill to one or more virtual files.
    fn render_skill(&self, skill: &Skill) -> Result<Vec<ExportedFile>>;
}

/// Runtimes that can render agent personas.
pub trait AgentCapability: CodingAgentRuntime {
    /// Render a single agent persona to one or more virtual files.
    fn render_agent(&self, agent: &Agent) -> Result<Vec<ExportedFile>>;
}

/// Runtimes that can render hooks.
///
/// Hooks are rendered collectively because most runtimes serialize all
/// hooks into a single config file (e.g. `hooks.json`).
pub trait HookCapability: CodingAgentRuntime {
    /// Render a collection of hooks to one or more virtual files.
    fn render_hooks(&self, hooks: &[Hook]) -> Result<Vec<ExportedFile>>;
}

/// Runtimes that can render standalone scripts.
pub trait ScriptCapability: CodingAgentRuntime {
    /// Render a single script (shebang + body) to a virtual file.
    fn render_script(&self, script: &Script) -> Result<ExportedFile>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_paths() -> RuntimePaths {
        RuntimePaths {
            skills_dir: ScopedRelative::same(".claude/skills"),
            agents_dir: ScopedRelative::same(".claude/agents"),
            scripts_dir: ScopedRelative::same(".claude/scripts"),
            hooks_file: Some(ScopedRelative::same(".claude/hooks.json")),
            mcp_config_file: Some(ScopedRelative::same(".mcp.json")),
        }
    }

    #[test]
    fn project_scope_anchors_under_project_root() {
        let p = sample_paths();
        let anchor = Path::new("/tmp/proj");
        assert_eq!(p.skills_dir_for(anchor, Scope::Project), PathBuf::from("/tmp/proj/.claude/skills"));
        assert_eq!(p.hooks_path_for(anchor, Scope::Project), Some(PathBuf::from("/tmp/proj/.claude/hooks.json")));
        assert_eq!(p.mcp_config_path_for(anchor, Scope::Project), Some(PathBuf::from("/tmp/proj/.mcp.json")));
    }

    #[test]
    fn scoped_relative_distinct_project_and_user() {
        let sr = ScopedRelative { project: ".claude/skills".into(), user: ".config/claude/skills".into() };
        assert_eq!(sr.for_scope(Scope::Project), &PathBuf::from(".claude/skills"));
        assert_eq!(sr.for_scope(Scope::User), &PathBuf::from(".config/claude/skills"));
    }

    #[test]
    fn missing_optional_paths_return_none() {
        let p = RuntimePaths {
            skills_dir: ScopedRelative::same(".foo/skills"),
            agents_dir: ScopedRelative::same(".foo/agents"),
            scripts_dir: ScopedRelative::same(".foo/scripts"),
            hooks_file: None,
            mcp_config_file: None,
        };
        assert!(p.hooks_path_for(Path::new("/"), Scope::Project).is_none());
        assert!(p.mcp_config_path_for(Path::new("/"), Scope::Project).is_none());
    }
}
