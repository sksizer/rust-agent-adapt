//! Sourcegraph Amp runtime adapter.
//!
//! Project-scope layout:
//!
//! ```text
//! <project>/.agents/            (shared Agent Skills convention)
//! ├── skills/<slug>/SKILL.md
//! ├── agents/<slug>.md
//! ├── scripts/<slug>.{sh,py,js}
//! ├── hooks.json
//! └── settings.json             (MCP config, Claude-Code-compatible shape)
//! ```
//!
//! Amp uses the shared `.agents/` directory convention for skills (like
//! Codex CLI) and keeps its own tool-name vocabulary where most names
//! match Claude Code but a few diverge (e.g. `create_file` for `Write`,
//! `read_web_page` for `WebFetch`, `get_diagnostics` for `LSP`).

use std::sync::OnceLock;

use crate::mcp::{McpCapability, McpServer, render::render_claude_json};
use crate::runtimes::standard;
use crate::{
    Agent, AgentCapability, CodingAgentRuntime, ExportedFile, FieldNaming, FrontmatterDialect, Hook, HookCapability,
    Result, RuntimeId, RuntimePaths, ScopedRelative, Script, ScriptCapability, Skill, SkillCapability, ToolRegistry,
};

/// Amp runtime adapter.
pub struct Amp;

fn registry() -> &'static ToolRegistry {
    static REGISTRY: OnceLock<ToolRegistry> = OnceLock::new();
    REGISTRY.get_or_init(ToolRegistry::default)
}

fn paths() -> &'static RuntimePaths {
    static PATHS: OnceLock<RuntimePaths> = OnceLock::new();
    PATHS.get_or_init(|| RuntimePaths {
        skills_dir: ScopedRelative::same(".agents/skills"),
        agents_dir: ScopedRelative::same(".agents/agents"),
        scripts_dir: ScopedRelative::same(".agents/scripts"),
        hooks_file: Some(ScopedRelative::same(".agents/hooks.json")),
        mcp_config_file: Some(ScopedRelative::same(".agents/settings.json")),
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

impl CodingAgentRuntime for Amp {
    fn id(&self) -> RuntimeId {
        RuntimeId::Amp
    }
    fn display_name(&self) -> &'static str {
        "Amp"
    }
    fn paths(&self) -> &RuntimePaths {
        paths()
    }
    fn frontmatter_dialect(&self) -> &FrontmatterDialect {
        dialect()
    }
}

impl SkillCapability for Amp {
    fn render_skill(&self, skill: &Skill) -> Result<Vec<ExportedFile>> {
        standard::render_skill(self, skill, registry())
    }
}

impl AgentCapability for Amp {
    fn render_agent(&self, agent: &Agent) -> Result<Vec<ExportedFile>> {
        standard::render_agent(self, agent, registry())
    }
}

impl HookCapability for Amp {
    fn render_hooks(&self, hooks: &[Hook]) -> Result<Vec<ExportedFile>> {
        standard::render_hooks(self, hooks, registry())
    }
}

impl ScriptCapability for Amp {
    fn render_script(&self, script: &Script) -> Result<ExportedFile> {
        standard::render_script(self, script)
    }
}

impl McpCapability for Amp {
    fn render_mcp_server(&self, server: &McpServer) -> Result<Vec<ExportedFile>> {
        let content = render_claude_json(server)?;
        Ok(standard::wrap_mcp_config(self, content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn skill_path_under_shared_agents_dir() {
        let s = Skill::new("My Skill", "d", "b");
        let files = Amp.render_skill(&s).unwrap();
        assert_eq!(files[0].path, PathBuf::from(".agents/skills/my-skill/SKILL.md"));
    }

    #[test]
    fn write_tool_is_create_file_for_amp() {
        let mut s = Skill::new("x", "d", "b");
        s.allowed_tools = vec!["Write".into()];
        let files = Amp.render_skill(&s).unwrap();
        assert!(files[0].text().unwrap().contains("- create_file"));
    }
}
