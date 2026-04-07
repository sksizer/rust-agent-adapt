# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.2.0] - 2026-04-07

### Removed

- legacy v0.1 `providers/` module (`AgentConfigProvider`, `ClaudeProvider`,
  `CodexProvider`, `GeminiProvider`, `all_providers`, `install_to_all`).
  The merge-into-existing-config MCP installer is replaced by the new
  `install::install_mcp_server` + `runtimes::*` capability traits, which
  emit standalone config files. A future install helper will add
  merge-aware writes when the need arises.

## [0.1.1] - 2026-04-07

### Added

- initial agent-adapt crate
- convert to workspace and add agent-adapt-core
- add 6 runtime adapters, pack composition, and install helpers

### Changed

- collapse workspace into single agent-adapt crate and expand scope


