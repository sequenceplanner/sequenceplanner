use super::*;

/// Simply a predicate with a name (path) attached.

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct NamedPredicate {
    pub path: SPPath,
    pub predicate: Predicate,
}

impl NamedPredicate {
    pub fn new(path: SPPath, predicate: Predicate) -> Self {
        Self {
            path,
            predicate,
        }
    }
}
