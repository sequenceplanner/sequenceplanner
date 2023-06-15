use r2r;
use std::sync::Arc;
use tokio::sync::Mutex;

use sp_domain::*;
use sp_model::*;
use sp_runner::*;


mod gripper {
    use super::*;

    #[derive(Resource)]
    pub struct OpenService {
        #[Variable(type = "bool", initial = false)]
        pub call: Variable,
    }

    #[derive(Resource)]
    pub struct CloseService {
        #[Variable(type = "bool", initial = false)]
        pub call: Variable,

        #[Variable(type = "bool", initial = false)]
        pub has_part: Variable,
    }

    #[derive(Resource)]
    pub struct Gripper {
        #[Resource]
        pub open: OpenService,

        #[Resource]
        pub close: CloseService,
    }
}

mod gripper_async {
    use super::*;

    #[derive(Resource, Clone)]
    pub struct Gripper {
        #[Variable(type = "bool", initial = false)]
        pub opened: Variable,

        #[Variable(type = "bool", initial = false)]
        pub opening: Variable,

        #[Variable(type = "bool", initial = false)]
        pub part_sensor: Variable,
    }

    impl Gripper {
        pub fn make_model(self, _mb: &mut ModelBuilder, node: &mut r2r::Node) {
            use r2r::gripper_msgs::srv::Open;
            let client = Arc::new(Mutex::new(node.create_client::<Open::Service>("/gripper/open").expect("could not create ros client")));

            let async_action: AsyncActionFunction = Box::new(move |state| {
                let self_clone = self.clone();
                let _cloned_state = state.clone();
                let cloned_client = client.clone();
                // Set "opening" state.
                let pre_state = SPState::new_from_values(&[(self_clone.opening.path.clone(), true.to_spvalue())]);
                (pre_state, Box::pin(async move {
                    let cl = cloned_client.lock().await;
                    let result = cl.request(&Open::Request { }).expect("could not request").await?;
                    let state_update = SPState::new_from_values(&[
                        (self_clone.opening.path.clone(), false.to_spvalue()),
                        (self_clone.opened.path.clone(), true.to_spvalue())
                    ]);
                    Ok(state_update)
                }))
            });
            let transition = AsyncTransition::new("t1".into(), Predicate::TRUE, async_action);
            // mb.add_async_transition(transition)
        }
    }
}

mod control_box {
    use super::*;

    #[derive(Resource)]
    pub struct Goal {
        #[Variable(type = "bool", initial = false)]
        pub on: Variable,
    }

    #[derive(Resource)]
    pub struct SetLightAction {
        #[Variable(type = "bool", initial = false)]
        pub call: Variable,
        #[Variable(type = "string")]
        pub status: Variable,

        #[Resource]
        pub goal: Goal,
        // #[Resource]
        // feedback: Feedback,
        // #[Resource]
        // result: Result,
    }

    #[derive(Resource)]
    pub struct ControlBox {
        #[Variable(type = "bool", initial = false)]
        #[Input]
        pub blue_light_on: Variable,

        #[Resource]
        pub set_light_action: SetLightAction,
    }
}


#[tokio::test]
async fn launch_dorna_model() {


    #[derive(Resource)]
    struct Model {
        #[Resource]
        pub control_box: control_box::ControlBox,

        #[Resource]
        pub gripper: gripper_async::Gripper,
    }

    let m = Model::new("m");
    let mut mb: ModelBuilder = ModelBuilder::from(&m);

    mb.add_message(m.control_box.setup_inputs("control_box/measured", "control_box_msgs/msg/Measured"));

    // TODO: auto-generate these...
    let request = vec![MessageVariable {
        ros_path: "on".into(),
        path: "m.control_box.set_light_action.goal.on".into(),
    }];

    let action_msg = Message {
        name: m.control_box.set_light_action.status.path.clone(),
        topic: "/control_box/set_light".into(),
        category: MessageCategory::Action,
        message_type: MessageType::Ros("control_box_msgs/action/SetLight".into()),
        variables: request,
        variables_response: vec![],
        variables_feedback: vec![],
        send_predicate: p!(m.control_box.set_light_action.call),
    };

    mb.add_message(action_msg);

    // Add some transitions
    mb.add_transition("call_action".into(),
                      p!([m.control_box.set_light_action.status == "ok"] &&
                         [m.control_box.blue_light_on == m.control_box.set_light_action.goal.on]),
                      vec![a!(m.control_box.set_light_action.goal.on = !m.control_box.set_light_action.goal.on),
                           a!(m.control_box.set_light_action.call)
                      ]);
    mb.add_transition("reset_action".into(),
                      p!([m.control_box.set_light_action.status == "succeeded"]),
                      vec![a!(!m.control_box.set_light_action.call)]);


    // Start a new ros node just because we can. (this way we can easily speak mqtt aswell.)
    let ctx = r2r::Context::create().expect("could not start ros");
    let mut node = r2r::Node::create(ctx, "testnode", "").expect("could not create ros node");

    m.gripper.make_model(&mut mb, &mut node);

    use r2r::example_interfaces::srv::AddTwoInts;
    let client = Arc::new(Mutex::new(node.create_client::<AddTwoInts::Service>("/add_two_ints").expect("could not create ros client")));
    let _ros_handle = tokio::task::spawn_blocking(move || loop {
        node.spin_once(std::time::Duration::from_millis(100));
    });

    // Launch and run for a few seconds.
    let mut rm = RunnerModel::from(mb);

    let async_action: AsyncActionFunction = Box::new(move |state| {
        let _cloned_state = state.clone();
        let cloned_client = client.clone();
        let mut value = state.sp_value_from_path(&"test".into()).cloned().unwrap_or(0.to_spvalue());

        // Unsure if this should be a proper action.
        let pre_state = SPState::new_from_values(&[("in_progress".into(), true.to_spvalue())]);
        (pre_state, Box::pin(async move {
            let int_value: i32 = if let SPValue::Int32(n) = &value { *n } else { 0 };
            let req = AddTwoInts::Request { a: int_value as i64, b: 1 };
            let mut sum = 0;
            let cl = cloned_client.lock().await;
            if let Ok(resp) = cl.request(&req).expect("could not request").await {
                println!("Got result here, sleeping 1 sec. {}", resp.sum);
                sum = resp.sum;
            }

            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            if let SPValue::Int32(n) = &mut value {
                *n= sum as i32;
            }
            println!("Done sleeping, expecting state update now");
            let state_update = SPState::new_from_values(&[
                ("test".into(), value),
                ("in_progress".into(), false.to_spvalue())
            ]);
            Ok(state_update)
        }))
    });

    let in_progress = SPPath::from("in_progress");
    let transition = AsyncTransition::new("t1".into(), p!(!in_progress), async_action);

    rm.async_transitions.push(transition);

    let r = launch_model(rm);
    let t = tokio::time::timeout(std::time::Duration::from_millis(10000), r).await;
    println!("Timeout: {:?}", t);
}
