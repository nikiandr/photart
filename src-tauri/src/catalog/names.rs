use super::error::CatalogError;

/// Validates a Catalog or Shoot name as a single path component.
///
/// Rejects empty (after trim), `.`, `..`, path separators, and multi-component names.
pub fn validate_single_path_component_name(name: &str) -> Result<String, CatalogError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(CatalogError::new("invalid_name", "Name cannot be empty."));
    }

    if trimmed == "." || trimmed == ".." {
        return Err(CatalogError::new(
            "invalid_name",
            "Name cannot be \".\" or \"..\".",
        ));
    }

    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err(CatalogError::new(
            "invalid_name",
            "Name cannot contain path separators.",
        ));
    }

    if trimmed.contains('\0') {
        return Err(CatalogError::new(
            "invalid_name",
            "Name cannot contain null characters.",
        ));
    }

    // Reject multi-component names that would resolve differently after Path joining.
    let as_path = std::path::Path::new(trimmed);
    if as_path.components().count() != 1 {
        return Err(CatalogError::new(
            "invalid_name",
            "Name must be a single path component.",
        ));
    }

    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_names() {
        assert_eq!(
            validate_single_path_component_name("  Photart Catalog  ").unwrap(),
            "Photart Catalog"
        );
        assert_eq!(
            validate_single_path_component_name("Shoot-01").unwrap(),
            "Shoot-01"
        );
    }

    #[test]
    fn rejects_empty_dot_and_separators() {
        assert_eq!(
            validate_single_path_component_name("   ").unwrap_err().code,
            "invalid_name"
        );
        assert_eq!(
            validate_single_path_component_name(".").unwrap_err().code,
            "invalid_name"
        );
        assert_eq!(
            validate_single_path_component_name("..").unwrap_err().code,
            "invalid_name"
        );
        assert_eq!(
            validate_single_path_component_name("a/b").unwrap_err().code,
            "invalid_name"
        );
        assert_eq!(
            validate_single_path_component_name("a\\b")
                .unwrap_err()
                .code,
            "invalid_name"
        );
    }
}
