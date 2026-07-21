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
    emitted_names: Vec<String>,
}

impl<'a> FrontmatterBuilder<'a> {
    /// Start a new frontmatter block with the given dialect.
    pub fn new(dialect: &'a FrontmatterDialect) -> Self {
        Self { dialect, lines: Vec::new(), emitted_names: Vec::new() }
    }

    fn render_name(&mut self, canonical: &str) -> String {
        let name = match self.dialect.field_naming {
            FieldNaming::Snake => canonical.to_string(),
            FieldNaming::Kebab => canonical.replace('_', "-"),
        };
        self.emitted_names.push(name.clone());
        name
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

    /// Emit an annotation entry: a verbatim field name and an arbitrary
    /// JSON value rendered as YAML.
    ///
    /// Annotations are frontmatter keys the model does not understand, so
    /// no dialect naming conversion and no `omit_fields` check apply — the
    /// key is emitted exactly as given. Nested objects render as indented
    /// block maps with sorted keys, arrays as block sequences, and strings
    /// are double-quoted whenever a plain scalar would re-parse as a
    /// different type.
    ///
    /// An entry whose name matches a field this builder already emitted is
    /// skipped — the modeled field wins, and the frontmatter never carries
    /// a duplicate key.
    pub fn raw_entry(&mut self, name: &str, value: &serde_json::Value) -> &mut Self {
        if self.emitted_names.iter().any(|n| n == name) {
            return self;
        }
        self.emitted_names.push(name.to_string());
        emit_yaml_entry(name, value, 0, &mut self.lines);
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

/// Append `key: value` (recursing into arrays and objects) at the given
/// indent level. Object keys are sorted so output is deterministic even
/// when `serde_json` is built with `preserve_order`.
fn emit_yaml_entry(key: &str, value: &serde_json::Value, indent: usize, out: &mut Vec<String>) {
    use serde_json::Value;
    let pad = "  ".repeat(indent);
    match value {
        Value::Array(items) if items.is_empty() => out.push(format!("{pad}{key}: []")),
        Value::Object(map) if map.is_empty() => out.push(format!("{pad}{key}: {{}}")),
        Value::Array(items) => {
            out.push(format!("{pad}{key}:"));
            for item in items {
                emit_yaml_sequence_item(item, indent + 1, out);
            }
        }
        Value::Object(map) => {
            out.push(format!("{pad}{key}:"));
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            for k in keys {
                emit_yaml_entry(k, &map[k.as_str()], indent + 1, out);
            }
        }
        scalar => out.push(format!("{pad}{key}: {}", yaml_scalar(scalar))),
    }
}

/// Append one `- item` sequence entry, recursing for nested collections.
fn emit_yaml_sequence_item(item: &serde_json::Value, indent: usize, out: &mut Vec<String>) {
    use serde_json::Value;
    let pad = "  ".repeat(indent);
    match item {
        Value::Array(items) if items.is_empty() => out.push(format!("{pad}- []")),
        Value::Object(map) if map.is_empty() => out.push(format!("{pad}- {{}}")),
        Value::Array(items) => {
            out.push(format!("{pad}-"));
            for nested in items {
                emit_yaml_sequence_item(nested, indent + 1, out);
            }
        }
        Value::Object(map) => {
            out.push(format!("{pad}-"));
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            for k in keys {
                emit_yaml_entry(k, &map[k.as_str()], indent + 1, out);
            }
        }
        scalar => out.push(format!("{pad}- {}", yaml_scalar(scalar))),
    }
}

/// Render a scalar JSON value as a YAML scalar.
///
/// Strings emit plain only when re-parsing them cannot change their type
/// or structure; anything ambiguous (empty, bool/null/number lookalikes,
/// YAML indicator characters, leading/trailing spaces, newlines) is
/// double-quoted via JSON escaping, which is valid YAML.
fn yaml_scalar(value: &serde_json::Value) -> String {
    use serde_json::Value;
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) if plain_scalar_safe(s) => s.clone(),
        Value::String(s) => Value::String(s.clone()).to_string(),
        // Arrays/objects never reach here — the emitters above match them.
        other => other.to_string(),
    }
}

fn plain_scalar_safe(s: &str) -> bool {
    let Some(first) = s.chars().next() else {
        return false;
    };
    let lower = s.to_ascii_lowercase();
    if matches!(lower.as_str(), "true" | "false" | "null" | "~" | "yes" | "no" | "on" | "off") {
        return false;
    }
    if s.parse::<f64>().is_ok() {
        return false;
    }
    (first.is_ascii_alphanumeric() || first == '_' || first == '/')
        && !s.ends_with(' ')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '-' | '.' | '/'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
    fn raw_entry_scalar_kinds() {
        let d = kebab();
        let mut b = FrontmatterBuilder::new(&d);
        b.raw_entry("count", &json!(3));
        b.raw_entry("ratio", &json!(1.5));
        b.raw_entry("enabled", &json!(true));
        b.raw_entry("nothing", &json!(null));
        b.raw_entry("label", &json!("plain value"));
        let out = b.build();
        assert!(out.contains("count: 3"));
        assert!(out.contains("ratio: 1.5"));
        assert!(out.contains("enabled: true"));
        assert!(out.contains("nothing: null"));
        assert!(out.contains("label: plain value"));
    }

    #[test]
    fn raw_entry_quotes_ambiguous_strings() {
        let d = kebab();
        let mut b = FrontmatterBuilder::new(&d);
        b.raw_entry("looks_bool", &json!("true"));
        b.raw_entry("looks_num", &json!("42"));
        b.raw_entry("has_colon", &json!("key: value"));
        b.raw_entry("multiline", &json!("a\nb"));
        b.raw_entry("empty", &json!(""));
        let out = b.build();
        assert!(out.contains("looks_bool: \"true\""));
        assert!(out.contains("looks_num: \"42\""));
        assert!(out.contains("has_colon: \"key: value\""));
        assert!(out.contains("multiline: \"a\\nb\""));
        assert!(out.contains("empty: \"\""));
    }

    #[test]
    fn raw_entry_name_is_verbatim_not_dialected() {
        let d = kebab();
        let mut b = FrontmatterBuilder::new(&d);
        b.raw_entry("snake_key", &json!("v"));
        let out = b.build();
        assert!(out.contains("snake_key: v"));
        assert!(!out.contains("snake-key"));
    }

    #[test]
    fn raw_entry_nested_collections() {
        let d = kebab();
        let mut b = FrontmatterBuilder::new(&d);
        b.raw_entry("meta", &json!({"zeta": [1, 2], "alpha": {"inner": "x"}}));
        b.raw_entry("list", &json!(["a", {"k": "v"}]));
        b.raw_entry("empty_list", &json!([]));
        b.raw_entry("empty_map", &json!({}));
        let out = b.build();
        let expected = "\
meta:
  alpha:
    inner: x
  zeta:
    - 1
    - 2
list:
  - a
  -
    k: v
empty_list: []
empty_map: {}
";
        assert_eq!(out, format!("---\n{expected}---\n"));
    }

    #[test]
    fn raw_entry_skips_names_already_emitted() {
        let d = kebab();
        let mut b = FrontmatterBuilder::new(&d);
        b.scalar("model", "opus");
        b.raw_entry("model", &json!("haiku"));
        b.raw_entry("model", &json!("sonnet"));
        let out = b.build();
        assert_eq!(out.matches("model:").count(), 1);
        assert!(out.contains("model: opus"));
    }

    #[test]
    fn raw_entry_duplicate_guard_uses_rendered_name() {
        let d = kebab();
        let mut b = FrontmatterBuilder::new(&d);
        b.list("allowed_tools", &["Read".into()]);
        b.raw_entry("allowed-tools", &json!("clobber"));
        let out = b.build();
        assert_eq!(out.matches("allowed-tools:").count(), 1);
        assert!(!out.contains("clobber"));
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
