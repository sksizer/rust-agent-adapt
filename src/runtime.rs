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
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimePaths {
    /// The runtime's top-level directory within a project root, e.g. `.claude`.
    pub project_root_dir: PathBuf,
    /// The runtime's top-level directory within the user's home dir.
    pub user_root_dir: PathBuf,
    /// Subdirectory (relative to the scope root) where skills are installed.
    pub skills_subdir: PathBuf,
    /// Subdirectory where agent personas are installed.
    pub agents_subdir: PathBuf,
    /// Subdirectory where scripts are installed.
    pub scripts_subdir: PathBuf,
    /// Path of the hooks configuration file, if this runtime has one.
    pub hooks_file: Option<PathBuf>,
    /// Path of the MCP servers configuration file, if this runtime has one.
    pub mcp_config_file: Option<PathBuf>,
}

impl RuntimePaths {
    /// Resolve the runtime's root directory for the given scope.
    pub fn root_for(&self, anchor: &Path, scope: Scope) -> PathBuf {
        match scope {
            Scope::Project => anchor.join(&self.project_root_dir),
            Scope::User => anchor.join(&self.user_root_dir),
        }
    }

    /// Resolved skills directory.
    pub fn skills_dir(&self, anchor: &Path, scope: Scope) -> PathBuf {
        self.root_for(anchor, scope).join(&self.skills_subdir)
    }

    /// Resolved agents directory.
    pub fn agents_dir(&self, anchor: &Path, scope: Scope) -> PathBuf {
        self.root_for(anchor, scope).join(&self.agents_subdir)
    }

    /// Resolved scripts directory.
    pub fn scripts_dir(&self, anchor: &Path, scope: Scope) -> PathBuf {
        self.root_for(anchor, scope).join(&self.scripts_subdir)
    }

    /// Resolved hooks file path, if this runtime has one.
    pub fn hooks_path(&self, anchor: &Path, scope: Scope) -> Option<PathBuf> {
        self.hooks_file.as_ref().map(|f| self.root_for(anchor, scope).join(f))
    }

    /// Resolved MCP config file path, if this runtime has one.
    pub fn mcp_config_path(&self, anchor: &Path, scope: Scope) -> Option<PathBuf> {
        self.mcp_config_file.as_ref().map(|f| self.root_for(anchor, scope).join(f))
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
