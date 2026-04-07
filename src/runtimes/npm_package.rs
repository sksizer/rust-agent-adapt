//! Generic npm-package runtime adapter.
//!
//! This isn't a coding-agent runtime in the usual sense — it's a
//! packaging target for sharing packs via the npm registry or a flat
//! directory layout:
//!
//! ```text
//! <pack-root>/
//! ├── skills/<slug>/SKILL.md
//! ├── agents/<slug>.md
//! └── scripts/<slug>.{sh,py,js}
//! ```
//!
//! It supports skills, agents, and scripts, but intentionally **does
//! not** implement [`crate::HookCapability`] or
//! [`crate::mcp::McpCapability`] — hooks and MCP servers don't make
//! sense in a standalone package that will be consumed by an unknown
//! runtime. Pack composition functions that require those capabilities
//! will refuse to accept this runtime at compile time.

use std::sync::OnceLock;

use crate::runtimes::standard;
use crate::{
    Agent, AgentCapability, CodingAgentRuntime, ExportedFile, FieldNaming, FrontmatterDialect, Result, RuntimeId,
    RuntimePaths, ScopedRelative, Script, ScriptCapability, Skill, SkillCapability, ToolRegistry,
};

/// Generic npm-package runtime adapter.
pub struct NpmPackage;

fn registry() -> &'static ToolRegistry {
    static REGISTRY: OnceLock<ToolRegistry> = OnceLock::new();
    REGISTRY.get_or_init(ToolRegistry::default)
}

fn paths() -> &'static RuntimePaths {
    static PATHS: OnceLock<RuntimePaths> = OnceLock::new();
    PATHS.get_or_init(|| RuntimePaths {
        skills_dir: ScopedRelative::same("skills"),
        agents_dir: ScopedRelative::same("agents"),
        scripts_dir: ScopedRelative::same("scripts"),
        // No hooks file — npm packages don't ship hook configs.
        hooks_file: None,
        // No MCP config — npm packages don't register MCP servers with
        // a specific runtime.
        mcp_config_file: None,
    })
}

fn dialect() -> &'static FrontmatterDialect {
    static DIALECT: OnceLock<FrontmatterDialect> = OnceLock::new();
    DIALECT.get_or_init(|| FrontmatterDialect {
        field_naming: FieldNaming::Kebab,
        omit_fields: &[],
        emit_user_invocable_default: false,
    })
}

impl CodingAgentRuntime for NpmPackage {
    fn id(&self) -> RuntimeId {
        RuntimeId::NpmPackage
    }
    fn display_name(&self) -> &'static str {
        "npm package"
    }
    fn paths(&self) -> &RuntimePaths {
        paths()
    }
    fn frontmatter_dialect(&self) -> &FrontmatterDialect {
        dialect()
    }
}

impl SkillCapability for NpmPackage {
    fn render_skill(&self, skill: &Skill) -> Result<Vec<ExportedFile>> {
        standard::render_skill(self, skill, registry())
    }
}

impl AgentCapability for NpmPackage {
    fn render_agent(&self, agent: &Agent) -> Result<Vec<ExportedFile>> {
        standard::render_agent(self, agent, registry())
    }
}

impl ScriptCapability for NpmPackage {
    fn render_script(&self, script: &Script) -> Result<ExportedFile> {
        standard::render_script(self, script)
    }
}

// Intentionally no HookCapability or McpCapability impl — npm packages
// don't ship hook configs or MCP server registrations.

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn skill_path_is_flat_skills_dir() {
        let s = Skill::new("My Skill", "d", "b");
        let files = NpmPackage.render_skill(&s).unwrap();
        assert_eq!(files[0].path, PathBuf::from("skills/my-skill/SKILL.md"));
    }

    #[test]
    fn no_hooks_or_mcp_config_paths() {
        assert!(NpmPackage.paths().hooks_file.is_none());
        assert!(NpmPackage.paths().mcp_config_file.is_none());
    }
}
