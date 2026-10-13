use std::hash::{DefaultHasher, Hash, Hasher};

use super::keys;

pub fn folder_tag(folder: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    folder.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExclusionScope {
    Empty,
    Everywhere,
    Inside(String),
    Outside,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderKind {
    Source,
    Excluded,
}

impl FolderKind {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Source => keys::library::FOLDERS,
            Self::Excluded => keys::library::EXCLUDED_FOLDERS,
        }
    }
}
