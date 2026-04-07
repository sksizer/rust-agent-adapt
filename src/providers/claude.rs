//! Claude Code — `.mcp.json`
//!
//! ```json
//! {
//!   "mcpServers": {
//!     "<name>": { "type": "http", "url": "..." }
//!   }
//! }
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use crate::{Error, McpServer, McpTransport};

use super::AgentConfigProvider;

pub struct ClaudeProvider;

impl AgentConfigProvider for ClaudeProvider {
    fn agent_name(&self) -> &str {
        "Claude Code"
    }

    fn config_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(".mcp.json")
    }

    fn install(&self, project_root: &Path, server: &McpServer) -> Result<(), Error> {
        let path = self.config_path(project_root);

        let mut root: serde_json::Value =
            if path.exists() { serde_json::from_str(&fs::read_to_string(&path)?)? } else { serde_json::json!({}) };

        let servers = root
            .as_object_mut()
            .ok_or(Error::InvalidFormat("root is not an object"))?
            .entry("mcpServers")
            .or_insert_with(|| serde_json::json!({}));

        let entry = match &server.transport {
            McpTransport::Http { url } => {
                serde_json::json!({ "type": "http", "url": url })
            }
            McpTransport::Stdio { command, args, env } => {
                serde_json::json!({
                    "type": "stdio",
                    "command": command,
                    "args": args,
                    "env": env,
                })
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
    use crate::providers::install_to_all;
    use tempfile::TempDir;

    #[test]
    fn fresh_config_is_written() {
        let dir = TempDir::new().unwrap();
        let server = McpServer::http("ontological", "http://localhost:4243/mcp");
        ClaudeProvider.install(dir.path(), &server).expect("install should succeed");

        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join(".mcp.json")).unwrap()).unwrap();
        assert_eq!(v["mcpServers"]["ontological"]["type"], "http");
        assert_eq!(v["mcpServers"]["ontological"]["url"], "http://localhost:4243/mcp");
    }

    #[test]
    fn existing_servers_are_preserved() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join(".mcp.json"),
            r#"{ "mcpServers": { "other": { "type": "stdio", "command": "test" } } }"#,
        )
        .unwrap();

        let server = McpServer::http("ontological", "http://localhost:4243/mcp");
        ClaudeProvider.install(dir.path(), &server).unwrap();

        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join(".mcp.json")).unwrap()).unwrap();
        assert_eq!(v["mcpServers"]["other"]["type"], "stdio");
        assert_eq!(v["mcpServers"]["ontological"]["type"], "http");
    }

    #[test]
    fn install_to_all_uses_custom_port() {
        let dir = TempDir::new().unwrap();
        let server = McpServer::http("ontological", "http://localhost:9999/mcp");
        let errors = install_to_all(dir.path(), &server);
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");

        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join(".mcp.json")).unwrap()).unwrap();
        assert_eq!(v["mcpServers"]["ontological"]["url"], "http://localhost:9999/mcp");
    }
}
