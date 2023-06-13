use sp_domain::*;
use sp_model::*;
use sp_ros::*;
use std::time::Duration;
use std::sync::{Arc, Mutex};

pub struct RunnerModel {
    /// Initial runnner state
    pub initial_state: SPState,

    /// Resource communication defintions.
    pub messages: Vec<Message>,

    /// Transitions
    pub transitions: Vec<Transition>,

    /// Async transitions
    pub async_transitions: Vec<crate::AsyncTransition>,
}

impl RunnerModel {
    pub fn from(model: ModelBuilder) -> Self {
        RunnerModel {
            initial_state: model.get_initial_state(),
            messages: model.messages,
            transitions: model.transitions,
            async_transitions: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SPRunnerInput {
    Tick,
    StateChange(SPState),
    NewPlan(Vec<SPPath>),
}

pub async fn launch_model(mut runner_model: RunnerModel) -> Result<(), SPError> {
    log_info!("startar SP!");

    let (tx_runner, rx_runner) = tokio::sync::mpsc::channel(2);
    let (tx_new_state, rx_new_state) = tokio::sync::mpsc::channel(2);
    let (tx_runner_state, rx_runner_state) = tokio::sync::watch::channel(runner_model.initial_state.clone());


    tokio::spawn(merger(rx_new_state, tx_runner.clone()));
    tokio::spawn(ticker_async(std::time::Duration::from_millis(1000), tx_runner.clone()));

    let _ros_comm = sp_ros::RosComm::new(
        rx_runner_state.clone(),
        tx_new_state.clone(),
        &runner_model.messages
    ).await?;


    let tx_runner_task = tx_runner.clone();
    let runner_handle = tokio::spawn(async move {
        runner(
            &mut runner_model,
            tx_runner_task,
            rx_runner,
            tx_runner_state,
        ).await;
    });

    let err = runner_handle.await;

    // Will never get here
    println!("The runner terminated!: {:?}", err);
    log_error!("The SP runner terminated: {:?}", err);

    Ok(())

}


async fn runner(
    model: &mut RunnerModel,
    tx_input: tokio::sync::mpsc::Sender<SPRunnerInput>,
    mut rx_input: tokio::sync::mpsc::Receiver<SPRunnerInput>,
    tx_state_out: tokio::sync::watch::Sender<SPState>
) {
    log_info!("Runner start");

    let mut ticker = crate::Ticker {
        state: model.initial_state.clone(),
        runner_tx: tx_input.clone(),
        controlled_transitions: vec![],
        uncontrolled_transitions: model.transitions.clone(),
        predicates: vec![],
        async_transitions: vec![],
        active_async_transitions: vec![],
        controlled_queue: vec![],
    };

    // move async transitions out of runner model.
    std::mem::swap(&mut ticker.async_transitions, &mut model.async_transitions);

    loop {
        log_info!("Looping");
        let mut state_has_probably_changed = false;
        let mut ticked = false;
        let mut last_fired_transitions = vec![];
        let input = rx_input.recv().await;
        if let Some(input) = input {
            match input {
                SPRunnerInput::StateChange(s) => {
                    if !ticker.state.are_new_values_the_same(&s) {
                        let state_id = ticker.state.id();
                        ticker.state.extend(s);
                        if state_id != ticker.state.id() {
                            // changed_variables have added a new variable
                            ticker.update_state_paths();
                        }
                        last_fired_transitions = ticker.tick_transitions();
                        state_has_probably_changed = true;
                    } else {
                        ticker.update_state_paths();
                    }
                },
                SPRunnerInput::Tick => {
                    log_info!("Ticked");
                    last_fired_transitions = ticker.tick_transitions();
                    ticked = true;
                },
                SPRunnerInput::NewPlan(_plan) => {
                    // runner.set_plan(plan_name, plan);
                },
            }

            // if there's nothing to do in this cycle, continue
            if !state_has_probably_changed && last_fired_transitions.is_empty() && !ticked {
                continue;
            } else {
                // println!("state changed? {}", state_has_probably_changed);
                // println!("transition fired? {}", !last_fired_transitions.is_empty());
                // println!("ticked? {}", ticked);
            }

            println!("{:?}", last_fired_transitions);
            println!("{}", ticker.state);

            let _res = tx_state_out.send(ticker.state.clone());
        }

    }
}


struct MergedState {
    pub states: Vec<SPState>,
}
impl MergedState {
    pub fn new() -> MergedState {
        MergedState{states: vec!()}
    }
}

/// Merging states if many states arrives at the same time
async fn merger(
    mut rx_mess: tokio::sync::mpsc::Receiver<SPState>,
    tx_runner: tokio::sync::mpsc::Sender<SPRunnerInput>,
) {
    let (tx, mut rx) = tokio::sync::watch::channel(false);
    let ms_arc = Arc::new(Mutex::new(MergedState::new()));

    let ms_in = ms_arc.clone();
    tokio::spawn(async move {
        loop {
            let s = rx_mess.recv().await.expect("The state channel should always work!");
            {
                ms_in.lock().unwrap().states.push(s);
            }
            tx.send(true).expect("internal channel in merge should always work!");
        }
    });

    let ms_out = ms_arc.clone();
    tokio::spawn(async move {
        loop {
            let _result = rx.changed().await;
            let mut states = {
                let mut x = ms_out.lock().unwrap();
                let res = x.states.clone();
                x.states = vec!();
                res
            };
            states.reverse();
            if !states.is_empty() {
                let mut x = states.pop().unwrap();
                for y in states {
                    if let Some(other) =  try_extend(&mut x, y) {
                        // Can not be merged so sending what we have
                        let _res = tx_runner.send(SPRunnerInput::StateChange(x.clone())).await;
                        x = other;
                    }
                }
                let _res = tx_runner.send(SPRunnerInput::StateChange(x)).await;
            }
        }
    });


}

/// Tries to extend the state only if the state does not contain the same
/// path or if that path has the same value, else will leave the state unchanged
/// and returns false.
fn try_extend(state: &mut SPState, other_state: SPState) -> Option<SPState> {
    let can_extend = other_state.projection().state.iter().all(|(p, v)| {
        let self_v = state.sp_value_from_path(p);
        p.leaf() == "timestamp" || self_v.map(|x| x == v.value()).unwrap_or(true)
    });
    if can_extend {
        state.extend(other_state);
        None
    } else {
        Some(other_state)
    }
}

/// The ticker that sends a tick to the runner at an interval defined by ´freq´
async fn ticker_async(period: Duration, tx_runner: tokio::sync::mpsc::Sender<SPRunnerInput>) {
    let mut ticker = tokio::time::interval(period);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        ticker.tick().await;
        let _res = tx_runner.send(SPRunnerInput::Tick).await;
    }
}
