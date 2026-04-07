# agent-adapt

Rust data structures and adapters for working with coding-agent configuration
across the different formats that agents use.

Coding agents (Claude Code, Codex, Gemini, ...) each keep their own config
files in their own formats and locations. `agent-adapt` provides a neutral
data model and per-agent providers that translate it into each agent's native
file.

## Supported agents

| Agent       | Config file              | Format |
|-------------|--------------------------|--------|
| Claude Code | `.mcp.json`              | JSON   |
| Codex       | `.codex/config.toml`     | TOML   |
| Gemini      | `.gemini/settings.json`  | JSON   |

Paths are resolved relative to a caller-supplied project root. Providers merge
into any existing config, preserving unrelated entries, and create intermediate
directories as needed.

## Example: register an MCP server with every known agent

```rust
use agent_adapt::{McpServer, providers::install_to_all};
use std::path::Path;

let server = McpServer::http("my-server", "http://localhost:4243/mcp");
let failures = install_to_all(Path::new("/path/to/project"), &server);

for (agent, err) in failures {
    eprintln!("failed to configure {agent}: {err}");
}
```

## Example: target a single agent

```rust
use agent_adapt::{McpServer, providers::ClaudeProvider, AgentConfigProvider};
use std::path::Path;

let server = McpServer::stdio("my-server", "my-binary", ["--serve"]);
ClaudeProvider.install(Path::new("/path/to/project"), &server)?;
# Ok::<(), agent_adapt::Error>(())
```

## License

MIT. See [LICENSE.md](./LICENSE.md).
