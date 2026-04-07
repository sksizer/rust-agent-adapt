mod claude;
mod codex;
mod gemini;

pub use claude::ClaudeProvider;
pub use codex::CodexProvider;
pub use gemini::GeminiProvider;

use std::path::{Path, PathBuf};

use crate::{Error, McpServer};

/// Knows how to write an [`McpServer`] entry into one agent's config file.
pub trait AgentConfigProvider {
    /// Human-readable name of the agent (used in error messages).
    fn agent_name(&self) -> &str;

    /// Path of the config file this provider manages, relative to the project root.
    fn config_path(&self, project_root: &Path) -> PathBuf;

    /// Upsert the `server` entry into the agent's config file.
    ///
    /// Merges into any existing config, preserving all other entries.
    /// Creates intermediate directories if needed.
    fn install(&self, project_root: &Path, server: &McpServer) -> Result<(), Error>;
}

/// All built-in providers, in the order they are tried.
pub fn all_providers() -> Vec<Box<dyn AgentConfigProvider>> {
    vec![Box::new(ClaudeProvider), Box::new(CodexProvider), Box::new(GeminiProvider)]
}

/// Install `server` into every known agent config file under `project_root`.
///
/// Failures are collected and returned rather than propagated — a single
/// provider failing should not prevent the others from running.
///
/// Returns a `Vec` of `(agent_name, error)` for any providers that failed.
pub fn install_to_all(project_root: &Path, server: &McpServer) -> Vec<(String, Error)> {
    all_providers()
        .into_iter()
        .filter_map(|p| p.install(project_root, server).err().map(|e| (p.agent_name().to_string(), e)))
        .collect()
}
