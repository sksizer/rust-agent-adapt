//! The virtual filesystem produced by capability renderers.
//!
//! Capability trait methods never touch the real filesystem — they return
//! [`ExportedFile`] values, and callers (or the concrete install helpers in
//! downstream crates) decide whether to write them to disk, bundle them
//! into a zip, or inspect them in-memory. This keeps the rendering logic
//! pure and trivially testable.
//!
//! [`ExportedFile::content`] is `Vec<u8>` rather than `String` so binary
//! assets (images, compiled binaries for future runtimes) work without a
//! second file type. A [`ExportedFile::text`] accessor handles the
//! common UTF-8 case.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Error;

/// Classifies an [`ExportedFile`] by its role in the export. Mostly used
/// by UIs and tests that want to filter or group files without pattern-
/// matching on paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportedFileType {
    /// A rendered [`crate::Skill`] — typically a `SKILL.md` or similar.
    Skill,
    /// A rendered [`crate::Role`] body.
    Role,
    /// A rendered [`crate::Agent`] persona file.
    Agent,
    /// A hooks configuration file (e.g. `hooks.json`).
    Hook,
    /// An executable script.
    Script,
    /// A runtime configuration file (e.g. `.mcp.json`, `.codex/config.toml`).
    Config,
    /// A pack manifest or index (e.g. `CLAUDE.md`, `manifest.json`).
    Manifest,
    /// A README or other documentation file.
    Readme,
    /// Any other file type not covered above.
    Other,
}

/// One file in a virtual tree — a path, raw byte content, and a type tag.
///
/// The path is relative. It's the caller's job (or
/// [`ExportedTree::write_to_dir`] in a downstream install helper) to anchor
/// it against a concrete root directory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportedFile {
    /// Relative path of this file within the export tree.
    pub path: PathBuf,
    /// Raw bytes of the file. Use [`ExportedFile::text`] to decode as UTF-8.
    pub content: Vec<u8>,
    /// Classification of what this file represents.
    pub kind: ExportedFileType,
}

impl ExportedFile {
    /// Construct an [`ExportedFile`] from a UTF-8 string body. The most
    /// common case; binary callers should build the struct directly.
    pub fn text_file(path: impl Into<PathBuf>, text: impl Into<String>, kind: ExportedFileType) -> Self {
        Self { path: path.into(), content: text.into().into_bytes(), kind }
    }

    /// Interpret [`ExportedFile::content`] as UTF-8 text. Returns an
    /// [`Error::Render`] if the bytes aren't valid UTF-8.
    pub fn text(&self) -> Result<&str, Error> {
        std::str::from_utf8(&self.content)
            .map_err(|e| Error::render(format!("exported file {:?} is not utf-8: {e}", self.path)))
    }
}

/// An ordered collection of [`ExportedFile`]s produced by a render
/// function.
///
/// Insertion order is preserved so deterministic snapshot tests don't have
/// to sort. Duplicate paths are *not* deduplicated at insert time — that's
/// a caller concern if it matters.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExportedTree {
    files: Vec<ExportedFile>,
}

impl ExportedTree {
    /// Create an empty tree.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a single file to the tree.
    pub fn push(&mut self, file: ExportedFile) {
        self.files.push(file);
    }

    /// Append multiple files at once. Takes any `IntoIterator<Item = ExportedFile>`.
    pub fn extend(&mut self, files: impl IntoIterator<Item = ExportedFile>) {
        self.files.extend(files);
    }

    /// Borrow the files as a slice.
    pub fn as_slice(&self) -> &[ExportedFile] {
        &self.files
    }

    /// Consume the tree and return its files as a `Vec`.
    pub fn into_files(self) -> Vec<ExportedFile> {
        self.files
    }

    /// Number of files in the tree.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether the tree has no files.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Iterate over the files.
    pub fn iter(&self) -> std::slice::Iter<'_, ExportedFile> {
        self.files.iter()
    }
}

impl IntoIterator for ExportedTree {
    type Item = ExportedFile;
    type IntoIter = std::vec::IntoIter<ExportedFile>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.into_iter()
    }
}

impl<'a> IntoIterator for &'a ExportedTree {
    type Item = &'a ExportedFile;
    type IntoIter = std::slice::Iter<'a, ExportedFile>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.iter()
    }
}

impl FromIterator<ExportedFile> for ExportedTree {
    fn from_iter<I: IntoIterator<Item = ExportedFile>>(iter: I) -> Self {
        Self { files: iter.into_iter().collect() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_file_roundtrips() {
        let f = ExportedFile::text_file("skills/x.md", "hello", ExportedFileType::Skill);
        assert_eq!(f.text().unwrap(), "hello");
        assert_eq!(f.kind, ExportedFileType::Skill);
    }

    #[test]
    fn text_errors_on_invalid_utf8() {
        let f = ExportedFile { path: "bad.bin".into(), content: vec![0xff, 0xfe, 0xfd], kind: ExportedFileType::Other };
        assert!(f.text().is_err());
    }

    #[test]
    fn tree_preserves_insertion_order() {
        let mut t = ExportedTree::new();
        t.push(ExportedFile::text_file("a", "1", ExportedFileType::Skill));
        t.push(ExportedFile::text_file("b", "2", ExportedFileType::Skill));
        t.push(ExportedFile::text_file("c", "3", ExportedFileType::Skill));
        let paths: Vec<_> = t.iter().map(|f| f.path.to_str().unwrap()).collect();
        assert_eq!(paths, vec!["a", "b", "c"]);
    }

    #[test]
    fn tree_extend_works() {
        let mut t = ExportedTree::new();
        t.extend([
            ExportedFile::text_file("a", "1", ExportedFileType::Skill),
            ExportedFile::text_file("b", "2", ExportedFileType::Skill),
        ]);
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn tree_from_iter() {
        let t: ExportedTree = vec![
            ExportedFile::text_file("a", "1", ExportedFileType::Skill),
            ExportedFile::text_file("b", "2", ExportedFileType::Skill),
        ]
        .into_iter()
        .collect();
        assert_eq!(t.len(), 2);
    }
}
