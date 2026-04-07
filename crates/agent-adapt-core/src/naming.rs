//! Canonical kebab-case slugging for skill and tool names.
//!
//! Agentpants historically had two copies of this logic ([`slugify_skill_name`]
//! in `export/skill_md.rs` with a 64-char truncation, and `to_tool_name` in
//! `export/validation.rs` without one), which drifted. This module is the
//! single source of truth: [`slugify`] does the kebab conversion once, and
//! [`slugify_skill_name`] layers the Agent Skills spec 64-character limit on
//! top.
//!
//! # Slugging rules
//!
//! 1. Lowercase every ASCII alphabetic character.
//! 2. Keep ASCII alphanumerics and hyphens as-is.
//! 3. Replace every other character (including spaces, underscores, and
//!    unicode punctuation) with a single hyphen.
//! 4. Collapse consecutive hyphens into one.
//! 5. Trim leading and trailing hyphens.
//!
//! [`slugify_skill_name`] additionally truncates the result to 64 characters
//! and re-trims any trailing hyphen introduced by truncation, matching the
//! Anthropic Agent Skills spec for the `name:` frontmatter field.

use std::collections::HashSet;

/// Maximum length of a skill name per the Agent Skills spec.
pub const SKILL_NAME_MAX_LEN: usize = 64;

/// Convert an arbitrary string into a kebab-case slug.
///
/// See the [module docs](self) for the exact rules. No length limit is
/// applied — use [`slugify_skill_name`] for the spec-compliant skill-name
/// variant.
///
/// # Examples
///
/// ```
/// use agent_adapt_core::naming::slugify;
/// assert_eq!(slugify("My Cool Skill"), "my-cool-skill");
/// assert_eq!(slugify("skill@v2.0!"), "skill-v2-0");
/// assert_eq!(slugify("a---b___c"), "a-b-c");
/// assert_eq!(slugify("--hello--"), "hello");
/// ```
pub fn slugify(name: &str) -> String {
    let mapped: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c.to_ascii_lowercase() } else { '-' })
        .collect();

    // Collapse consecutive hyphens.
    let mut collapsed = String::with_capacity(mapped.len());
    let mut last_was_hyphen = false;
    for c in mapped.chars() {
        if c == '-' {
            if !last_was_hyphen {
                collapsed.push(c);
            }
            last_was_hyphen = true;
        } else {
            collapsed.push(c);
            last_was_hyphen = false;
        }
    }

    collapsed.trim_matches('-').to_string()
}

/// Slugify a skill name and clamp it to the Agent Skills spec's 64-character
/// limit, re-trimming any trailing hyphen introduced by truncation.
///
/// # Examples
///
/// ```
/// use agent_adapt_core::naming::slugify_skill_name;
/// assert_eq!(slugify_skill_name("My Skill"), "my-skill");
/// let long = "a".repeat(100);
/// assert_eq!(slugify_skill_name(&long).len(), 64);
/// ```
pub fn slugify_skill_name(name: &str) -> String {
    let slug = slugify(name);
    if slug.len() > SKILL_NAME_MAX_LEN { slug[..SKILL_NAME_MAX_LEN].trim_end_matches('-').to_string() } else { slug }
}

/// Disambiguate a list of tool/skill names in place by appending numeric
/// suffixes to duplicates.
///
/// The first occurrence of each name is left untouched; subsequent
/// occurrences gain a `-1`, `-2`, ... suffix. The suffix counter restarts
/// from 1 for each distinct base name, and suffixed names that would collide
/// with an existing entry are skipped to the next integer.
///
/// # Examples
///
/// ```
/// use agent_adapt_core::naming::deduplicate_tool_names;
/// let mut names = vec!["read".to_string(), "read".to_string(), "write".to_string()];
/// deduplicate_tool_names(&mut names);
/// assert_eq!(names, vec!["read", "read-1", "write"]);
/// ```
pub fn deduplicate_tool_names(names: &mut [String]) {
    let mut seen: HashSet<String> = HashSet::new();
    for name in names.iter_mut() {
        let original = name.clone();
        let mut suffix = 1;
        while seen.contains(name) {
            *name = format!("{}-{}", original, suffix);
            suffix += 1;
        }
        seen.insert(name.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("My Skill"), "my-skill");
    }

    #[test]
    fn slugify_underscores() {
        assert_eq!(slugify("my_cool_skill"), "my-cool-skill");
    }

    #[test]
    fn slugify_special_chars() {
        assert_eq!(slugify("skill@v2.0!"), "skill-v2-0");
    }

    #[test]
    fn slugify_leading_trailing_hyphens() {
        assert_eq!(slugify("--hello--"), "hello");
    }

    #[test]
    fn slugify_consecutive_hyphens() {
        assert_eq!(slugify("a---b___c"), "a-b-c");
    }

    #[test]
    fn slugify_unicode_becomes_hyphens() {
        // Non-ASCII characters are treated as separators, consistent with the
        // agentpants implementation we replaced.
        assert_eq!(slugify("café résumé"), "caf-r-sum");
    }

    #[test]
    fn slugify_already_kebab_passthrough() {
        assert_eq!(slugify("already-kebab-case"), "already-kebab-case");
    }

    #[test]
    fn slugify_skill_name_short_passthrough() {
        assert_eq!(slugify_skill_name("short"), "short");
    }

    #[test]
    fn slugify_skill_name_truncates_to_64() {
        let long = "a".repeat(100);
        let result = slugify_skill_name(&long);
        assert_eq!(result.len(), SKILL_NAME_MAX_LEN);
    }

    #[test]
    fn slugify_skill_name_trims_hyphen_after_truncation() {
        // Construct a name whose 64th character is a hyphen, forcing retrim.
        let name = format!("{}-tail", "a".repeat(63));
        let result = slugify_skill_name(&name);
        assert!(!result.ends_with('-'));
        assert!(result.len() <= SKILL_NAME_MAX_LEN);
    }

    #[test]
    fn deduplicate_basic() {
        let mut names = vec!["read".into(), "read".into(), "write".into()];
        deduplicate_tool_names(&mut names);
        assert_eq!(names, vec!["read", "read-1", "write"]);
    }

    #[test]
    fn deduplicate_triple_collision() {
        let mut names = vec!["x".into(), "x".into(), "x".into()];
        deduplicate_tool_names(&mut names);
        assert_eq!(names, vec!["x", "x-1", "x-2"]);
    }

    #[test]
    fn deduplicate_empty_passthrough() {
        let mut names: Vec<String> = vec![];
        deduplicate_tool_names(&mut names);
        assert!(names.is_empty());
    }

    #[test]
    fn deduplicate_no_duplicates_passthrough() {
        let mut names = vec!["a".into(), "b".into(), "c".into()];
        deduplicate_tool_names(&mut names);
        assert_eq!(names, vec!["a", "b", "c"]);
    }
}
