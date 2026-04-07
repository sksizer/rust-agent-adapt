# agent-adapt

Rust data structures and adapters for working with coding-agent configuration
across the different formats that agents use.

Coding agents (Claude Code, Codex, Gemini, and others) each keep their own
config files in their own formats and locations. `agent-adapt` provides:

- a neutral data model for concepts that live in those configs, and
- per-agent provider implementations that read/write each agent's native format.

The goal is that a tool integrating with multiple agents can describe *what* it
wants to configure once, and let `agent-adapt` translate that into the right
file in the right shape for each agent.

## Status

Early. The initial focus is MCP server registration — registering an MCP server
entry into each agent's config file without clobbering unrelated entries. More
configuration surfaces (permissions, tool allowlists, model defaults, etc.) will
land over time as the data model is generalized.

## Supported agents

| Agent       | Config file                  | Format |
|-------------|------------------------------|--------|
| Claude Code | `.mcp.json`                  | JSON   |
| Codex       | `.codex/config.toml`         | TOML   |
| Gemini      | `.gemini/settings.json`      | JSON   |

Paths are resolved relative to a caller-supplied project root.

## Example — register an MCP server with every known agent

```rust
use agent_adapt::{McpServer, providers::install_to_all};
use std::path::Path;

let server = McpServer::http("my-server", "http://localhost:4243/mcp");
let failures = install_to_all(Path::new("/path/to/project"), &server);

for (agent, err) in failures {
    eprintln!("failed to configure {agent}: {err}");
}
```

Each provider merges into any existing config, preserving unrelated entries,
and creates intermediate directories as needed.

## Example — target a single agent

```rust
use agent_adapt::{McpServer, providers::ClaudeProvider, AgentConfigProvider};
use std::path::Path;

let server = McpServer::stdio("my-server", "my-binary", ["--serve"]);
ClaudeProvider.install(Path::new("/path/to/project"), &server)?;
# Ok::<(), agent_adapt::Error>(())
```

## Design notes

- **Non-destructive merges.** Providers parse the existing file, upsert the
  entry, and write it back. Other entries and unrelated keys are left alone.
- **Best-effort fan-out.** `install_to_all` never short-circuits — it runs
  every provider and returns the collected failures so one agent's breakage
  can't block the others.
- **Extensible.** Implement `AgentConfigProvider` to add a new agent.

## License

MIT. See [LICENSE.md](./LICENSE.md).
