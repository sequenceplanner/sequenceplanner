use sp_domain::*;
use serde::{Deserialize, Serialize};

// For derive macro.
pub use sp_model_derive::Resource;
pub trait Resource: Clone {
    fn new(name: &str) -> Self where Self: Sized;
    fn get_variables(&self) -> Vec<Variable>;
    fn get_input_mapping(&self) -> Vec<(SPPath, SPPath)>;
    fn get_output_mapping(&self) -> Vec<(SPPath, SPPath)>;

    fn setup_inputs(&self, topic: &str, msg_type: &str) -> Message {
        Message {
            name: topic.into(),
            topic: topic.into(),
            category: MessageCategory::Incoming,
            message_type: MessageType::Ros(msg_type.to_owned()),
            variables: self.get_input_mapping().iter().map(
                |(v1, v2)| MessageVariable::new(v1,v2)).collect(),
            variables_response: vec!(),
            variables_feedback: vec!(),
            send_predicate: Predicate::TRUE
        }
    }

    fn setup_outputs(&self, topic: &str, msg_type: &str) -> Message {
        Message {
            name: topic.into(),
            topic: topic.into(),
            category: MessageCategory::OutGoing,
            message_type: MessageType::Ros(msg_type.to_owned()),
            variables: self.get_output_mapping().iter().map(
                |(v1, v2)| MessageVariable::new(v1,v2)).collect(),
            variables_response: vec!(),
            variables_feedback: vec!(),
            send_predicate: Predicate::TRUE // TODO: FIX predicates for outgoing
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Message {
    pub name: SPPath,
    pub topic: SPPath,
    pub category: MessageCategory,
    pub message_type: MessageType,
    pub variables: Vec<MessageVariable>,
    pub variables_response: Vec<MessageVariable>,
    pub variables_feedback: Vec<MessageVariable>,
    pub send_predicate: Predicate,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum MessageCategory {
    OutGoing,
    Incoming,
    Service,
    Action
}
impl Default for MessageCategory {
    fn default() -> Self {
        MessageCategory::OutGoing
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Ros(String),
    JsonFlat,
    Json,
}
impl Default for MessageType {
    fn default() -> Self {
        MessageType::Json
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct MessageVariable {
    pub ros_path: SPPath,
    pub path: SPPath,
}

impl MessageVariable {
    pub fn new(path: &SPPath, ros_path: &SPPath) -> Self {
        MessageVariable {
            ros_path: ros_path.clone(),
            path: path.clone(),
        }
    }
}


#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TransitionType {
    Controlled,
    Auto,
    Effect,
    Runner,
}

impl TransitionType {
    fn is_formal(&self) -> bool {
        self != &TransitionType::Runner
    }
}

/// A transition in the context of a model is made up of potentially
/// several basic transitions. E.g. one transition for planning and
/// additional runner transitions.
/// They will execute in synchrony by the runner.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ModelTransition {
    pub transitions: Vec<(Transition, TransitionType)>
}

pub fn get_formal_transitions(mts: &[ModelTransition]) -> Vec<Transition> {
    let mut trans = vec![];
    for mt in mts {
        for (t, tt) in &mt.transitions {
            if tt.is_formal() {
                trans.push(t.clone());
            }
        }
    }
    trans
}

/// Operations can abstract away implementation details from the planner.
/// By defaut, only i -> e -> i are included in the formal representation.
pub fn operation(path: SPPath,
                 formal_pre: Predicate,
                 formal_actions: Vec<Action>,
                 runner_pre: Predicate,
                 runner_actions: Vec<Action>,
                 formal_post: Predicate,
                 formal_post_actions: Vec<Action>,
                 runner_post: Predicate,
                 runner_post_actions: Vec<Action>) -> (Variable, Vec<ModelTransition>) {
    let mut var = Variable::new(path.clone(), SPValueType::String,
                                vec!["i".to_spvalue(),
                                     "e".to_spvalue(),
                                     "f".to_spvalue()]);
    var.initial_state = "i".to_spvalue();
    let formal_pre = p!([path == "i"] && [formal_pre]);
    let mut formal_actions = formal_actions.clone();
    formal_actions.push(a!(path = "e"));

    let formal_post = p!([path == "e"] && [formal_post]);
    let mut formal_post_actions = formal_post_actions.clone();
    formal_post_actions.push(a!(path = "f"));

    let trans = vec![
        ModelTransition {
            transitions: vec![
                (Transition::new(path.add_child("formal_start".into()), formal_pre, formal_actions),
                 TransitionType::Controlled),
                (Transition::new(path.add_child("runner_start".into()), runner_pre, runner_actions),
                 TransitionType::Runner),
            ]
        },
        ModelTransition {
            transitions: vec![
                (Transition::new(path.add_child("formal_finish".into()), formal_post, formal_post_actions),
                 TransitionType::Auto),
                (Transition::new(path.add_child("runner_finish".into()), runner_post, runner_post_actions),
                 TransitionType::Runner),
            ]
        }
    ];

    (var, trans)
}
