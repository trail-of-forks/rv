#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GemNameError {
    #[error("Gem name cannot be empty")]
    Empty,
    #[error("Gem name contains invalid characters")]
    InvalidCharacters,
    #[error("Gem name cannot be `.` or `..`")]
    InvalidPathComponent,
}

/// Validate a gem name before it is used as an identifier or path component.
///
/// Gem names in lockfiles and compact-index responses use the same ASCII subset:
/// alphanumeric characters plus `.`, `_`, and `-`. Rejecting path separators and
/// path-only components here keeps untrusted names from becoming filesystem paths.
pub fn validate_gem_name(name: &str) -> Result<(), GemNameError> {
    if name.is_empty() {
        return Err(GemNameError::Empty);
    }

    if matches!(name, "." | "..") {
        return Err(GemNameError::InvalidPathComponent);
    }

    if !name
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || matches!(char, '.' | '_' | '-'))
    {
        return Err(GemNameError::InvalidCharacters);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_normal_gem_names() {
        for name in ["rack", "foo.bar_baz-1", "JSON"] {
            assert!(validate_gem_name(name).is_ok(), "{name} should be valid");
        }
    }

    #[test]
    fn rejects_path_like_gem_names() {
        for name in [
            "",
            ".",
            "..",
            "../../owned",
            r"..\..\owned",
            "/owned",
            r"C:\owned",
        ] {
            assert!(
                validate_gem_name(name).is_err(),
                "{name} should be rejected"
            );
        }
    }
}
