//! Codex — `.codex/config.toml`
//!
//! ```toml
//! [mcp_servers.<name>]
//! url = "http://..."
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use crate::{Error, McpServer, McpTransport};

use super::AgentConfigProvider;

pub struct CodexProvider;

impl AgentConfigProvider for CodexProvider {
    fn agent_name(&self) -> &str {
        "Codex"
    }

    fn config_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(".codex").join("config.toml")
    }

    fn install(&self, project_root: &Path, server: &McpServer) -> Result<(), Error> {
        let dir = project_root.join(".codex");
        fs::create_dir_all(&dir)?;
        let path = self.config_path(project_root);

        let mut root: toml::Value =
            if path.exists() { fs::read_to_string(&path)?.parse()? } else { toml::Value::Table(toml::map::Map::new()) };

        let table = root.as_table_mut().ok_or(Error::InvalidFormat("root is not a table"))?;

        let servers = table.entry("mcp_servers").or_insert_with(|| toml::Value::Table(toml::map::Map::new()));

        let servers_table = servers.as_table_mut().ok_or(Error::InvalidFormat("mcp_servers is not a table"))?;

        let entry = match &server.transport {
            McpTransport::Http { url } => {
                let mut m = toml::map::Map::new();
                m.insert("url".to_string(), toml::Value::String(url.clone()));
                toml::Value::Table(m)
            }
            McpTransport::Stdio { command, args, .. } => {
                let mut m = toml::map::Map::new();
                m.insert("command".to_string(), toml::Value::String(command.clone()));
                m.insert(
                    "args".to_string(),
                    toml::Value::Array(args.iter().map(|a| toml::Value::String(a.clone())).collect()),
                );
                toml::Value::Table(m)
            }
        };

        servers_table.insert(server.name.clone(), entry);
        fs::write(&path, toml::to_string_pretty(&root)?)?;
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
        assert!(!dir.path().join(".codex").exists());

        let server = McpServer::http("ontological", "http://localhost:4243/mcp");
        CodexProvider.install(dir.path(), &server).unwrap();

        let content = fs::read_to_string(dir.path().join(".codex/config.toml")).unwrap();
        let v: toml::Value = content.parse().unwrap();
        assert_eq!(v["mcp_servers"]["ontological"]["url"].as_str().unwrap(), "http://localhost:4243/mcp");
    }

    #[test]
    fn preserves_existing_entries() {
        let dir = TempDir::new().unwrap();
        let codex_dir = dir.path().join(".codex");
        fs::create_dir_all(&codex_dir).unwrap();
        fs::write(
            codex_dir.join("config.toml"),
            "[model]\nname = \"gpt-4\"\n\n[mcp_servers.other]\nurl = \"http://other\"\n",
        )
        .unwrap();

        let server = McpServer::http("ontological", "http://localhost:4243/mcp");
        CodexProvider.install(dir.path(), &server).unwrap();

        let content = fs::read_to_string(codex_dir.join("config.toml")).unwrap();
        let v: toml::Value = content.parse().unwrap();
        assert_eq!(v["model"]["name"].as_str().unwrap(), "gpt-4");
        assert_eq!(v["mcp_servers"]["other"]["url"].as_str().unwrap(), "http://other");
        assert_eq!(v["mcp_servers"]["ontological"]["url"].as_str().unwrap(), "http://localhost:4243/mcp");
    }
}
