use sp_domain::*;

mod ticker;
pub use ticker::*;

mod runner;
pub use runner::*;

use std::boxed::Box;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub enum AsyncActionError {
    Other(String),
}

impl From<&str> for AsyncActionError {
    fn from(err: &str) -> AsyncActionError {
        AsyncActionError::Other(err.to_string())
    }
}

pub type AsyncActionResult = Result<SPState, AsyncActionError>;

/// The function type of a goose transaction function.
pub type AsyncActionFunction = Box<
    dyn for<'r> Fn(
            &'r SPState,
        ) -> (SPState, Pin<Box<dyn Future<Output = AsyncActionResult> + Send>>)
        + Send
        + Sync,
>;


pub struct AsyncTransition {
    pub path: SPPath,
    /// Guard
    pub guard: Predicate,
    /// A required function that is executed each time this transaction runs.
    pub function: AsyncActionFunction,
}
impl AsyncTransition {
    pub fn new(path: SPPath, guard: Predicate, function: AsyncActionFunction) -> Self {
        AsyncTransition {
            path,
            guard,
            function,
        }
    }
}


pub struct AsyncRunner {
    pub transactions: Vec<AsyncTransition>,
}

impl AsyncRunner {
    pub fn register_transaction(&mut self, transaction: AsyncTransition) {
        self.transactions.push(transaction);
    }
}

async fn test() -> AsyncActionResult {
    Err("oh no from test".into())
}

async fn example() {
    let mut runner = AsyncRunner {
        transactions: vec![],
    };

    let closure: AsyncActionFunction = Box::new(move |state| {
        let mut cloned_state = state.clone();
        (SPState::new(), Box::pin(async move {
            cloned_state.add_variable("a.b".into(), 5.to_spvalue());
            tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
            Ok(cloned_state)
        }))
    });

    let closure2: AsyncActionFunction = Box::new(move |_| {
        (SPState::new(), Box::pin(async move {
            Ok(state!(["a", "b"] => 9))
        }))
    });

    let transaction = AsyncTransition::new("t1".into(), Predicate::TRUE, closure);
    let transaction2 = AsyncTransition::new("t2".into(), Predicate::TRUE, closure2);
    // We need to do the variable dance as scenario.register_transaction returns self and hence moves
    // self out of `scenario`. By storing it in a new local variable and then moving it over
    // we can avoid that error.
    runner.register_transaction(transaction);
    runner.register_transaction(transaction2);

    // Invoke the stored callback.
    //let callback = .clone();
    //struct_with_callback.execute(&my_struct).await;
    let spstate = state!(["a", "b"] => 2, ["a", "c"] => true, ["k", "l"] => true);
    println!("before my_function: {}", spstate.sp_value_from_path(&"a.b".into()).unwrap());
    let function = &runner.transactions[0].function;
    let function2 = &runner.transactions[1].function;
    let new_state = function(&spstate).1.await.expect("function failed");
    let new_state2 = function2(&spstate).1.await.expect("function failed");
    println!("after my_function: {}", new_state.sp_value_from_path(&"a.b".into()).unwrap());
    println!("after my_function2: {}", new_state2.sp_value_from_path(&"a.b".into()).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trans_funcs() {

        example().await;

        assert_eq!(4, 4);
    }
}
