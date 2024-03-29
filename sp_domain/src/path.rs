//! The SPPath is used for identifying items in a model.

use super::*;
use serde::{Deserialize, Serialize};

const PATH_SEP: &str = ".";

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Clone, Default, Debug)]
pub struct SPPath {
    pub path: Vec<String>,
}

impl std::fmt::Display for SPPath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.path.join(PATH_SEP))
    }
}

impl From<&str> for SPPath {
    fn from(s: &str) -> Self {
        let path: Vec<String> = s
            .trim_start_matches(PATH_SEP)
            .trim_end_matches(PATH_SEP)
            .split(PATH_SEP)
            .map(|x|x.to_string())
            .collect();
        Self { path }
    }
}

impl From<String> for SPPath {
    fn from(s: String) -> Self {
        let path: Vec<String> = s
            .trim_start_matches(PATH_SEP)
            .trim_end_matches(PATH_SEP)
            .split(PATH_SEP)
            .map(|x|x.to_string())
            .collect();
        Self { path }
    }
}

impl<T: Sized + AsRef<str>, const N: usize> From<&[T; N]> for SPPath {
    fn from(s: &[T; N]) -> Self {
        let path: Vec<String> = s.iter().map(|s| s.as_ref().to_string()).collect();
        Self { path }
    }
}

impl<T: AsRef<str>> From<&[T]> for SPPath {
    fn from(s: &[T]) -> Self {
        let path: Vec<String> = s.iter().map(|s| s.as_ref().to_string()).collect();
        Self { path }
    }
}

impl From<Vec<String>> for SPPath {
    fn from(path: Vec<String>) -> Self {
        Self { path }
    }
}

impl From<&Variable> for SPPath {
    fn from(v: &Variable) -> Self {
        v.path.clone()
    }
}

impl ToPredicateValue for SPPath {
    fn to_predicate_value(&self) -> PredicateValue {
        PredicateValue::SPPath(self.clone(), None)
    }
}

impl ToPredicate for SPPath {
    fn to_predicate(&self) -> Predicate {
        Predicate::EQ(self.to_predicate_value(), true.to_predicate_value())
    }
}

impl SPPath {
    pub fn new() -> SPPath {
        SPPath { path: vec![] }
    }
    pub fn add_child(&self, sub: &str) -> Self {
        let mut p = self.path.clone();
        p.push(sub.to_string());
        SPPath::from(p)
    }
    pub fn add_child_mut(&mut self, sub: &str) {
        self.path.push(sub.to_string());
    }
    pub fn add_parent(&self, root: &str) -> Self {
        let mut p = self.path.clone();
        p.insert(0, root.to_string());
        SPPath::from(p)
    }
    pub fn add_parent_mut(&mut self, root: &str) {
        self.path.insert(0, root.to_string());
    }
    pub fn add_child_path_mut(&mut self, sub: &SPPath) {
        self.path.append(&mut sub.path.clone())
    }
    pub fn add_parent_path_mut(&mut self, root: &SPPath) -> SPPath {
        let mut new_path = root.path.clone();
        new_path.append(&mut self.path);
        self.path = new_path;
        self.clone()
    }
    pub fn add_child_path(&self, sub: &SPPath) -> SPPath {
        let mut p = self.path.clone();
        p.append(&mut sub.path.clone());
        SPPath::from(p)
    }
    pub fn add_parent_path(&self, root: &SPPath) -> SPPath {
        let mut new_path = root.path.clone();
        new_path.append(&mut self.path.clone());
        SPPath::from(new_path)
    }

    pub fn drop_parent(&mut self, parent: &SPPath) -> SPResult<()> {
        let zipped = self.path.iter().zip(parent.path.iter());
        let match_len = zipped.filter(|(a, b)| a == b).count();
        if match_len == parent.path.len() {
            self.path.drain(0..match_len);
            Ok(())
        } else {
            Err(SPError::No(format!(
                "cannot drop parent as it does not exist: {self} - {parent}"
            )))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    pub fn is_child_of(&self, other: &SPPath) -> bool {
        (self.path.len() >= other.path.len())
            && other.path.iter().zip(self.path.iter()).all(|(a, b)| a == b)
    }

    pub fn is_child_of_any(&self, others: &[SPPath]) -> bool {
        others.iter().any(|o| self.is_child_of(o))
    }

    pub fn root(&self) -> String {
        self.path.first().unwrap_or(&String::default()).clone()
    }

    pub fn parent(&self) -> SPPath {
        if self.path.len() <= 1 {
            SPPath::new()
        } else {
            SPPath::from(&self.path[..self.path.len() - 1])
        }
    }

    pub fn drop_root(&self) -> SPPath {
        if self.path.is_empty() {
            SPPath::new()
        } else {
            SPPath::from(&self.path[1..])
        }
    }

    pub fn leaf(&self) -> String {
        if self.path.is_empty() {
            "".to_string()
        } else {
            self.path[self.path.len() - 1].clone()
        }
    }

    pub fn leaf_as_path(&self) -> SPPath {
        let leaf = if self.path.is_empty() {
            "".to_string()
        } else {
            self.path[self.path.len() - 1].clone()
        };
        SPPath { path: vec![leaf] }
    }

    pub fn drop_leaf(&mut self) -> String {
        if !self.path.is_empty() {
            self.path.remove(self.path.len() - 1)
        } else {
            String::new()
        }
    }

    /// returns the next name in the path of this SPPath based on a path
    /// that is the current parent to this path
    pub fn next_node_in_path(&self, parent_path: &SPPath) -> Option<String> {
        if self.is_child_of(parent_path) && self.path.len() > parent_path.path.len() {
            Some(self.path[parent_path.path.len()].clone())
        } else {
            None
        }
    }
}


#[cfg(test)]
mod tests_paths {
    use super::*;
    #[test]
    fn making() {
        let ab = SPPath::from(&["a", "b"]);
        let ab_v2 = SPPath::new();
        let ab_v2 = ab_v2.add_child("a").add_child("b");
        let ab_v3 = SPPath::from("a.b");

        assert_eq!(ab.to_string(), "a.b".to_string());
        assert_eq!(ab_v2, ab);
        assert_eq!(ab_v3, ab);
        assert_ne!(SPPath::from("b.a"), ab);
        assert_ne!(SPPath::from(&["b", "a"]), ab);
        assert_ne!(SPPath::from(&["a", "b", "c"]), ab);
    }

    #[test]
    fn drop_parent() {
        let mut ab = SPPath::from(&["a", "b", "c"]);
        let parent = SPPath::from(&["a", "b"]);
        ab.drop_parent(&parent).unwrap();
        assert_eq!(ab, SPPath::from(&["c"]));

        let mut ab = SPPath::from(&["a", "b", "c"]);
        let parent = SPPath::from(&["a", "b", "c"]);
        ab.drop_parent(&parent).unwrap();
        assert_eq!(ab, SPPath::new());

        let mut ab = SPPath::from(&["a", "b", "c"]);
        let parent = SPPath::from(&["a"]);
        ab.drop_parent(&parent).unwrap();
        assert_eq!(ab, SPPath::from(&["b", "c"]));
    }

    #[test]
    fn drop_parent_fail() {
        let mut ab = SPPath::from(&["a", "b", "c"]);
        let parent = SPPath::from(&["a", "c"]);
        let res = ab.drop_parent(&parent);
        assert!(res.is_err())
    }

    #[test]
    fn get_next_name() {
        let p = SPPath::from("a.b.c.d");
        println! {"{}", serde_json::to_string(&p).unwrap()};
    }
}
