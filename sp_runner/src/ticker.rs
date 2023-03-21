use sp_domain::*;

#[derive(Debug, PartialEq, Clone)]
pub struct RunnerPredicate(StatePath, Predicate);

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Ticker {
    pub state: SPState,

    pub controlled_transitions: Vec<Transition>,
    pub auto_transitions: Vec<Transition>,
    pub runner_transitions: Vec<Transition>,

    pub predicates: Vec<NamedPredicate>,

    /// Predicates with computed statepaths
    pub runner_predicates: Vec<RunnerPredicate>,

    /// Allowed to run
    pub controlled_queue: Vec<SPPath>,
}

impl Ticker {
    pub fn tick_transitions(&mut self) -> Vec<SPPath> {
        let mut fired = Vec::new();
        let mut counter = 0;
        loop {
            let f = self.tick_auto();

            if f.is_empty() {
                // println!("f empty, fired is {:?}", fired);
                break;
            } else {
                counter += 1;
                if counter > 10 {
                    // there is probably a self loop in the model
                    let t_names = f
                        .iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    panic!("self loop with transitions {t_names}");
                    // break;
                }
                println!("runner one more time! adding new fired {f:?}");
                fired.extend(f);
            }
        }
        fired
    }

    pub fn tick_auto(&mut self) -> Vec<SPPath> {
        Ticker::upd_preds(&mut self.state, &self.runner_predicates);
        self.auto_transitions
            .iter()
            .flat_map(|t| {
                if !t.actions.is_empty() && t.eval(&self.state) {
                    // TODO: handle errors
                    let _r = t.next(&mut self.state);
                    Ticker::upd_preds(&mut self.state, &self.runner_predicates);
                    Some(t.path.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn tick_first_controlled(&mut self) -> Option<SPPath> {
        Ticker::upd_preds(&mut self.state, &self.runner_predicates);
        if let Some(first) = self.controlled_queue.first() {
            if let Some(first) = self.controlled_transitions.iter().find(|t|&t.path == first) {
                if first.eval(&self.state) {
                    // TODO: handle errors
                    let _r = first.next(&mut self.state);
                    Ticker::upd_preds(&mut self.state, &self.runner_predicates);
                    let _throw_first = self.controlled_queue.pop();
                    return Some(first.path.clone());
                }
            }
        }
        None
    }

    pub fn upd_preds(state: &mut SPState, rps: &[RunnerPredicate]) {
        for rp in rps {
            let value = rp.1.eval(state).to_spvalue();
            if let Err(e) = state.force(&rp.0, &value) {
                eprintln!(
                    "The predicate {:?} does not have an updated state path. Got error: {}",
                    rp.0, e
                );
            }
        }
    }


    /// After changing anything in the Ticker, run this method to update the state variables.
    pub fn update_state_paths(&mut self) {
        for x in self.controlled_transitions.iter_mut() {
            x.upd_state_path(&self.state)
        }
        for x in self.auto_transitions.iter_mut() {
            x.upd_state_path(&self.state)
        }
        for x in self.runner_transitions.iter_mut() {
            x.upd_state_path(&self.state)
        }

        let psp: Vec<RunnerPredicate> = self.predicates.iter()
            .map(|p| RunnerPredicate(self.state.state_path(&p.path).expect("pred not in state"),
                                     p.predicate.clone())).collect();
        self.runner_predicates = psp;

        // also update any new predicates with values their correct assignments
        Ticker::upd_preds(&mut self.state, &self.runner_predicates);
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

        let mut ticker = Ticker {
            state: s,
            auto_transitions: vec![t1 ,t2],
            .. Ticker::default()
        };
        ticker.update_state_paths();

        let res = ticker.tick_transitions();
        println!("FIRED: {:?}", res);
    }

    // #[test]
    // fn testing_large_model() {
    //     let plan = SPPath::from_string("plan");
    //     let mut s = state!(plan => 1);

    //     let ts: Vec<Transition> = (1..100)
    //         .map(|i| {
    //             let g = p! {plan == i};
    //             let a = a!(plan = (i + 1));
    //             let rg = Predicate::TRUE;
    //             let ra = vec![];
    //             Transition::new(&format!("t_{}", i), g, rg, vec![a], ra, TransitionType::Auto)
    //         })
    //         .collect();

    //     let upd_ts: Vec<Vec<&Transition>> = ts.iter().map(|t| (vec![t])).collect();
    //     let _x = &upd_ts;

    //     let ps = vec![];
    //     for _i in 1..100 {
    //         let res = SPTicker::tick(&mut s, &upd_ts, &ps);
    //         s.take_transition();
    //         println!("fired: {:?}, state: {:?}", res, s.sp_value_from_path(&plan));
    //     }
    // }
}
