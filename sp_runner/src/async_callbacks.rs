use sp_domain::*;

mod ticker;
pub use ticker::*;

mod runner;
pub use runner::*;


// Old code could not do closures but easier to use because wrapping in pin box could
// be done automatically. Not used now but keeping for reference.
use std::boxed::Box;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

trait AsyncCallback<'a, U, T> {
    type Output: std::future::Future<Output=T> + 'a + Send;
    fn call(self, arg: &'a U) -> Self::Output;
}

impl<'a, Fut: 'a, F, U: 'a, T> AsyncCallback<'a, U, T> for F
where
    F: FnOnce(&'a U) -> Fut,
    Fut: std::future::Future<Output=T> + Send,
{
    type Output = Fut;
    fn call(self, arg: &'a U) -> Fut {
        self(arg)
    }
}

type AsyncTaskFunction<U,T> = Box<dyn for<'r> Fn(&'r U) -> Pin<Box<dyn Future<Output = T> + Send + 'r>> + Sync + Send>;

fn from_async_function<F, U, T>(cb: F) -> AsyncTaskFunction<U, T>
where
    for<'a> F: AsyncCallback<'a, U, T> + Clone + 'static + Send + Sync,
{
        Box::new(move |s| Box::pin(cb.clone().call(s)))
}

#[derive(Debug, Clone)]
struct MyStruct {
    a: usize,
    b: usize,
    c: String,
    // ...
}

type GooseTaskFunction = AsyncTaskFunction<MyStruct, MyStruct>;

struct StructWithCallback {
    callback: GooseTaskFunction,
}

impl StructWithCallback {

    fn new(cb: GooseTaskFunction) -> Self
    {
        Self {
            callback: cb,
        }
    }
}

async fn my_function(my_struct: &MyStruct) -> MyStruct {
    println!("in my_function: {:?}", my_struct);
    let mut ms = my_struct.clone();
    ms.a = 5;
    return ms;
}

async fn example() {
    let wrapped_function = from_async_function(my_function);
    let struct_with_callback = StructWithCallback::new(wrapped_function);

    let my_struct = MyStruct {
        a: 1,
        b: 2,
        c: "a string".to_string(),
    };

    // Invoke the stored callback.
    //let callback = .clone();
    //struct_with_callback.execute(&my_struct).await;
    let my_struct = (struct_with_callback.callback)(&my_struct).await;
    println!("after my_function: {:?}", my_struct);
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
