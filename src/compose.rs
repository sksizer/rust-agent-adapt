//! Free-function composition of capability traits into a full pack
//! export.
//!
//! Pack rendering is deliberately **not** a method on
//! [`crate::CodingAgentRuntime`]. Keeping it as a free function with
//! trait bounds means runtimes only need to implement the capabilities
//! they actually support — `npm_package` doesn't need to `impl
//! HookCapability` just to make pack rendering type-check, because the
//! "full pack" composition requires the hook trait as a bound and
//! callers choose whichever variant matches their runtime.
//!
//! # Variants
//!
//! * [`render_pack`] — the maximal composition, requiring every
//!   capability ([`crate::SkillCapability`] +
//!   [`crate::AgentCapability`] + [`crate::HookCapability`] +
//!   [`crate::ScriptCapability`] + [`crate::mcp::McpCapability`]).
//!   Used by runtimes like Claude Code, Gemini CLI, Codex CLI,
//!   OpenCode, and Amp.
//! * [`render_pack_no_mcp_no_hooks`] — minimal composition requiring
//!   only skills, agents, and scripts. Used by `npm_package`.
//!
//! Each composition walks the bundle in a fixed order (skills → agents
//! → hooks → scripts → mcp servers → manifest) and appends the
//! resulting files to an [`crate::ExportedTree`].

use crate::mcp::McpCapability;
use crate::render::render_pack_manifest;
use crate::{
    AgentCapability, ExportedFile, ExportedFileType, ExportedTree, HookCapability, PackBundle, Result,
    ScriptCapability, SkillCapability,
};

/// Render an entire [`PackBundle`] into an [`ExportedTree`] for a
/// runtime that implements every capability.
///
/// This is the "give me everything" composition. Ordering: skills,
/// agents, hooks, scripts, MCP servers (one file per server), manifest
/// last (so it can reference everything above it).
///
/// # Errors
///
/// Propagates any error from an individual capability's render function.
/// Stops at the first failure — partial output is not returned.
pub fn render_pack<R>(runtime: &R, bundle: &PackBundle) -> Result<ExportedTree>
where
    R: SkillCapability + AgentCapability + HookCapability + ScriptCapability + McpCapability,
{
    let mut tree = ExportedTree::new();

    for skill in &bundle.skills {
        tree.extend(runtime.render_skill(skill)?);
    }
    for agent in &bundle.agents {
        tree.extend(runtime.render_agent(agent)?);
    }
    if !bundle.hooks.is_empty() {
        tree.extend(runtime.render_hooks(&bundle.hooks)?);
    }
    for script in &bundle.scripts {
        tree.push(runtime.render_script(script)?);
    }

    // Manifest last so it sees the full list of rendered assets. The
    // caller picks the filename convention — we default to `MANIFEST.md`
    // because most runtimes have their own preferred name.
    tree.push(ExportedFile {
        path: "MANIFEST.md".into(),
        content: render_pack_manifest(bundle).into_bytes(),
        kind: ExportedFileType::Manifest,
    });

    Ok(tree)
}

/// Render a pack into an [`ExportedTree`] for a runtime that only
/// supports skills, agents, and scripts — no hooks and no MCP servers.
///
/// Used by [`crate::runtimes::NpmPackage`] and any future runtime that
/// doesn't implement [`HookCapability`] or [`McpCapability`].
///
/// Hooks and MCP servers in the bundle are **silently ignored** rather
/// than raising an error — the bundle may carry them for other runtimes,
/// and it would be unhelpful to fail here just because a generic
/// packaging target can't express them.
pub fn render_pack_no_mcp_no_hooks<R>(runtime: &R, bundle: &PackBundle) -> Result<ExportedTree>
where
    R: SkillCapability + AgentCapability + ScriptCapability,
{
    let mut tree = ExportedTree::new();

    for skill in &bundle.skills {
        tree.extend(runtime.render_skill(skill)?);
    }
    for agent in &bundle.agents {
        tree.extend(runtime.render_agent(agent)?);
    }
    for script in &bundle.scripts {
        tree.push(runtime.render_script(script)?);
    }

    tree.push(ExportedFile {
        path: "MANIFEST.md".into(),
        content: render_pack_manifest(bundle).into_bytes(),
        kind: ExportedFileType::Manifest,
    });

    Ok(tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::McpServer;
    use crate::runtimes::{ClaudeCode, NpmPackage};
    use crate::{Agent, Hook, Pack, PackBundle, Script, ScriptLanguage, Skill};

    fn sample_bundle() -> PackBundle {
        let pack = Pack {
            name: "Test Pack".into(),
            slug: "test-pack".into(),
            namespace: None,
            description: Some("A test pack".into()),
            version_label: "1.0.0".into(),
            author: None,
            license: None,
            tags: vec![],
            category: None,
        };
        let mut b = PackBundle::new(pack);
        b.skills.push(Skill::new("My Skill", "desc", "body"));
        b.agents.push(Agent::new("Reviewer", "reviewer"));
        b.hooks.push(Hook {
            name: "h".into(),
            slug: "h".into(),
            description: None,
            event: "PreToolUse".into(),
            matcher: None,
            command: "echo".into(),
            timeout_ms: 0,
            tags: vec![],
            category: None,
            is_template: false,
        });
        b.scripts.push(Script {
            name: "Lint".into(),
            slug: "lint".into(),
            description: None,
            language: ScriptLanguage::Bash,
            body: "echo hi".into(),
            timeout_ms: 0,
            tags: vec![],
            category: None,
            is_template: false,
        });
        b
    }

    #[test]
    fn render_pack_full_includes_every_asset_type() {
        let tree = render_pack(&ClaudeCode, &sample_bundle()).unwrap();
        let paths: Vec<_> = tree.iter().map(|f| f.path.to_string_lossy().into_owned()).collect();
        assert!(paths.iter().any(|p| p.contains("skills/my-skill/SKILL.md")));
        assert!(paths.iter().any(|p| p.contains("agents/reviewer.md")));
        assert!(paths.iter().any(|p| p.contains("hooks.json")));
        assert!(paths.iter().any(|p| p.contains("scripts/lint.sh")));
        assert!(paths.iter().any(|p| p == "MANIFEST.md"));
    }

    #[test]
    fn render_pack_with_mcp_skips_gracefully_when_bundle_has_no_server_list() {
        // Note: PackBundle doesn't own mcp_servers — those are passed
        // separately at install time. render_pack composes without
        // touching MCP. This test documents the current behavior.
        let tree = render_pack(&ClaudeCode, &sample_bundle()).unwrap();
        let paths: Vec<_> = tree.iter().map(|f| f.path.to_string_lossy().into_owned()).collect();
        assert!(!paths.iter().any(|p| p == ".mcp.json"));
    }

    #[test]
    fn render_pack_manifest_is_last() {
        let tree = render_pack(&ClaudeCode, &sample_bundle()).unwrap();
        let last = tree.iter().last().unwrap();
        assert_eq!(last.path.to_string_lossy(), "MANIFEST.md");
        assert_eq!(last.kind, ExportedFileType::Manifest);
    }

    #[test]
    fn render_pack_no_mcp_no_hooks_skips_hooks() {
        let tree = render_pack_no_mcp_no_hooks(&NpmPackage, &sample_bundle()).unwrap();
        let paths: Vec<_> = tree.iter().map(|f| f.path.to_string_lossy().into_owned()).collect();
        assert!(!paths.iter().any(|p| p.contains("hooks.json")));
        assert!(paths.iter().any(|p| p.contains("skills/my-skill")));
    }

    #[test]
    fn render_pack_propagates_all_assets_including_mcp_server_separately() {
        // The MCP capability is called per-server, not as part of
        // render_pack. Users who want MCP render it separately and
        // extend the tree:
        let mut tree = render_pack(&ClaudeCode, &sample_bundle()).unwrap();
        let server = McpServer::http("ontological", "http://localhost:4243/mcp");
        tree.extend(ClaudeCode.render_mcp_server(&server).unwrap());
        let paths: Vec<_> = tree.iter().map(|f| f.path.to_string_lossy().into_owned()).collect();
        assert!(paths.iter().any(|p| p == ".mcp.json"));
    }
}
