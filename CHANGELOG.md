# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.3.0] - 2026-07-20

### Added

- `Skill::annotations` — unrecognized frontmatter keys carried on the
  model and re-emitted verbatim (sorted, after the modeled fields) by
  every skill render. Keys colliding with an emitted modeled field are
  skipped. Makes disk → model → disk round-trips lossless for keys the
  model does not understand.
- `Skill::resources` + `SkillResource` — peer files (scripts, templates,
  references) rendered beside `SKILL.md` under the slug directory, with
  a per-file executable flag. New `ExportedFileType::Resource` variant.
- `ExportedFile::executable` — install layers now set (or clear) unix
  executable bits when materializing files. Rendered `Script` files are
  marked executable.
- `runtimes::skill_renderer_for(id)` — look up a runtime as a boxed
  `SkillCapability`, so callers can render skills from a `RuntimeId`
  without matching on concrete runtime types.
- `FrontmatterBuilder::raw_entry` — emit a verbatim-named field with an
  arbitrary JSON value rendered as YAML (nested maps/sequences, quoting
  whenever a plain scalar would re-parse as a different type).

### Changed

- **Breaking:** `Skill` and `ExportedFile` gained public fields; code
  constructing them with struct literals must add the new fields
  (`Skill::new` and `ExportedFile::text_file` are unaffected).
  `ExportedFileType` gained a variant; exhaustive matches must cover
  `Resource`.

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


