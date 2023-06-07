use sp_domain::*;
use sp_model::*;
use sp_runner::*;


#[tokio::test]
async fn launch_dorna_model() {
    #[derive(Resource)]
    struct Goal {
        #[Variable(type = "bool", initial = false)]
        pub on: Variable,
    }

    #[derive(Resource)]
    struct Action {
        #[Variable(type = "bool", initial = false)]
        pub call: Variable,
        #[Variable(type = "string", domain = "ok requesting accepted rejected succeeded aborted requesting_cancel cancelling cancel_rejected timeout")]
        pub status: Variable,

        #[Resource]
        goal: Goal,
        // #[Resource]
        // feedback: Feedback,
        // #[Resource]
        // result: Result,
    }

    #[derive(Resource)]
    struct BlueLight {
        #[Variable(type = "bool", initial = false)]
        #[Input]
        pub blue_light_on: Variable,

        #[Resource]
        pub action: Action,
    }

    #[derive(Resource)]
    struct Model {
        #[Resource]
        pub blue_light: BlueLight,
    }

    let m = Model::new("m");
    let mut mb: ModelBuilder = ModelBuilder::from(&m);

    mb.add_message(m.blue_light.setup_inputs("control_box/measured", "control_box_msgs/msg/Measured"));

    // TODO: auto-generate these...
    let request = vec![MessageVariable {
        ros_path: "on".into(),
        path: "m.blue_light.action.goal.on".into(),
    }];

    let action_msg = Message {
        name: m.blue_light.action.status.path.clone(),
        topic: "/control_box/set_light".into(),
        category: MessageCategory::Action,
        message_type: MessageType::Ros("control_box_msgs/action/SetLight".into()),
        variables: request,
        variables_response: vec![],
        variables_feedback: vec![],
        send_predicate: p!(m.blue_light.action.call),
    };

    mb.add_message(action_msg);

    // Add some transitions
    mb.add_transition("call_action".into(),
                      p!([m.blue_light.action.status == "ok"] && [m.blue_light.blue_light_on == m.blue_light.action.goal.on]),
                      vec![a!(m.blue_light.action.goal.on = !m.blue_light.action.goal.on),
                           a!(m.blue_light.action.call)
                      ]);
    mb.add_transition("reset_action".into(),
                      p!([m.blue_light.action.status == "succeeded"]),
                      vec![a!(!m.blue_light.action.call)]);


    // Launch and run for two seconds.
    let rm = RunnerModel::from(mb);
    let r = launch_model(rm);
    let t = tokio::time::timeout(std::time::Duration::from_millis(5000), r).await;
    println!("Timeout: {:?}", t);
}
