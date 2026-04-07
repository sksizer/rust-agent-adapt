//! A tiny YAML-frontmatter builder tailored to the Agent Skills spec.
//!
//! Deliberately avoids pulling in a full YAML crate. The spec frontmatter
//! is a flat map of scalar strings, optional booleans, and one multiline
//! string (`description`), so hand-emitting is simpler and gives us
//! byte-stable output for snapshot tests.
//!
//! # Usage
//!
//! ```
//! use agent_adapt::{FieldNaming, FrontmatterDialect};
//! use agent_adapt::render::FrontmatterBuilder;
//!
//! let dialect = FrontmatterDialect {
//!     field_naming: FieldNaming::Kebab,
//!     omit_fields: &[],
//!     emit_user_invocable_default: false,
//! };
//! let mut b = FrontmatterBuilder::new(&dialect);
//! b.scalar("name", "my-skill");
//! b.scalar("description", "Short description");
//! b.list("allowed_tools", &["Read".into(), "Write".into()]);
//! let yaml = b.build();
//! assert!(yaml.starts_with("---\n"));
//! assert!(yaml.contains("allowed-tools:"));
//! ```

use crate::{FieldNaming, FrontmatterDialect};

/// Fluent builder that emits a YAML frontmatter block (`---` delimited)
/// honoring a [`FrontmatterDialect`].
///
/// Fields are added in the order the caller pushes them; insertion order
/// is preserved so output is deterministic and snapshot-testable. Field
/// names passed to the builder use the *canonical* snake_case form; the
/// builder converts to kebab-case when the dialect requires it.
pub struct FrontmatterBuilder<'a> {
    dialect: &'a FrontmatterDialect,
    lines: Vec<String>,
}

impl<'a> FrontmatterBuilder<'a> {
    /// Start a new frontmatter block with the given dialect.
    pub fn new(dialect: &'a FrontmatterDialect) -> Self {
        Self { dialect, lines: Vec::new() }
    }

    fn render_name(&self, canonical: &str) -> String {
        match self.dialect.field_naming {
            FieldNaming::Snake => canonical.to_string(),
            FieldNaming::Kebab => canonical.replace('_', "-"),
        }
    }

    fn is_omitted(&self, canonical: &str) -> bool {
        self.dialect.omit_fields.contains(&canonical)
    }

    /// Emit a scalar string field. Empty strings are skipped.
    ///
    /// Multi-line values automatically use YAML block-scalar syntax
    /// (`description: |`) with indented continuation lines.
    pub fn scalar(&mut self, canonical_name: &str, value: &str) -> &mut Self {
        if value.is_empty() || self.is_omitted(canonical_name) {
            return self;
        }
        let name = self.render_name(canonical_name);
        if value.contains('\n') {
            self.lines.push(format!("{name}: |"));
            for line in value.lines() {
                self.lines.push(format!("  {line}"));
            }
        } else {
            self.lines.push(format!("{name}: {value}"));
        }
        self
    }

    /// Emit a quoted scalar (wraps in double quotes). Used for fields like
    /// `argument-hint` where the spec shows a quoted example. Empty
    /// strings are skipped.
    pub fn scalar_quoted(&mut self, canonical_name: &str, value: &str) -> &mut Self {
        if value.is_empty() || self.is_omitted(canonical_name) {
            return self;
        }
        let name = self.render_name(canonical_name);
        self.lines.push(format!("{name}: \"{value}\""));
        self
    }

    /// Emit a boolean field. Unlike scalars, booleans are always written
    /// when the field is not in `omit_fields`.
    pub fn boolean(&mut self, canonical_name: &str, value: bool) -> &mut Self {
        if self.is_omitted(canonical_name) {
            return self;
        }
        let name = self.render_name(canonical_name);
        self.lines.push(format!("{name}: {value}"));
        self
    }

    /// Emit a list of strings as a YAML block sequence. Empty lists are
    /// skipped — no `foo: []` is emitted.
    pub fn list(&mut self, canonical_name: &str, items: &[String]) -> &mut Self {
        if items.is_empty() || self.is_omitted(canonical_name) {
            return self;
        }
        let name = self.render_name(canonical_name);
        self.lines.push(format!("{name}:"));
        for item in items {
            self.lines.push(format!("  - {item}"));
        }
        self
    }

    /// Finalize and return the frontmatter block including the surrounding
    /// `---` delimiters.
    pub fn build(self) -> String {
        let mut out = String::from("---\n");
        for line in self.lines {
            out.push_str(&line);
            out.push('\n');
        }
        out.push_str("---\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kebab() -> FrontmatterDialect {
        FrontmatterDialect { field_naming: FieldNaming::Kebab, omit_fields: &[], emit_user_invocable_default: false }
    }

    fn snake() -> FrontmatterDialect {
        FrontmatterDialect { field_naming: FieldNaming::Snake, omit_fields: &[], emit_user_invocable_default: false }
    }

    #[test]
    fn empty_block_has_delimiters() {
        let d = kebab();
        let b = FrontmatterBuilder::new(&d);
        assert_eq!(b.build(), "---\n---\n");
    }

    #[test]
    fn scalar_kebab_converts_name() {
        let d = kebab();
        let mut b = FrontmatterBuilder::new(&d);
        b.scalar("allowed_tools", "x");
        let out = b.build();
        assert!(out.contains("allowed-tools: x"));
        assert!(!out.contains("allowed_tools:"));
    }

    #[test]
    fn scalar_snake_preserves_name() {
        let d = snake();
        let mut b = FrontmatterBuilder::new(&d);
        b.scalar("allowed_tools", "x");
        assert!(b.build().contains("allowed_tools: x"));
    }

    #[test]
    fn empty_scalar_is_skipped() {
        let d = kebab();
        let mut b = FrontmatterBuilder::new(&d);
        b.scalar("name", "");
        assert_eq!(b.build(), "---\n---\n");
    }

    #[test]
    fn multiline_scalar_uses_block_scalar() {
        let d = kebab();
        let mut b = FrontmatterBuilder::new(&d);
        b.scalar("description", "line one\nline two");
        let out = b.build();
        assert!(out.contains("description: |"));
        assert!(out.contains("  line one"));
        assert!(out.contains("  line two"));
    }

    #[test]
    fn list_empty_skipped() {
        let d = kebab();
        let mut b = FrontmatterBuilder::new(&d);
        b.list("allowed_tools", &[]);
        assert_eq!(b.build(), "---\n---\n");
    }

    #[test]
    fn list_emits_block_sequence() {
        let d = kebab();
        let mut b = FrontmatterBuilder::new(&d);
        b.list("allowed_tools", &["Read".to_string(), "Write".to_string()]);
        let out = b.build();
        assert!(out.contains("allowed-tools:"));
        assert!(out.contains("  - Read"));
        assert!(out.contains("  - Write"));
    }

    #[test]
    fn boolean_always_emitted() {
        let d = kebab();
        let mut b = FrontmatterBuilder::new(&d);
        b.boolean("disable_model_invocation", true);
        assert!(b.build().contains("disable-model-invocation: true"));
    }

    #[test]
    fn omit_fields_drops_even_when_set() {
        let d = FrontmatterDialect {
            field_naming: FieldNaming::Kebab,
            omit_fields: &["argument_hint"],
            emit_user_invocable_default: false,
        };
        let mut b = FrontmatterBuilder::new(&d);
        b.scalar_quoted("argument_hint", "x");
        b.scalar("name", "y");
        let out = b.build();
        assert!(!out.contains("argument-hint"));
        assert!(out.contains("name: y"));
    }

    #[test]
    fn insertion_order_preserved() {
        let d = kebab();
        let mut b = FrontmatterBuilder::new(&d);
        b.scalar("name", "z");
        b.scalar("description", "d");
        b.list("allowed_tools", &["Read".into()]);
        let out = b.build();
        let name_pos = out.find("name:").unwrap();
        let desc_pos = out.find("description:").unwrap();
        let tools_pos = out.find("allowed-tools:").unwrap();
        assert!(name_pos < desc_pos);
        assert!(desc_pos < tools_pos);
    }
}
