//!
//!
use super::*;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub path: SPPath,
    pub guard: Predicate,
    pub actions: Vec<Action>,
}

impl Transition {
    pub fn new(path: SPPath, guard: Predicate, actions: Vec<Action>) -> Self {
        Transition {
            path,
            guard,
            actions,
        }
    }

    pub fn upd_state_path(&mut self, state: &SPState) {
        self.guard.upd_state_path(state);
        self.actions
            .iter_mut()
            .for_each(|a| a.upd_state_path(state));
    }

    // TODO: think about if this should include runner actions.
    pub fn modifies(&self) -> HashSet<SPPath> {
        let mut r = HashSet::new();

        r.extend(self.actions.iter().map(|a| a.var.clone()));
        r
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = format!("{}: {}/{:?}", self.path, self.guard, self.actions);
        write!(fmtr, "{s}")
    }
}

impl EvaluatePredicate for Transition {
    fn eval(&self, state: &SPState) -> bool {
        self.guard.eval(state) && self.actions.iter().all(|a| a.eval(state))
    }
}

impl NextAction for Transition {
    fn next(&self, state: &mut SPState) -> SPResult<()> {
        for a in &self.actions {
            a.next(state)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test_items {
    use super::*;

    #[test]
    fn testing_transitions() {
        let ab = SPPath::from(&["a", "b"]);
        let ac = SPPath::from(&["a", "c"]);
        let kl = SPPath::from(&["k", "l"]);
        let xy = SPPath::from(&["x", "y"]);

        let mut s = state!(ab => 2, ac => true, kl => 3, xy => false);

        let a = a!(ac = false);
        let b = a!(ab = kl);

        let t1 = Transition::new("t1".into(), p!(ac), vec![a]);
        let t2 = Transition::new("t2".into(), p!(!ac), vec![b]);

        let res = t1.eval(&s);
        println!("t1.eval: {:?}", res);
        assert!(res);

        let res = t1.next(&mut s);
        println!("t1.next: {:?}", res);

        s.take_transition();
        assert_eq!(s.sp_value_from_path(&ac).unwrap(), &false.to_spvalue());

        let res = t2.eval(&s);
        println!("t2: {:?}", res);
        assert!(res);

        let res = t2.next(&mut s);
        println!("t2.next: {:?}", res);

        s.take_transition();
        assert_eq!(s.sp_value_from_path(&ab).unwrap(), &3.to_spvalue());
    }
}
