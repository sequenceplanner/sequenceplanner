use sp_domain::*;
use futures::future::FutureExt;

pub struct ActiveAsyncTransition {
    path: SPPath,
    handle: tokio::task::JoinHandle<crate::AsyncActionResult>,
}

pub struct Ticker {
    pub state: SPState,

    /// To be able to tick from within the ticker (when completing async actions)
    pub runner_tx: tokio::sync::mpsc::Sender<crate::SPRunnerInput>,

    /// Controlled transitions (those that are controlled by the planner)
    pub controlled_transitions: Vec<Transition>,
    /// Runner transitions, auto transitions, effects
    pub uncontrolled_transitions: Vec<Transition>,
    pub predicates: Vec<NamedPredicate>,

    pub async_transitions: Vec<crate::AsyncTransition>,
    pub active_async_transitions: Vec<ActiveAsyncTransition>,

    /// Allowed to run
    pub controlled_queue: Vec<SPPath>,
}

impl Ticker {
    pub fn tick_transitions(&mut self) -> Vec<SPPath> {
        let mut fired = self.tick_uncontrolled();

        // check for finished async actions
        let mut state_changes = SPState::new();
        self.active_async_transitions
            .retain_mut(|ActiveAsyncTransition { path: _path, handle }| {
                let boxed = handle.boxed();
                if let Some(result) = boxed.now_or_never() {
                    // println!("Async action {} finished.", path);
                    if let Ok(Ok(state)) = result {
                        // println!("Result: {}", state);
                        state_changes.extend(state);
                    }
                    false
                } else {
                    true
                }
            });
        // apply state changes
        self.state.extend(state_changes);
        self.state.upd_preds(&self.predicates);


        for at in &self.async_transitions {
            if at.guard.eval(&self.state) {
                // only start if not running already
                if !self.active_async_transitions.iter().any(|aat| aat.path == at.path) {
                    // Spawn
                    // println!("Spawned async action {}", at.path);
                    let fut = (at.function)(&self.state); // should we return a "pre"-state with the future?
                    let runner_tx = self.runner_tx.clone();
                    let handle = tokio::spawn(async move {
                        let result = fut.await;
                        let _send_res = runner_tx.send(crate::SPRunnerInput::Tick).await;
                        result
                    }
                    );
                    self.active_async_transitions.push(ActiveAsyncTransition {
                        path: at.path.clone(),
                        handle
                    });
                }
            }
        }

        if let Some(p) = self.tick_first_controlled() {
            fired.push(p);
        }
        self.state.take_transition();
        fired
    }

    pub fn tick_uncontrolled(&mut self) -> Vec<SPPath> {
        self.state.upd_preds(&self.predicates);
        self.uncontrolled_transitions
            .iter()
            .flat_map(|t| {
                if !t.actions.is_empty() && t.eval(&self.state) {
                    // TODO: handle errors
                    let _r = t.next(&mut self.state);
                    self.state.upd_preds(&self.predicates);
                    Some(t.path.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn tick_first_controlled(&mut self) -> Option<SPPath> {
        self.state.upd_preds(&self.predicates);
        if let Some(first) = self.controlled_queue.first() {
            if let Some(first) = self.controlled_transitions.iter().find(|t|&t.path == first) {
                if first.eval(&self.state) {
                    // TODO: handle errors
                    let _r = first.next(&mut self.state);
                    self.state.upd_preds(&self.predicates);
                    let _throw_first = self.controlled_queue.pop();
                    return Some(first.path.clone());
                }
            }
        }
        None
    }

    /// After changing anything in the Ticker, run this method to update the state variables.
    pub fn update_state_paths(&mut self) {
        for x in self.controlled_transitions.iter_mut() {
            x.upd_state_path(&self.state)
        }
        for x in self.uncontrolled_transitions.iter_mut() {
            x.upd_state_path(&self.state)
        }

        // also update any new predicates with values their correct assignments
        for x in self.predicates.iter_mut() {
            x.upd_state_path(&self.state);
        }
        self.state.upd_preds(&self.predicates);
    }
}



#[cfg(test)]
mod test_new_ticker {
    use super::*;

    #[test]
    fn testing_tick() {
        let ab = SPPath::from(&["a", "b"]);
        let ac = SPPath::from(&["a", "c"]);
        let kl = SPPath::from(&["k", "l"]);
        let xy = SPPath::from(&["x", "y"]);
        let pred = SPPath::from(&["pred"]);

        let s = state!(ab => 2, ac => true, kl => 3, xy => false, pred => false);

        let a = a!(ac = false);
        let b = a!(ab = kl);

        let t1 = Transition::new("t1".into(), p!(ac), vec![a]);
        let t2 = Transition::new("t2".into(), p!(!ac), vec![b]);

        // let mut ticker = Ticker {
        //     state: s,
        //     uncontrolled_transitions: vec![t1],
        //     controlled_queue: vec![t2.path.clone()],
        //     controlled_transitions: vec![t2],
        //     .. Ticker::default()
        // };
        // ticker.update_state_paths();

        // let res = ticker.tick_transitions();
        // println!("FIRED: {:?}", res);
    }

}
