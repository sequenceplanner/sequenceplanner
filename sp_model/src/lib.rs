use sp_domain::*;
use serde::{Deserialize, Serialize};

// For derive macro.
pub use sp_model_derive::Resource;
pub trait Resource {
    fn new(name: &str) -> Self;
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
pub struct ModelBuilder {
    pub variables: Vec<Variable>,
    pub transitions: Vec<Transition>,

    pub messages: Vec<Message>,
}


impl ModelBuilder {
    pub fn from(model: &impl Resource) -> Self {
        let mut mb = ModelBuilder {
            variables: vec![],
            transitions: vec![],
            messages: vec![],
        };
        mb.variables.extend(model.get_variables());
        mb
    }

    pub fn get_initial_state(&self) -> SPState {
        SPState::new_from_variables(&self.variables)
    }

    pub fn add_message(&mut self, m: Message) {
        self.messages.push(m);
    }

    /// Operations can abstract away implementation details ql  o;jqjs from the planner.
    /// By defaut, only i -> e -> i are included in the formal representation.
    pub fn add_operation(&mut self,
                         path: SPPath,
                         pre: Predicate,
                         mut actions: Vec<Action>,
                         post: Predicate,
                         mut post_actions: Vec<Action>) -> SPPath {
        let mut var = Variable::new(path.clone(), SPValueType::String,
                                    vec!["i".to_spvalue(),
                                         "e".to_spvalue(),
                                         "f".to_spvalue()]);
        var.initial_state = "i".to_spvalue();
        let pre = p!([path == "i"] && [pre]);
        actions.push(a!(path = "e"));

        let post = p!([path == "e"] && [post]);
        post_actions.push(a!(path = "f"));

        let trans = vec![
            Transition::new(path.add_child("start".into()), pre, actions),
            Transition::new(path.add_child("finish".into()), post, post_actions),
        ];

        let path = var.path.clone();
        self.variables.push(var);
        self.transitions.extend(trans);
        path
    }

    pub fn add_transition(&mut self,
                          path: SPPath,
                          guard: Predicate,
                          actions: Vec<Action>) {
        self.transitions.push(Transition { path, guard, actions });
    }


}
