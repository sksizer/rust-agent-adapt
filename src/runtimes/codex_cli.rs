//! OpenAI Codex CLI runtime adapter.
//!
//! Project-scope layout:
//!
//! ```text
//! <project>/
//! ├── .agents/                   (shared Agent Skills convention)
//! │   ├── skills/<slug>/SKILL.md
//! │   ├── agents/<slug>.md
//! │   └── scripts/<slug>.{sh,py,js}
//! └── .codex/
//!     ├── hooks.json
//!     └── config.toml             (MCP config)
//! ```
//!
//! Codex CLI splits its asset layout: skills live under `.agents/`
//! following the shared Anthropic Agent Skills convention, while MCP
//! servers and hooks are in the Codex-specific `.codex/` directory.
//! The MCP config is TOML, not JSON.

use std::sync::OnceLock;

use crate::mcp::{McpCapability, McpServer, render::render_codex_toml};
use crate::runtimes::standard;
use crate::{
    Agent, AgentCapability, CodingAgentRuntime, ExportedFile, FieldNaming, FrontmatterDialect, Hook, HookCapability,
    Result, RuntimeId, RuntimePaths, ScopedRelative, Script, ScriptCapability, Skill, SkillCapability, ToolRegistry,
};

/// Codex CLI runtime adapter.
pub struct CodexCli;

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
        hooks_file: Some(ScopedRelative::same(".codex/hooks.json")),
        mcp_config_file: Some(ScopedRelative::same(".codex/config.toml")),
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

impl CodingAgentRuntime for CodexCli {
    fn id(&self) -> RuntimeId {
        RuntimeId::CodexCli
    }
    fn display_name(&self) -> &'static str {
        "Codex CLI"
    }
    fn paths(&self) -> &RuntimePaths {
        paths()
    }
    fn frontmatter_dialect(&self) -> &FrontmatterDialect {
        dialect()
    }
}

impl SkillCapability for CodexCli {
    fn render_skill(&self, skill: &Skill) -> Result<Vec<ExportedFile>> {
        standard::render_skill(self, skill, registry())
    }
}

impl AgentCapability for CodexCli {
    fn render_agent(&self, agent: &Agent) -> Result<Vec<ExportedFile>> {
        standard::render_agent(self, agent, registry())
    }
}

impl HookCapability for CodexCli {
    fn render_hooks(&self, hooks: &[Hook]) -> Result<Vec<ExportedFile>> {
        standard::render_hooks(self, hooks, registry())
    }
}

impl ScriptCapability for CodexCli {
    fn render_script(&self, script: &Script) -> Result<ExportedFile> {
        standard::render_script(self, script)
    }
}

impl McpCapability for CodexCli {
    fn render_mcp_server(&self, server: &McpServer) -> Result<Vec<ExportedFile>> {
        let content = render_codex_toml(server)?;
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
        let files = CodexCli.render_skill(&s).unwrap();
        assert_eq!(files[0].path, PathBuf::from(".agents/skills/my-skill/SKILL.md"));
    }

    #[test]
    fn mcp_config_path_is_codex_config_toml() {
        let files = CodexCli.render_mcp_server(&McpServer::http("x", "http://y")).unwrap();
        assert_eq!(files[0].path, PathBuf::from(".codex/config.toml"));
        assert!(files[0].text().unwrap().contains("mcp_servers"));
    }

    #[test]
    fn hooks_path_is_codex_hooks_json() {
        let hook = Hook {
            name: "x".into(),
            slug: "x".into(),
            description: None,
            event: "PreToolUse".into(),
            matcher: None,
            command: "echo".into(),
            timeout_ms: 0,
            tags: vec![],
            category: None,
            is_template: false,
        };
        let files = CodexCli.render_hooks(&[hook]).unwrap();
        assert_eq!(files[0].path, PathBuf::from(".codex/hooks.json"));
    }
}
