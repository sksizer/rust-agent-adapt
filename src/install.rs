//! Concrete filesystem helpers on top of [`ExportedTree`] and the
//! capability traits.
//!
//! Everything above this layer is pure: renders return
//! [`ExportedFile`]s, not disk writes. This module is the thin
//! side-effectful edge that materializes those virtual files onto a
//! real filesystem, anchored against a caller-supplied project root.
//!
//! # Tree writing
//!
//! [`write_tree`] takes an [`ExportedTree`] and writes each
//! [`ExportedFile`] to `<anchor>/<file.path>`, creating parent
//! directories as needed. It returns the list of absolute paths it
//! wrote so callers can report or post-process them.
//!
//! # Per-capability shortcuts
//!
//! [`install_skill`], [`install_agent`], [`install_hooks`],
//! [`install_script`], and [`install_mcp_server`] are one-liner
//! shortcuts that render + write in a single call. They exist so
//! callers don't have to build an [`ExportedTree`] just to install a
//! single asset.
//!
//! # Pack installation
//!
//! [`install_pack`] composes [`crate::compose::render_pack`] with
//! [`write_tree`] to install an entire [`PackBundle`] with one call.

use std::fs;
use std::path::{Path, PathBuf};

use crate::mcp::{McpCapability, McpServer};
use crate::{
    Agent, AgentCapability, ExportedFile, ExportedTree, Hook, HookCapability, PackBundle, Result, Script,
    ScriptCapability, Skill, SkillCapability, compose, runtime::Scope,
};

impl ExportedTree {
    /// Write every file in the tree to disk, anchored at `anchor`.
    ///
    /// See [`write_tree`] for details. This is a method-call alias for
    /// callers that already have a tree in hand.
    pub fn write_to_dir(&self, anchor: &Path) -> Result<Vec<PathBuf>> {
        write_tree(self, anchor)
    }
}

/// Write every file in `tree` to disk, anchored at `anchor`.
///
/// Each [`ExportedFile::path`] is joined onto `anchor` to produce an
/// absolute path; parent directories are created recursively. Returns
/// the list of absolute paths written, in the order the tree yielded
/// them.
///
/// # Errors
///
/// Stops at the first I/O failure. Files written before the failure
/// remain on disk — this function does not roll back. Callers that
/// need atomic behavior should write to a staging directory and
/// `rename` when the full tree succeeds.
pub fn write_tree(tree: &ExportedTree, anchor: &Path) -> Result<Vec<PathBuf>> {
    let mut written = Vec::with_capacity(tree.len());
    for file in tree.iter() {
        written.push(write_one(file, anchor)?);
    }
    Ok(written)
}

fn write_one(file: &ExportedFile, anchor: &Path) -> Result<PathBuf> {
    let absolute = anchor.join(&file.path);
    if let Some(parent) = absolute.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&absolute, &file.content)?;
    Ok(absolute)
}

/// Render and install a single [`Skill`] onto `project_root`.
///
/// Delegates to [`SkillCapability::render_skill`] + [`write_tree`].
/// Returns the absolute paths written.
///
/// The `scope` parameter is currently informational — path resolution
/// uses the runtime's project-scope paths. Future versions will honor
/// `Scope::User` by anchoring against the user's home directory.
pub fn install_skill<R: SkillCapability>(
    runtime: &R,
    project_root: &Path,
    _scope: Scope,
    skill: &Skill,
) -> Result<Vec<PathBuf>> {
    let files = runtime.render_skill(skill)?;
    let tree: ExportedTree = files.into_iter().collect();
    write_tree(&tree, project_root)
}

/// Render and install a single [`Agent`] onto `project_root`.
pub fn install_agent<R: AgentCapability>(
    runtime: &R,
    project_root: &Path,
    _scope: Scope,
    agent: &Agent,
) -> Result<Vec<PathBuf>> {
    let files = runtime.render_agent(agent)?;
    let tree: ExportedTree = files.into_iter().collect();
    write_tree(&tree, project_root)
}

/// Render and install a collection of [`Hook`]s onto `project_root`.
///
/// Empty slice or runtimes with no hooks file produce a no-op
/// (returning an empty `Vec`), not an error.
pub fn install_hooks<R: HookCapability>(
    runtime: &R,
    project_root: &Path,
    _scope: Scope,
    hooks: &[Hook],
) -> Result<Vec<PathBuf>> {
    let files = runtime.render_hooks(hooks)?;
    let tree: ExportedTree = files.into_iter().collect();
    write_tree(&tree, project_root)
}

/// Render and install a single [`Script`] onto `project_root`.
pub fn install_script<R: ScriptCapability>(
    runtime: &R,
    project_root: &Path,
    _scope: Scope,
    script: &Script,
) -> Result<PathBuf> {
    let file = runtime.render_script(script)?;
    let written = write_one(&file, project_root)?;
    Ok(written)
}

/// Render and install a single [`McpServer`] onto `project_root`.
///
/// Writes a **standalone** config file containing only this server.
/// Callers that need to merge into an existing user-owned config
/// should use [`crate::providers`] instead (the legacy v0.1 merge-aware
/// installer).
pub fn install_mcp_server<R: McpCapability>(
    runtime: &R,
    project_root: &Path,
    _scope: Scope,
    server: &McpServer,
) -> Result<Vec<PathBuf>> {
    let files = runtime.render_mcp_server(server)?;
    let tree: ExportedTree = files.into_iter().collect();
    write_tree(&tree, project_root)
}

/// Render and install an entire [`PackBundle`] onto `project_root`.
///
/// Requires the runtime to implement every capability. For generic
/// packaging targets without hooks or MCP, use
/// [`install_pack_no_mcp_no_hooks`].
pub fn install_pack<R>(runtime: &R, project_root: &Path, bundle: &PackBundle) -> Result<Vec<PathBuf>>
where
    R: SkillCapability + AgentCapability + HookCapability + ScriptCapability + McpCapability,
{
    let tree = compose::render_pack(runtime, bundle)?;
    write_tree(&tree, project_root)
}

/// Render and install a [`PackBundle`] onto `project_root` for a
/// runtime that only supports skills, agents, and scripts.
pub fn install_pack_no_mcp_no_hooks<R>(runtime: &R, project_root: &Path, bundle: &PackBundle) -> Result<Vec<PathBuf>>
where
    R: SkillCapability + AgentCapability + ScriptCapability,
{
    let tree = compose::render_pack_no_mcp_no_hooks(runtime, bundle)?;
    write_tree(&tree, project_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::McpServer;
    use crate::runtimes::ClaudeCode;
    use crate::{Pack, PackBundle, Skill};
    use tempfile::TempDir;

    #[test]
    fn write_tree_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let mut tree = ExportedTree::new();
        tree.push(ExportedFile::text_file("deeply/nested/file.md", "hello", crate::ExportedFileType::Skill));
        let written = write_tree(&tree, dir.path()).unwrap();
        assert_eq!(written.len(), 1);
        assert!(written[0].exists());
        assert_eq!(fs::read_to_string(&written[0]).unwrap(), "hello");
    }

    #[test]
    fn install_skill_writes_to_runtime_convention() {
        let dir = TempDir::new().unwrap();
        let skill = Skill::new("My Skill", "desc", "body");
        let written = install_skill(&ClaudeCode, dir.path(), Scope::Project, &skill).unwrap();
        assert_eq!(written.len(), 1);
        let expected = dir.path().join(".claude/skills/my-skill/SKILL.md");
        assert_eq!(written[0], expected);
        assert!(expected.exists());
    }

    #[test]
    fn install_mcp_server_writes_claude_json_at_project_root() {
        let dir = TempDir::new().unwrap();
        let server = McpServer::http("ontological", "http://localhost:4243/mcp");
        let written = install_mcp_server(&ClaudeCode, dir.path(), Scope::Project, &server).unwrap();
        assert_eq!(written.len(), 1);
        assert_eq!(written[0], dir.path().join(".mcp.json"));
        let content = fs::read_to_string(&written[0]).unwrap();
        assert!(content.contains("\"mcpServers\""));
        assert!(content.contains("\"ontological\""));
    }

    #[test]
    fn install_pack_writes_full_tree() {
        let dir = TempDir::new().unwrap();
        let pack = Pack {
            name: "Test Pack".into(),
            slug: "test-pack".into(),
            namespace: None,
            description: None,
            version_label: "1.0.0".into(),
            author: None,
            license: None,
            tags: vec![],
            category: None,
        };
        let mut bundle = PackBundle::new(pack);
        bundle.skills.push(Skill::new("One", "d", "b"));
        bundle.skills.push(Skill::new("Two", "d", "b"));

        let written = install_pack(&ClaudeCode, dir.path(), &bundle).unwrap();
        assert!(written.iter().any(|p| p.ends_with(".claude/skills/one/SKILL.md")));
        assert!(written.iter().any(|p| p.ends_with(".claude/skills/two/SKILL.md")));
        assert!(written.iter().any(|p| p.ends_with("MANIFEST.md")));
    }

    #[test]
    fn exported_tree_method_alias_works() {
        let dir = TempDir::new().unwrap();
        let mut tree = ExportedTree::new();
        tree.push(ExportedFile::text_file("a.txt", "x", crate::ExportedFileType::Other));
        let written = tree.write_to_dir(dir.path()).unwrap();
        assert_eq!(written.len(), 1);
    }
}
