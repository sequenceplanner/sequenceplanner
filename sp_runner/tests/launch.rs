use sp_domain::*;
use sp_model::*;
use sp_runner::*;

#[tokio::test]
async fn launch_empty_model() {
    #[derive(Resource)]
    struct Inner {
        #[Variable(type = "String", initial = "hej", domain = "hej svejs")]
        #[Input(mapping = "data")]
        pub input: Variable,
    }

    #[derive(Resource)]
    struct Model {
        #[Variable(type = "String", initial = "hej", domain = "hej svejs")]
        #[Input(mapping = "data")]
        pub input: Variable,

        #[Variable(type = "String", initial = "hej", domain = "hej svejs")]
        #[Output(mapping = "data")]
        pub output: Variable,

        #[Resource]
        pub inner: Inner,
    }

    let m = Model::new("m");
    let mut mb: ModelBuilder = ModelBuilder::from(&m);

    mb.add_message(m.setup_outputs("output_topic", "std_msgs/msg/String"));
    let inputs = m.setup_inputs("input_topic", "std_msgs/msg/String");
    println!("{:?}", inputs);
    mb.add_message(inputs);
    let inputs = m.inner.setup_inputs("input_topic2", "std_msgs/msg/String");
    println!("{:?}", inputs);
    mb.add_message(inputs);

    let rm = RunnerModel::from(mb);
    let r = launch_model(rm).await;



    println!("{:?}", r);
}
