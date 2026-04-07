//! The runtime trait hierarchy: [`CodingAgentRuntime`] as the base, and
//! one peer sub-trait per capability ([`SkillCapability`],
//! [`AgentCapability`], [`HookCapability`], [`ScriptCapability`]).
//!
//! MCP support lives in a sibling crate (`agent-adapt-mcp`) as another peer
//! capability with the same shape — it is deliberately *not* privileged in
//! core, so runtimes that don't care about MCP never pay for it and runtimes
//! that do implement `McpCapability` alongside the skill/agent/hook ones.
//!
//! # Why separate traits per capability
//!
//! Not every runtime supports every asset type — `npm_package` doesn't
//! have hooks, `amp` doesn't install MCP servers the same way. Putting
//! each capability in its own trait lets a runtime express exactly what it
//! supports, and lets composition functions (e.g. a future `render_pack`)
//! bound themselves on only the capabilities they actually use.
//!
//! # Pure rendering only
//!
//! Every method in this module returns an [`crate::ExportedFile`]
//! (or a vec of them) — none touch the real filesystem. Concrete install
//! helpers that write trees to disk live in a downstream crate and are
//! built on top of these traits.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Agent, ExportedFile, Hook, Result, Script, Skill};

/// Closed enum of every runtime the workspace knows about.
///
/// Adding a new runtime means adding a variant here and implementing the
/// capability traits in `agent-adapt-runtimes`. User-defined runtimes were
/// considered and rejected for v0.2 — agentpants' `CustomCodingAgent`
/// table turned out to be a launch-shim (binary path, launch command),
/// not a custom export format, so it never needed to round-trip through
/// this enum.
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
    /// Generic npm package layout — not a runtime per se, but a valid
    /// export target for sharing packs via the npm registry.
    NpmPackage,
}

/// Project scope vs user scope for path resolution.
///
/// `Project` anchors paths under the project root (e.g. `<project>/.claude/`).
/// `User` anchors them under the user's home directory (e.g. `~/.claude/`).
/// Runtimes that only support one scope return the same path for both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Paths relative to a project root.
    Project,
    /// Paths relative to the user's home directory.
    User,
}

/// Convention bundle describing where a runtime stores its various asset
/// types. Kept as a plain struct rather than a set of trait methods so
/// custom/future runtimes can be constructed from data without implementing
/// a new trait.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimePaths {
    /// The runtime's top-level directory within a project root, e.g. `.claude`.
    pub project_root_dir: PathBuf,
    /// The runtime's top-level directory within the user's home dir, e.g. `.claude`.
    pub user_root_dir: PathBuf,
    /// Subdirectory (relative to the scope root) where skills are installed.
    pub skills_subdir: PathBuf,
    /// Subdirectory where agent personas are installed.
    pub agents_subdir: PathBuf,
    /// Subdirectory where scripts are installed.
    pub scripts_subdir: PathBuf,
    /// Path (relative to the scope root) of the hooks configuration file,
    /// if the runtime has one.
    pub hooks_file: Option<PathBuf>,
    /// Path of the MCP servers configuration file, if the runtime has one.
    /// Used by `agent-adapt-mcp`; kept here so the MCP crate can read it
    /// without re-duplicating path knowledge.
    pub mcp_config_file: Option<PathBuf>,
}

impl RuntimePaths {
    /// Resolve the runtime's root directory for the given scope, anchored
    /// against the caller-supplied project or home path.
    ///
    /// Callers are responsible for picking the right anchor — core does not
    /// touch `std::env::home_dir()` or any OS APIs.
    pub fn root_for(&self, anchor: &Path, scope: Scope) -> PathBuf {
        match scope {
            Scope::Project => anchor.join(&self.project_root_dir),
            Scope::User => anchor.join(&self.user_root_dir),
        }
    }

    /// Resolved skills directory for the given scope.
    pub fn skills_dir(&self, anchor: &Path, scope: Scope) -> PathBuf {
        self.root_for(anchor, scope).join(&self.skills_subdir)
    }

    /// Resolved agents directory for the given scope.
    pub fn agents_dir(&self, anchor: &Path, scope: Scope) -> PathBuf {
        self.root_for(anchor, scope).join(&self.agents_subdir)
    }

    /// Resolved scripts directory for the given scope.
    pub fn scripts_dir(&self, anchor: &Path, scope: Scope) -> PathBuf {
        self.root_for(anchor, scope).join(&self.scripts_subdir)
    }

    /// Resolved hooks file path, or `None` if this runtime has no hooks file.
    pub fn hooks_path(&self, anchor: &Path, scope: Scope) -> Option<PathBuf> {
        self.hooks_file.as_ref().map(|f| self.root_for(anchor, scope).join(f))
    }

    /// Resolved MCP config file path, or `None` if this runtime has no
    /// MCP config file.
    pub fn mcp_config_path(&self, anchor: &Path, scope: Scope) -> Option<PathBuf> {
        self.mcp_config_file.as_ref().map(|f| self.root_for(anchor, scope).join(f))
    }
}

/// Casing convention for YAML frontmatter field names.
///
/// Claude Code's Agent Skills spec uses kebab-case (`allowed-tools`,
/// `user-invocable`, `disable-model-invocation`); some earlier runtime
/// integrations used snake_case by mistake. This enum exists so the
/// shared frontmatter builder in `agent-adapt-render` can emit either
/// without duplicating the code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldNaming {
    /// `allowed-tools`, `user-invocable`, etc.
    Kebab,
    /// `allowed_tools`, `user_invocable`, etc.
    Snake,
}

/// Per-runtime YAML frontmatter quirks.
///
/// Lives in core rather than `agent-adapt-render` because runtime impls
/// need to hold one as a `&'static` constant and return it from
/// [`CodingAgentRuntime::frontmatter_dialect`]. The render crate consumes
/// these values; it doesn't define them.
#[derive(Debug, Clone, PartialEq)]
pub struct FrontmatterDialect {
    /// Whether field names are kebab-case or snake_case.
    pub field_naming: FieldNaming,
    /// Field names that should be unconditionally omitted from the rendered
    /// frontmatter even if the corresponding struct field is set. Used for
    /// runtimes whose spec only recognises a subset of the canonical fields.
    pub omit_fields: &'static [&'static str],
    /// If `true`, the renderer emits `user-invocable: true` explicitly
    /// instead of relying on the spec default.
    pub emit_user_invocable_default: bool,
}

/// The base runtime trait. Provides runtime identity and convention lookup;
/// individual capability traits extend it.
///
/// `Send + Sync` because runtime impls are typically zero-sized unit structs
/// that callers pass around freely.
pub trait CodingAgentRuntime: Send + Sync {
    /// Stable machine identifier for this runtime.
    fn id(&self) -> RuntimeId;

    /// Human-readable display name (e.g. `"Claude Code"`).
    fn display_name(&self) -> &'static str;

    /// Path conventions — where skills, agents, hooks, etc. live relative
    /// to a project root or user home.
    fn paths(&self) -> &RuntimePaths;

    /// YAML frontmatter dialect for this runtime. Consumed by the shared
    /// frontmatter builder in `agent-adapt-render`.
    fn frontmatter_dialect(&self) -> &FrontmatterDialect;
}

/// Runtimes that can render [`Skill`]s into an on-disk layout implement
/// this trait. The returned files are relative — they have not been
/// anchored against any project root.
pub trait SkillCapability: CodingAgentRuntime {
    /// Render a single skill to one or more virtual files.
    ///
    /// Most runtimes emit exactly one file (`SKILL.md`), but the return
    /// type is a `Vec` to accommodate runtimes that split the body and
    /// frontmatter, or that ship additional resources alongside.
    fn render_skill(&self, skill: &Skill) -> Result<Vec<ExportedFile>>;
}

/// Runtimes that can render agent personas implement this trait.
pub trait AgentCapability: CodingAgentRuntime {
    /// Render a single agent persona to one or more virtual files.
    fn render_agent(&self, agent: &Agent) -> Result<Vec<ExportedFile>>;
}

/// Runtimes that can render hooks implement this trait.
///
/// Unlike [`SkillCapability`] and [`AgentCapability`], hooks are rendered
/// collectively because most runtimes serialize all hooks into a single
/// configuration file (e.g. `hooks.json`).
pub trait HookCapability: CodingAgentRuntime {
    /// Render a collection of hooks to one or more virtual files.
    fn render_hooks(&self, hooks: &[Hook]) -> Result<Vec<ExportedFile>>;
}

/// Runtimes that can render standalone scripts implement this trait.
pub trait ScriptCapability: CodingAgentRuntime {
    /// Render a single script to a virtual file (shebang + body).
    fn render_script(&self, script: &Script) -> Result<ExportedFile>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_paths_project_scope_resolves_under_anchor() {
        let paths = RuntimePaths {
            project_root_dir: ".claude".into(),
            user_root_dir: ".claude".into(),
            skills_subdir: "skills".into(),
            agents_subdir: "agents".into(),
            scripts_subdir: "scripts".into(),
            hooks_file: Some("hooks.json".into()),
            mcp_config_file: Some(".mcp.json".into()),
        };
        let anchor = Path::new("/tmp/proj");
        assert_eq!(paths.skills_dir(anchor, Scope::Project), PathBuf::from("/tmp/proj/.claude/skills"));
        assert_eq!(paths.hooks_path(anchor, Scope::Project), Some(PathBuf::from("/tmp/proj/.claude/hooks.json")));
    }

    #[test]
    fn runtime_paths_no_hooks_file_returns_none() {
        let paths = RuntimePaths {
            project_root_dir: ".foo".into(),
            user_root_dir: ".foo".into(),
            skills_subdir: "skills".into(),
            agents_subdir: "agents".into(),
            scripts_subdir: "scripts".into(),
            hooks_file: None,
            mcp_config_file: None,
        };
        assert!(paths.hooks_path(Path::new("/"), Scope::Project).is_none());
        assert!(paths.mcp_config_path(Path::new("/"), Scope::Project).is_none());
    }
}
