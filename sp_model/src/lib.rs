use sp_domain::*;
use serde::{Deserialize, Serialize};

// For derive macro.
pub use sp_model_derive::Resource;
pub trait Resource {
    fn new(name: &str) -> Self;
    fn get_variables(&self) -> Vec<Variable>;
    fn get_input_mapping(&self) -> Vec<(SPPath, SPPath)>;
    fn get_output_mapping(&self) -> Vec<(SPPath, SPPath)>;

    fn setup_inputs(&self, topic: &str) {

        println!("{topic}: {:?}", self.get_input_mapping());
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

/// A transition in the context of a model is made up of potentially
/// several basic transitions. E.g. one transition for planning and
/// additional runner transitions.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ModelTransition {
    pub transitions: Vec<(Transition, TransitionType)>
}

pub fn operation(path: SPPath,
                 mut pre: Predicate,
                 mut actions: Vec<Action>,
                 mut post: Predicate,
                 mut post_actions: Vec<Action>) -> Vec<ModelTransition> {
    pre = p!([path == "i"] && [pre]);
    actions.push(a!(path = "e"));

    post = p!([path == "e"] && [post]);
    // TODO: reset transitions etc.
    post_actions.push(a!(path = "i"));

    vec![
        ModelTransition {
            transitions: vec![
                (Transition::new(path.add_child("op_start".into()), pre, actions), TransitionType::Runner),
                (Transition::new(path.add_child("op_finish".into()), post, post_actions), TransitionType::Runner)
            ]
        }
    ]
}
