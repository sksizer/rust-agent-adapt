//! Format-specific MCP server renderers.
//!
//! Each function here produces the standalone config file for a single
//! [`crate::mcp::McpServer`] in one runtime's format:
//!
//! * [`render_claude_json`] — `{ "mcpServers": { "<name>": { "type": "http", "url": "..." } } }`
//! * [`render_codex_toml`] — `[mcp_servers.<name>]` table
//! * [`render_gemini_json`] — same shape as Claude but with `httpUrl` instead of `url`
//!
//! Runtime [`crate::mcp::McpCapability`] impls call these and wrap the
//! result in an [`crate::ExportedFile`].
//!
//! These helpers emit a *standalone* config containing only the given
//! server. Merging into an existing user-owned config file is a future
//! [`crate::install`] concern.

use crate::mcp::{McpServer, McpTransport};
use crate::{Error, Result};

/// Render an MCP server as the full contents of Claude Code's `.mcp.json`
/// file (containing only this one server).
pub fn render_claude_json(server: &McpServer) -> Result<String> {
    let entry = match &server.transport {
        McpTransport::Http { url } => serde_json::json!({ "type": "http", "url": url }),
        McpTransport::Stdio { command, args, env } => serde_json::json!({
            "type": "stdio",
            "command": command,
            "args": args,
            "env": env,
        }),
    };
    let root = serde_json::json!({ "mcpServers": { &server.name: entry } });
    serde_json::to_string_pretty(&root).map_err(Error::from)
}

/// Render an MCP server as the full contents of Codex CLI's
/// `.codex/config.toml` file (containing only this one server).
pub fn render_codex_toml(server: &McpServer) -> Result<String> {
    let mut server_table = toml::map::Map::new();
    match &server.transport {
        McpTransport::Http { url } => {
            server_table.insert("url".into(), toml::Value::String(url.clone()));
        }
        McpTransport::Stdio { command, args, env } => {
            server_table.insert("command".into(), toml::Value::String(command.clone()));
            server_table.insert(
                "args".into(),
                toml::Value::Array(args.iter().map(|a| toml::Value::String(a.clone())).collect()),
            );
            if !env.is_empty() {
                let mut env_table = toml::map::Map::new();
                for (k, v) in env {
                    env_table.insert(k.clone(), toml::Value::String(v.clone()));
                }
                server_table.insert("env".into(), toml::Value::Table(env_table));
            }
        }
    }

    let mut servers_table = toml::map::Map::new();
    servers_table.insert(server.name.clone(), toml::Value::Table(server_table));

    let mut root = toml::map::Map::new();
    root.insert("mcp_servers".into(), toml::Value::Table(servers_table));

    toml::to_string_pretty(&toml::Value::Table(root)).map_err(|e| Error::Serialization(e.to_string()))
}

/// Render an MCP server as the full contents of Gemini CLI's
/// `.gemini/settings.json` file (containing only this one server).
///
/// Gemini uses `httpUrl` instead of `url` for HTTP transports.
pub fn render_gemini_json(server: &McpServer) -> Result<String> {
    let entry = match &server.transport {
        McpTransport::Http { url } => serde_json::json!({ "httpUrl": url }),
        McpTransport::Stdio { command, args, env } => serde_json::json!({
            "command": command,
            "args": args,
            "env": env,
        }),
    };
    let root = serde_json::json!({ "mcpServers": { &server.name: entry } });
    serde_json::to_string_pretty(&root).map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_http_has_type_and_url() {
        let s = McpServer::http("ontological", "http://localhost:4243/mcp");
        let out = render_claude_json(&s).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["mcpServers"]["ontological"]["type"], "http");
        assert_eq!(v["mcpServers"]["ontological"]["url"], "http://localhost:4243/mcp");
    }

    #[test]
    fn claude_stdio_has_type_and_command() {
        let s = McpServer::stdio("local", "my-server", ["--flag"]);
        let out = render_claude_json(&s).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["mcpServers"]["local"]["type"], "stdio");
        assert_eq!(v["mcpServers"]["local"]["command"], "my-server");
        assert_eq!(v["mcpServers"]["local"]["args"][0], "--flag");
    }

    #[test]
    fn codex_http_table_has_url() {
        let s = McpServer::http("ontological", "http://localhost:4243/mcp");
        let out = render_codex_toml(&s).unwrap();
        let v: toml::Value = out.parse().unwrap();
        assert_eq!(v["mcp_servers"]["ontological"]["url"].as_str().unwrap(), "http://localhost:4243/mcp");
    }

    #[test]
    fn codex_stdio_table_has_command_and_args() {
        let s = McpServer::stdio("local", "my-server", ["--flag"]);
        let out = render_codex_toml(&s).unwrap();
        let v: toml::Value = out.parse().unwrap();
        assert_eq!(v["mcp_servers"]["local"]["command"].as_str().unwrap(), "my-server");
        assert_eq!(v["mcp_servers"]["local"]["args"][0].as_str().unwrap(), "--flag");
    }

    #[test]
    fn gemini_http_uses_httpurl_field() {
        let s = McpServer::http("ontological", "http://localhost:4243/mcp");
        let out = render_gemini_json(&s).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["mcpServers"]["ontological"]["httpUrl"], "http://localhost:4243/mcp");
        assert!(v["mcpServers"]["ontological"]["url"].is_null());
    }
}
