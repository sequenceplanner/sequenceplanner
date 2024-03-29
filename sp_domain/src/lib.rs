//! The SP domain

pub mod value;
pub use value::*;

pub mod predicate;
pub use predicate::*;

pub mod predicate_parser;

pub mod state;
pub use state::*;

pub mod path;
pub use path::*;

pub mod variable;
pub use variable::*;

pub mod named_predicate;
pub use named_predicate::*;

pub mod transition;
pub use transition::*;

use serde::{Deserialize, Serialize};
use std::error;
use std::fmt;
use std::fmt::Display;

type SPResult<T> = std::result::Result<T, SPError>;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum SPError {
    No(String),
    Undefined,
}

impl std::convert::From<serde_json::Error> for SPError {
    fn from(e: serde_json::Error) -> Self {
        SPError::from_any(e)
    }
}

impl SPError {
    pub fn from_any<T: Display>(x: T) -> SPError {
        SPError::No(format!("{x}"))
    }
}

impl fmt::Display for SPError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SPError::Undefined => write!(f, "An undefined SP error!"),
            SPError::No(s) => write!(f, "Oh No: {s}"),
        }
    }
}

impl error::Error for SPError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests_domain {}
