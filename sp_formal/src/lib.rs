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
        let x = x.path.clone();
        let y = y.path.clone();

        let move_x = Transition::new("move_x".into(), p!(!p: x), vec![a!(p: x)]);
        let move_y = Transition::new("move_y".into(), p!(!p: y), vec![a!(p: y)]);
        tsm.transitions.push(move_x);
        tsm.transitions.push(move_y);

        let goal = p!([p: x] && [p: y]);
        let initial_state = state!(x => false, "y" => false);

        let result = plan(&tsm, &[(goal.clone(), None)], &initial_state, 5);
        assert!(result.is_ok());
        assert!(result.unwrap().plan_found);

        // add invariant that makes it impossible to reach goal.
        let invar_x = NamedPredicate::new("x_invar".into(), p!(! p:x));
        tsm.invariants.push(invar_x);
        let result = plan(&tsm, &[(goal, None)], &initial_state, 5);
        assert!(result.is_ok());
        assert!(!result.unwrap().plan_found);
    }
}
