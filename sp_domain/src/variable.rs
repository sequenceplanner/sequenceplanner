use super::*;

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct Variable {
    pub path: SPPath,
    pub value_type: SPValueType,
    pub domain: Vec<SPValue>,
}

impl Variable {
    pub fn new(name: &str, value_type: SPValueType, domain: Vec<SPValue>) -> Variable {
        let path = SPPath::from_string(name);
        Variable {
            path,
            value_type,
            domain,
        }
    }
    pub fn new_boolean(name: &str) -> Variable {
        Variable::new(
            name,
            SPValueType::Bool,
            vec![false.to_spvalue(), true.to_spvalue()],
        )
    }

    pub fn value_type(&self) -> SPValueType {
        self.value_type
    }
    pub fn domain(&self) -> &[SPValue] {
        self.domain.as_slice()
    }

    pub fn path(&self) -> &SPPath {
        &self.path
    }
}
