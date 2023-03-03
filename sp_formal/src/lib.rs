pub mod transition_system_model;
pub use transition_system_model::*;

pub mod planning;
pub use planning::*;

#[cfg(test)]
mod planning_tests {
    #![warn(unused_variables)]

    use super::*;
    use sp_domain::*;

    #[test]
    fn test_nusmv_planner() {
        let mut tsm = TransitionSystemModel::default();
        let x = Variable::new_boolean("x.y.z".into());
        let y = Variable::new_boolean("y.z".into());
        tsm.vars.push(x.clone());
        tsm.vars.push(y.clone());

        let move_x = Transition::new("move_x".into(), p!(!x), vec![a!(x)]);
        let move_y = Transition::new("move_y".into(), p!(!y), vec![a!(y)]);
        tsm.transitions.push(move_x);
        tsm.transitions.push(move_y);

        let goal = p!([x] && [y]);
        let x = x.path.clone();
        let y = y.path.clone();
        let initial_state = state!(x => false, y => false);

        let result = plan(&tsm, &[(goal.clone(), None)], &initial_state, 5);
        assert!(result.is_ok());
        assert!(result.unwrap().plan_found);

        // add invariant that makes it impossible to reach goal.
        let invar_x = NamedPredicate::new("x_invar".into(), p!(!x));
        tsm.invariants.push(invar_x);
        let result = plan(&tsm, &[(goal, None)], &initial_state, 5);
        assert!(result.is_ok());
        assert!(!result.unwrap().plan_found);
    }
}
