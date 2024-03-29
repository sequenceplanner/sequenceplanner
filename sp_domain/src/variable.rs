use super::*;

/// A variable with a type and an optional domain

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct Variable {
    pub path: SPPath,
    pub value_type: SPValueType,
    pub domain: Vec<SPValue>,
    pub initial_state: SPValue,
}

impl ToPredicateValue for Variable {
    fn to_predicate_value(&self) -> PredicateValue {
        PredicateValue::SPPath(self.path.clone(), None)
    }
}

impl ToPredicate for Variable {
    fn to_predicate(&self) -> Predicate {
        Predicate::EQ(self.to_predicate_value(), true.to_predicate_value())
    }
}

impl Variable {
    pub fn new(path: SPPath, value_type: SPValueType, domain: Vec<SPValue>) -> Self {
        Self {
            path,
            value_type,
            domain,
            initial_state: SPValue::Unknown,
        }
    }

    pub fn new_boolean(path: SPPath) -> Self {
        Self {
            path,
            value_type: SPValueType::Bool,
            domain: vec![false.to_spvalue(), true.to_spvalue()],
            initial_state: SPValue::Unknown,
        }
    }
}
