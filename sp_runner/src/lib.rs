mod ticker;
pub use ticker::*;

mod runner;
pub use runner::*;


#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
