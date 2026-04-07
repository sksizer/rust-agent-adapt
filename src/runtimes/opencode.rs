//! OpenCode (SST) runtime adapter.
//!
//! Project-scope layout:
//!
//! ```text
//! <project>/.opencode/
//! ├── skills/<slug>/SKILL.md
//! ├── agents/<slug>.md
//! ├── scripts/<slug>.{sh,py,js}
//! ├── hooks.json
//! └── config.json        (MCP config, Claude-Code-compatible shape)
//! ```
//!
//! OpenCode uses kebab-case field naming and lowercase tool names
//! (`read`, `edit`, `task`). Its MCP config format mirrors Claude Code's
//! JSON shape, so we render it with `render_claude_json`.

use std::sync::OnceLock;

use crate::mcp::{McpCapability, McpServer, render::render_claude_json};
use crate::runtimes::standard;
use crate::{
    Agent, AgentCapability, CodingAgentRuntime, ExportedFile, FieldNaming, FrontmatterDialect, Hook, HookCapability,
    Result, RuntimeId, RuntimePaths, ScopedRelative, Script, ScriptCapability, Skill, SkillCapability, ToolRegistry,
};

/// OpenCode runtime adapter.
pub struct OpenCode;

fn registry() -> &'static ToolRegistry {
    static REGISTRY: OnceLock<ToolRegistry> = OnceLock::new();
    REGISTRY.get_or_init(ToolRegistry::default)
}

fn paths() -> &'static RuntimePaths {
    static PATHS: OnceLock<RuntimePaths> = OnceLock::new();
    PATHS.get_or_init(|| RuntimePaths {
        skills_dir: ScopedRelative::same(".opencode/skills"),
        agents_dir: ScopedRelative::same(".opencode/agents"),
        scripts_dir: ScopedRelative::same(".opencode/scripts"),
        hooks_file: Some(ScopedRelative::same(".opencode/hooks.json")),
        mcp_config_file: Some(ScopedRelative::same(".opencode/config.json")),
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

impl CodingAgentRuntime for OpenCode {
    fn id(&self) -> RuntimeId {
        RuntimeId::OpenCode
    }
    fn display_name(&self) -> &'static str {
        "OpenCode"
    }
    fn paths(&self) -> &RuntimePaths {
        paths()
    }
    fn frontmatter_dialect(&self) -> &FrontmatterDialect {
        dialect()
    }
}

impl SkillCapability for OpenCode {
    fn render_skill(&self, skill: &Skill) -> Result<Vec<ExportedFile>> {
        standard::render_skill(self, skill, registry())
    }
}

impl AgentCapability for OpenCode {
    fn render_agent(&self, agent: &Agent) -> Result<Vec<ExportedFile>> {
        standard::render_agent(self, agent, registry())
    }
}

impl HookCapability for OpenCode {
    fn render_hooks(&self, hooks: &[Hook]) -> Result<Vec<ExportedFile>> {
        standard::render_hooks(self, hooks, registry())
    }
}

impl ScriptCapability for OpenCode {
    fn render_script(&self, script: &Script) -> Result<ExportedFile> {
        standard::render_script(self, script)
    }
}

impl McpCapability for OpenCode {
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
    fn skill_path_under_opencode_skills() {
        let s = Skill::new("My Skill", "d", "b");
        let files = OpenCode.render_skill(&s).unwrap();
        assert_eq!(files[0].path, PathBuf::from(".opencode/skills/my-skill/SKILL.md"));
    }

    #[test]
    fn tool_names_translated_to_opencode_vocabulary() {
        let mut s = Skill::new("x", "d", "b");
        s.allowed_tools = vec!["Read".into(), "Edit".into()];
        let files = OpenCode.render_skill(&s).unwrap();
        let text = files[0].text().unwrap();
        assert!(text.contains("- read"));
        assert!(text.contains("- edit"));
    }
}
