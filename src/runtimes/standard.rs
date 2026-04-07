//! Standard capability implementations shared by the built-in runtimes.
//!
//! Every Claude-Code-style runtime (that's five of the six we ship) has
//! the same rendering logic — the only per-runtime variations are:
//!
//! * where files land (captured in [`crate::RuntimePaths`]),
//! * the frontmatter dialect (captured in [`crate::FrontmatterDialect`]),
//! * the MCP config format (handled per-runtime by calling a specific
//!   [`crate::mcp::render`] function).
//!
//! These helpers use a runtime's `paths()` and `frontmatter_dialect()`
//! to do the common work. Runtime impls call them in one line each,
//! keeping each adapter file focused on declarations.

use std::path::PathBuf;

use crate::render::{render_agent_md, render_hooks_json, render_script_body, render_skill_md, script_filename};
use crate::{
    Agent, CodingAgentRuntime, ExportedFile, ExportedFileType, Hook, Result, Scope, Script, Skill, ToolRegistry,
    naming::slugify_skill_name,
};

/// Render a skill as a single `SKILL.md` file at
/// `<skills_dir>/<slug>/SKILL.md`.
///
/// The path is taken from the runtime's
/// [`RuntimePaths::skills_dir`](crate::RuntimePaths::skills_dir) for the
/// project scope — that's the canonical install location.
pub fn render_skill<R: CodingAgentRuntime + ?Sized>(
    runtime: &R,
    skill: &Skill,
    registry: &ToolRegistry,
) -> Result<Vec<ExportedFile>> {
    let slug = slugify_skill_name(&skill.name);
    let content = render_skill_md(skill, runtime.id(), runtime.frontmatter_dialect(), registry);
    let skills_dir = runtime.paths().skills_dir.for_scope(Scope::Project);
    let path: PathBuf = skills_dir.join(&slug).join("SKILL.md");
    Ok(vec![ExportedFile { path, content: content.into_bytes(), kind: ExportedFileType::Skill }])
}

/// Render an agent as a single `<slug>.md` file under the runtime's
/// `agents_dir`.
pub fn render_agent<R: CodingAgentRuntime + ?Sized>(
    runtime: &R,
    agent: &Agent,
    registry: &ToolRegistry,
) -> Result<Vec<ExportedFile>> {
    let content = render_agent_md(agent, runtime.id(), runtime.frontmatter_dialect(), registry);
    let agents_dir = runtime.paths().agents_dir.for_scope(Scope::Project);
    let path = agents_dir.join(format!("{}.md", agent.slug));
    Ok(vec![ExportedFile { path, content: content.into_bytes(), kind: ExportedFileType::Agent }])
}

/// Render a collection of hooks as a single file at the runtime's
/// `hooks_file` path.
///
/// Returns an empty `Vec` if the runtime has no hooks file or the input
/// slice is empty — either case is a no-op, not an error.
pub fn render_hooks<R: CodingAgentRuntime + ?Sized>(
    runtime: &R,
    hooks: &[Hook],
    registry: &ToolRegistry,
) -> Result<Vec<ExportedFile>> {
    if hooks.is_empty() {
        return Ok(Vec::new());
    }
    let Some(hooks_file) = runtime.paths().hooks_file.as_ref() else {
        return Ok(Vec::new());
    };
    let content = render_hooks_json(hooks, runtime.id(), registry)?;
    Ok(vec![ExportedFile {
        path: hooks_file.for_scope(Scope::Project).clone(),
        content: content.into_bytes(),
        kind: ExportedFileType::Hook,
    }])
}

/// Render a single script file (shebang + body) under the runtime's
/// `scripts_dir`.
pub fn render_script<R: CodingAgentRuntime + ?Sized>(runtime: &R, script: &Script) -> Result<ExportedFile> {
    let content = render_script_body(script);
    let scripts_dir = runtime.paths().scripts_dir.for_scope(Scope::Project);
    let path = scripts_dir.join(script_filename(script));
    Ok(ExportedFile { path, content: content.into_bytes(), kind: ExportedFileType::Script })
}

/// Wrap a pre-rendered MCP config string in an [`ExportedFile`] at the
/// runtime's `mcp_config_file` path.
///
/// Each runtime calls the format-specific renderer (`render_claude_json`,
/// `render_codex_toml`, `render_gemini_json`) and passes the result here.
/// Returns an empty `Vec` if the runtime declares no MCP config path.
pub fn wrap_mcp_config<R: CodingAgentRuntime + ?Sized>(runtime: &R, config_content: String) -> Vec<ExportedFile> {
    let Some(mcp_file) = runtime.paths().mcp_config_file.as_ref() else {
        return Vec::new();
    };
    vec![ExportedFile {
        path: mcp_file.for_scope(Scope::Project).clone(),
        content: config_content.into_bytes(),
        kind: ExportedFileType::Config,
    }]
}
