use crate::{GemNameError, Requirement, validate_gem_name};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct ProjectDependency {
    /// What gem this dependency uses.
    pub name: String,
    /// Constraints on what version of the gem can be used.
    pub requirement: Requirement,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProjectDependencyError {
    #[error("Dependency name cannot be empty")]
    EmptyName,
    #[error("Invalid dependency name: {0}")]
    InvalidName(#[from] GemNameError),
    #[error("Invalid requirement: {0}")]
    InvalidRequirement(#[from] crate::requirement::RequirementError),
}

impl std::fmt::Debug for ProjectDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{:?}", self.name, self.requirement)
    }
}

impl std::fmt::Display for ProjectDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;

        if !self.is_latest_version() {
            write!(f, " ({})", self.requirement)?;
        }

        Ok(())
    }
}

impl ProjectDependency {
    pub fn new(name: String, requirements: Vec<String>) -> Result<Self, ProjectDependencyError> {
        let requirement = Requirement::new(requirements)?;
        Self::from_requirement(name, requirement)
    }

    pub fn from_requirement(
        name: String,
        requirement: Requirement,
    ) -> Result<Self, ProjectDependencyError> {
        if name.is_empty() {
            return Err(ProjectDependencyError::EmptyName);
        }

        validate_gem_name(&name)?;

        Ok(Self { name, requirement })
    }

    pub fn is_latest_version(&self) -> bool {
        self.requirement.is_latest_version()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_creation() {
        let dep = ProjectDependency::new("test".to_string(), vec!["~> 1.0".to_string()]).unwrap();
        assert_eq!(dep.name, "test");
        assert!(!dep.is_latest_version());
    }

    #[test]
    fn test_dependency_latest_version() {
        let dep = ProjectDependency::new("test".to_string(), vec![]).unwrap();
        assert_eq!(dep.name, "test");
        assert!(dep.is_latest_version());
    }

    #[test]
    fn test_dependency_rejects_path_like_name() {
        let result = ProjectDependency::new("../../owned".to_string(), vec![]);
        assert!(matches!(
            result.unwrap_err(),
            ProjectDependencyError::InvalidName(_)
        ));
    }
}
