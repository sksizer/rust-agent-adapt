//! Anthropic Claude Code runtime adapter.
//!
//! Project-scope layout:
//!
//! ```text
//! <project>/
//! ├── .mcp.json            (at project root, not under .claude/)
//! └── .claude/
//!     ├── skills/<slug>/SKILL.md
//!     ├── agents/<slug>.md
//!     ├── scripts/<slug>.{sh,py,js}
//!     └── hooks.json
//! ```
//!
//! Claude Code is the reference runtime — every other adapter inherits
//! its vocabulary and only overrides where it diverges.

use std::sync::OnceLock;

use crate::mcp::{McpCapability, McpServer, render::render_claude_json};
use crate::runtimes::standard;
use crate::{
    Agent, AgentCapability, CodingAgentRuntime, ExportedFile, FieldNaming, FrontmatterDialect, Hook, HookCapability,
    Result, RuntimeId, RuntimePaths, ScopedRelative, Script, ScriptCapability, Skill, SkillCapability, ToolRegistry,
};

/// Claude Code runtime adapter. Zero-sized unit struct.
pub struct ClaudeCode;

fn registry() -> &'static ToolRegistry {
    static REGISTRY: OnceLock<ToolRegistry> = OnceLock::new();
    REGISTRY.get_or_init(ToolRegistry::default)
}

fn paths() -> &'static RuntimePaths {
    static PATHS: OnceLock<RuntimePaths> = OnceLock::new();
    PATHS.get_or_init(|| RuntimePaths {
        skills_dir: ScopedRelative::same(".claude/skills"),
        agents_dir: ScopedRelative::same(".claude/agents"),
        scripts_dir: ScopedRelative::same(".claude/scripts"),
        hooks_file: Some(ScopedRelative::same(".claude/hooks.json")),
        // Claude Code's MCP config lives at the project root, not under .claude/.
        mcp_config_file: Some(ScopedRelative::same(".mcp.json")),
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

impl CodingAgentRuntime for ClaudeCode {
    fn id(&self) -> RuntimeId {
        RuntimeId::ClaudeCode
    }
    fn display_name(&self) -> &'static str {
        "Claude Code"
    }
    fn paths(&self) -> &RuntimePaths {
        paths()
    }
    fn frontmatter_dialect(&self) -> &FrontmatterDialect {
        dialect()
    }
}

impl SkillCapability for ClaudeCode {
    fn render_skill(&self, skill: &Skill) -> Result<Vec<ExportedFile>> {
        standard::render_skill(self, skill, registry())
    }
}

impl AgentCapability for ClaudeCode {
    fn render_agent(&self, agent: &Agent) -> Result<Vec<ExportedFile>> {
        standard::render_agent(self, agent, registry())
    }
}

impl HookCapability for ClaudeCode {
    fn render_hooks(&self, hooks: &[Hook]) -> Result<Vec<ExportedFile>> {
        standard::render_hooks(self, hooks, registry())
    }
}

impl ScriptCapability for ClaudeCode {
    fn render_script(&self, script: &Script) -> Result<ExportedFile> {
        standard::render_script(self, script)
    }
}

impl McpCapability for ClaudeCode {
    fn render_mcp_server(&self, server: &McpServer) -> Result<Vec<ExportedFile>> {
        let content = render_claude_json(server)?;
        Ok(standard::wrap_mcp_config(self, content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExportedFileType;
    use std::path::PathBuf;

    fn sample_skill() -> Skill {
        let mut s = Skill::new("My Skill", "A test skill", "Do the thing.");
        s.allowed_tools = vec!["Read".into(), "Write".into()];
        s
    }

    #[test]
    fn id_is_claude_code() {
        assert_eq!(ClaudeCode.id(), RuntimeId::ClaudeCode);
    }

    #[test]
    fn skill_path_is_nested_under_claude_skills() {
        let files = ClaudeCode.render_skill(&sample_skill()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, PathBuf::from(".claude/skills/my-skill/SKILL.md"));
        assert_eq!(files[0].kind, ExportedFileType::Skill);
    }

    #[test]
    fn skill_content_is_kebab_frontmatter() {
        let files = ClaudeCode.render_skill(&sample_skill()).unwrap();
        let text = files[0].text().unwrap();
        assert!(text.contains("allowed-tools:"));
        assert!(text.contains("  - Read"));
        assert!(text.contains("  - Write"));
    }

    #[test]
    fn agent_path_uses_slug() {
        let agent = Agent::new("Reviewer", "reviewer");
        let files = ClaudeCode.render_agent(&agent).unwrap();
        assert_eq!(files[0].path, PathBuf::from(".claude/agents/reviewer.md"));
    }

    #[test]
    fn mcp_config_path_is_project_root_mcp_json() {
        let server = McpServer::http("x", "http://y");
        let files = ClaudeCode.render_mcp_server(&server).unwrap();
        assert_eq!(files[0].path, PathBuf::from(".mcp.json"));
    }
}
