pub mod dependency;
pub mod gem_name;
pub mod platform;
pub mod project_dependency;
pub mod release_tuple;
pub mod requirement;
pub mod specification;
pub mod version_platform;

pub use dependency::{Dependency, DependencyError, DependencyType};
pub use gem_name::{GemNameError, validate_gem_name};
pub use platform::{Platform, PlatformError};
pub use project_dependency::{ProjectDependency, ProjectDependencyError};
pub use release_tuple::{ReleaseTuple, ReleaseTupleError};
pub use requirement::{ComparisonOperator, Requirement, VersionConstraint};
pub use rv_version::{Version, VersionError};
pub use specification::{Specification, SpecificationError};
pub use version_platform::{VersionPlatform, VersionPlatformError};
