//! Gemini: `.gemini/settings.json`
//!
//! ```json
//! {
//!   "mcpServers": {
//!     "<name>": { "httpUrl": "..." }
//!   }
//! }
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use crate::{Error, McpServer, McpTransport};

use super::AgentConfigProvider;

pub struct GeminiProvider;

impl AgentConfigProvider for GeminiProvider {
    fn agent_name(&self) -> &str {
        "Gemini"
    }

    fn config_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(".gemini").join("settings.json")
    }

    fn install(&self, project_root: &Path, server: &McpServer) -> Result<(), Error> {
        let dir = project_root.join(".gemini");
        fs::create_dir_all(&dir)?;
        let path = self.config_path(project_root);

        let mut root: serde_json::Value =
            if path.exists() { serde_json::from_str(&fs::read_to_string(&path)?)? } else { serde_json::json!({}) };

        let servers = root
            .as_object_mut()
            .ok_or(Error::InvalidFormat("root is not an object"))?
            .entry("mcpServers")
            .or_insert_with(|| serde_json::json!({}));

        // Gemini uses `httpUrl` (not `url`) for HTTP transports.
        let entry = match &server.transport {
            McpTransport::Http { url } => {
                serde_json::json!({ "httpUrl": url })
            }
            McpTransport::Stdio { command, args, .. } => {
                serde_json::json!({ "command": command, "args": args })
            }
        };

        servers
            .as_object_mut()
            .ok_or(Error::InvalidFormat("mcpServers is not an object"))?
            .insert(server.name.clone(), entry);

        fs::write(&path, serde_json::to_string_pretty(&root)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn creates_directory_and_writes_fresh_config() {
        let dir = TempDir::new().unwrap();
        assert!(!dir.path().join(".gemini").exists());

        let server = McpServer::http("ontological", "http://localhost:4243/mcp");
        GeminiProvider.install(dir.path(), &server).unwrap();

        let content = fs::read_to_string(dir.path().join(".gemini/settings.json")).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(v["mcpServers"]["ontological"]["httpUrl"], "http://localhost:4243/mcp");
    }

    #[test]
    fn preserves_existing_entries() {
        let dir = TempDir::new().unwrap();
        let gemini_dir = dir.path().join(".gemini");
        fs::create_dir_all(&gemini_dir).unwrap();
        fs::write(
            gemini_dir.join("settings.json"),
            r#"{ "mcpServers": { "other": { "httpUrl": "http://other" } }, "theme": "dark" }"#,
        )
        .unwrap();

        let server = McpServer::http("ontological", "http://localhost:4243/mcp");
        GeminiProvider.install(dir.path(), &server).unwrap();

        let content = fs::read_to_string(gemini_dir.join("settings.json")).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(v["mcpServers"]["other"]["httpUrl"], "http://other");
        assert_eq!(v["theme"], "dark");
        assert_eq!(v["mcpServers"]["ontological"]["httpUrl"], "http://localhost:4243/mcp");
    }
}
