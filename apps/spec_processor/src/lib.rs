//! Spec processor — coherence checks and file collection for Ash plan/spec documents.
//!
//! Walks a repository tree, classifies files by naming convention, and validates
//! plan-index files and their associated task files for internal consistency.

pub mod changelog;
pub mod collect;
pub mod finding;
pub mod plan_index;
pub mod report;
pub mod spec_links;
