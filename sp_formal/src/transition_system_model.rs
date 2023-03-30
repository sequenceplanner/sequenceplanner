/// This module contain a simple model type that we can use for the
/// formal verification stuff.
use serde::{Deserialize, Serialize};
use sp_domain::*;
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct TransitionSystemModel {
    pub name: String,
    pub vars: Vec<Variable>,
    pub state_predicates: Vec<NamedPredicate>,
    pub transitions: Vec<Transition>,
    pub invariants: Vec<NamedPredicate>,

    /// TS model currently "compiled" against this state id.
    pub state_id: Uuid,
}

impl TransitionSystemModel {
    pub fn bad_state(&self, state: &SPState) -> bool {
        if self.state_id != state.id() {
            panic!("must update state id");
        }
        self.invariants.iter().any(|s| !s.predicate.eval(state))
    }
}
