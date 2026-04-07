//! Canonical kebab-case slugging for skill and tool names.
//!
//! `agent-adapt` uses one slugger, period. [`slugify`] does the kebab
//! conversion once, and [`slugify_skill_name`] layers the Agent Skills
//! spec 64-character limit on top.
//!
//! # Slugging rules
//!
//! 1. Lowercase every ASCII alphabetic character.
//! 2. Keep ASCII alphanumerics and hyphens as-is.
//! 3. Replace every other character with a single hyphen.
//! 4. Collapse consecutive hyphens into one.
//! 5. Trim leading and trailing hyphens.
//!
//! [`slugify_skill_name`] additionally truncates to 64 characters and
//! re-trims any trailing hyphen introduced by truncation.

use std::collections::HashSet;

/// Maximum length of a skill name per the Agent Skills spec.
pub const SKILL_NAME_MAX_LEN: usize = 64;

/// Convert an arbitrary string into a kebab-case slug. No length limit.
///
/// # Examples
///
/// ```
/// use agent_adapt::naming::slugify;
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

/// Slugify a skill name and clamp to the Agent Skills spec's 64-character
/// limit, re-trimming any trailing hyphen introduced by truncation.
///
/// # Examples
///
/// ```
/// use agent_adapt::naming::slugify_skill_name;
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
/// The first occurrence is left untouched; subsequent occurrences gain a
/// `-1`, `-2`, ... suffix.
///
/// # Examples
///
/// ```
/// use agent_adapt::naming::deduplicate_tool_names;
/// let mut names = vec!["read".into(), "read".into(), "write".into()];
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
        assert_eq!(slugify_skill_name(&long).len(), SKILL_NAME_MAX_LEN);
    }

    #[test]
    fn slugify_skill_name_trims_hyphen_after_truncation() {
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
