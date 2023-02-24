use sp_domain::*;

pub use sp_model_derive::Resource;

pub trait Resource {
    fn new(name: &str) -> Self;
    fn get_variables(&self) -> Vec<Variable>;
}
