use super::*;

/// A variable with a type and an optional domain

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct Variable {
    pub path: SPPath,
    pub value_type: SPValueType,
    pub domain: Vec<SPValue>,
}

impl Variable {
    pub fn new(name: &str, value_type: SPValueType, domain: Vec<SPValue>) -> Self {
        let path = SPPath::from_string(name);
        Self {
            path,
            value_type,
            domain,
        }
    }

    pub fn new_boolean(name: &str) -> Self {
        let path = SPPath::from_string(name);
        Self {
            path,
            value_type: SPValueType::Bool,
            domain: vec![false.to_spvalue(), true.to_spvalue()],
        }
    }
}
