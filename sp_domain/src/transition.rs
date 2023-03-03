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
        self.guard.eval(state)
    }

    fn eval2(&self, state: &SPState2) -> bool {
        self.guard.eval2(state) && self.actions.iter().all(|a| a.eval2(state))
    }
}

impl NextAction for Transition {
    fn next(&self, state: &mut SPState) -> SPResult<()> {
        for a in self.actions.iter() {
            a.next(state)?;
        }
        Ok(())
    }

    fn next2(&self, state: &mut SPState2) -> SPResult<()> {
        for a in self.actions.iter() {
            a.next2(state)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test_items {
    use super::*;

    #[test]
    fn testing_transitions() {
        let ab = SPPath::from_slice(&["a", "b"]);
        let ac = SPPath::from_slice(&["a", "c"]);
        let kl = SPPath::from_slice(&["k", "l"]);
        let xy = SPPath::from_slice(&["x", "y"]);

        let mut s = state!(ab => 2, ac => true, kl => 3, xy => false);
        let p = p!([!p: ac] && [!p: xy]);

        let a = a!(p: ac = false);
        let b = a!(p:ab <- p:kl);
        let c = a!(p:xy ? p);

        let t1 = Transition::new("t1", p!(p: ac), vec![a]);
        let t2 = Transition::new("t2", p!(!p: ac), vec![b]);
        let t3 = Transition::new("t3", Predicate::TRUE, vec![c]);

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

        s.take_transition();
        let res = t3.eval(&s);
        println!("t3: {:?}", res);
        assert!(res);
        t3.next(&mut s).unwrap();

        s.take_transition();
        assert_eq!(s.sp_value_from_path(&xy).unwrap(), &true.to_spvalue());
    }
}
