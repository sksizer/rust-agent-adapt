//! Google Gemini CLI runtime adapter.
//!
//! Project-scope layout:
//!
//! ```text
//! <project>/.gemini/
//! ├── skills/<slug>/SKILL.md
//! ├── agents/<slug>.md
//! ├── scripts/<slug>.{sh,py,js}
//! ├── hooks.json
//! └── settings.json        (MCP config)
//! ```
//!
//! Gemini CLI uses `httpUrl` instead of `url` for HTTP MCP transports;
//! tool-name vocabulary is snake_case (`read_file`, `run_shell_command`).

use std::sync::OnceLock;

use crate::mcp::{McpCapability, McpServer, render::render_gemini_json};
use crate::runtimes::standard;
use crate::{
    Agent, AgentCapability, CodingAgentRuntime, ExportedFile, FieldNaming, FrontmatterDialect, Hook, HookCapability,
    Result, RuntimeId, RuntimePaths, ScopedRelative, Script, ScriptCapability, Skill, SkillCapability, ToolRegistry,
};

/// Gemini CLI runtime adapter.
pub struct GeminiCli;

fn registry() -> &'static ToolRegistry {
    static REGISTRY: OnceLock<ToolRegistry> = OnceLock::new();
    REGISTRY.get_or_init(ToolRegistry::default)
}

fn paths() -> &'static RuntimePaths {
    static PATHS: OnceLock<RuntimePaths> = OnceLock::new();
    PATHS.get_or_init(|| RuntimePaths {
        skills_dir: ScopedRelative::same(".gemini/skills"),
        agents_dir: ScopedRelative::same(".gemini/agents"),
        scripts_dir: ScopedRelative::same(".gemini/scripts"),
        hooks_file: Some(ScopedRelative::same(".gemini/hooks.json")),
        mcp_config_file: Some(ScopedRelative::same(".gemini/settings.json")),
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

impl CodingAgentRuntime for GeminiCli {
    fn id(&self) -> RuntimeId {
        RuntimeId::GeminiCli
    }
    fn display_name(&self) -> &'static str {
        "Gemini CLI"
    }
    fn paths(&self) -> &RuntimePaths {
        paths()
    }
    fn frontmatter_dialect(&self) -> &FrontmatterDialect {
        dialect()
    }
}

impl SkillCapability for GeminiCli {
    fn render_skill(&self, skill: &Skill) -> Result<Vec<ExportedFile>> {
        standard::render_skill(self, skill, registry())
    }
}

impl AgentCapability for GeminiCli {
    fn render_agent(&self, agent: &Agent) -> Result<Vec<ExportedFile>> {
        standard::render_agent(self, agent, registry())
    }
}

impl HookCapability for GeminiCli {
    fn render_hooks(&self, hooks: &[Hook]) -> Result<Vec<ExportedFile>> {
        standard::render_hooks(self, hooks, registry())
    }
}

impl ScriptCapability for GeminiCli {
    fn render_script(&self, script: &Script) -> Result<ExportedFile> {
        standard::render_script(self, script)
    }
}

impl McpCapability for GeminiCli {
    fn render_mcp_server(&self, server: &McpServer) -> Result<Vec<ExportedFile>> {
        let content = render_gemini_json(server)?;
        Ok(standard::wrap_mcp_config(self, content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn skill_path_under_gemini_skills() {
        let s = Skill::new("My Skill", "d", "b");
        let files = GeminiCli.render_skill(&s).unwrap();
        assert_eq!(files[0].path, PathBuf::from(".gemini/skills/my-skill/SKILL.md"));
    }

    #[test]
    fn skill_tool_names_translated_to_gemini_vocabulary() {
        let mut s = Skill::new("x", "d", "b");
        s.allowed_tools = vec!["Read".into(), "Bash".into()];
        let files = GeminiCli.render_skill(&s).unwrap();
        let text = files[0].text().unwrap();
        assert!(text.contains("read_file"));
        assert!(text.contains("run_shell_command"));
    }

    #[test]
    fn mcp_config_path_is_gemini_settings_json() {
        let files = GeminiCli.render_mcp_server(&McpServer::http("x", "http://y")).unwrap();
        assert_eq!(files[0].path, PathBuf::from(".gemini/settings.json"));
        assert!(files[0].text().unwrap().contains("httpUrl"));
    }
}
