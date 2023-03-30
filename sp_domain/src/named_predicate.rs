use super::*;

/// Simply a predicate with a name (path) attached.

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct NamedPredicate {
    pub path: SPPath,
    pub predicate: Predicate,
    pub state_path: Option<StatePath>,
}

impl NamedPredicate {
    pub fn new(path: SPPath, predicate: Predicate) -> Self {
        Self {
            path,
            predicate,
            state_path: None,
        }
    }

    pub fn upd_state_path(&mut self, state: &SPState) {
        if let Some(sp) = state.state_path(&self.path) {
            self.state_path = Some(sp);
        } else {
            eprintln!("WARNING Could not update statepath");
        }
    }
}
