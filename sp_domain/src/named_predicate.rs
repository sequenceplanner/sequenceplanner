use super::*;

/// Simply a predicate with a name (path) attached.

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct NamedPredicate {
    pub path: SPPath,
    pub predicate: Predicate,
}

impl NamedPredicate {
    pub fn new(name: &str, predicate: Predicate) -> Self {
        let path = SPPath::from_string(name);
        Self { path, predicate }
    }
}
