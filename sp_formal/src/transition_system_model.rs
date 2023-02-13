/// This module contain a simple model type that we can use for the
/// formal verification stuff.
use serde::{Deserialize, Serialize};
use sp_domain::*;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct TransitionSystemModel {
    pub name: String,
    pub vars: Vec<Variable>,
    pub state_predicates: Vec<NamedPredicate>,
    pub transitions: Vec<Transition>,
    pub invariants: Vec<NamedPredicate>,
}
