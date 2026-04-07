# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-04-06

### Added

- initial commit
- add serde serialization example
- add VSCode settings
- justfile
- add automatic changelog generation with git-cliff
- add batch bring-up-to-date for all projects and improve execution
- add cargo dependency update scripts

### Changed

- run only cargo clippy
- extract Point into library crate and add release tooling
- replace local paths with git URLs for downstream projects

### Fixed

- handle empty ARGS array safely in cargo_update_all


