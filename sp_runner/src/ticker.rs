use sp_domain::*;

pub struct Ticker {
    pub state: SPState,
    pub controlled_transitions: Vec<Transition>,
    pub auto_transitions: Vec<Transition>,
    pub predicates: Vec<NamedPredicate>,
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
        Ticker::upd_preds(&mut self.state, &self.predicates);
        self.auto_transitions
            .iter()
            .flat_map(|t| {
                if !t.actions.is_empty() && t.eval(&self.state) {
                    // TODO: handle errors
                    let _r = t.next(&mut self.state);
                    Ticker::upd_preds(&mut self.state, &self.predicates);
                    Some(t.path.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn tick_first_controlled(&mut self) -> Option<SPPath> {
        Ticker::upd_preds(&mut self.state, &self.predicates);
        if let Some(first) = self.controlled_queue.first() {
            if let Some(first) = self.controlled_transitions.iter().find(|t|&t.path == first) {
                if first.eval(&self.state) {
                    // TODO: handle errors
                    let _r = first.next(&mut self.state);
                    Ticker::upd_preds(&mut self.state, &self.predicates);
                    let _throw_first = self.controlled_queue.pop();
                    return Some(first.path.clone());
                }
            }
        }
        None
    }

    /// TODO: cache state paths.
    pub fn upd_preds(state: &mut SPState, predicates: &[NamedPredicate]) {
        predicates.iter().for_each(|pr| {
            let value = pr.predicate.eval(state).to_spvalue();
            if let Err(e) = state.force_from_path(&pr.path, &value) {
                eprintln!(
                    "The predicate {:?} does not have an updated state path. Got error: {}",
                    pr.path, e
                );
            }
        })
    }
}
