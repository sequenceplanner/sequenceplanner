use sp_domain::*;
use sp_model::*;
use sp_formal::*;

/// A test model using nested structs and improved predicate macro.

#[derive(Resource)]
struct Resource1 {
    #[Variable(type = "String", initial = "hej", domain = "hej svejs")]
    #[Output(mapping = "hello")]
    pub field1: Variable,

    #[Variable(type = "bool", initial = true, domain = "true true")]
    #[Output] // default output mapping is the field name
    field2: Variable,

    #[Variable(type = "int", initial = 5, domain = "1 2 3 4 5")]
    #[Input] // default input mapping is the field name
    field3: Variable,

    #[Variable(type = "float", initial = 5.5, domain = "1.0 2.0 3.0 4.0 5.0")]
    #[Input(mapping = "field_4_more")]
    field4: Variable,

    #[Resource]
    nested: Resource2,
}

#[derive(Resource)]
struct Resource2 {
    // No initial value set here, will be SPValue::Unknown.
    #[Variable(type = "String", domain = "hej svejs")]
    variable_string: Variable,
}

#[derive(Resource)]
struct Model {
    #[Resource]
    resource1: Resource1,
    #[Resource]
    resource2: Resource2,
}

#[test]
fn make_model() {
    let model: Model = Model::new("model");
    let vars = model.get_variables();
    let mut state = SPState::new_from_variables(&vars);
    println!("initial_state:\n{}", state);

    let input_mapping = model.resource1.setup_inputs("test_input_topic", "std_msgs/msg/String");
    println!("input_mapping\n{:#?}", input_mapping);
    let output_mapping = model.resource1.setup_outputs("test_topic", "std_msgs/msg/String");
    println!("output_mapping\n{:#?}", output_mapping);

    let pred = p!(model.resource1.field2);
    assert!(pred.eval(&state));

    let pred = p!(model.resource1.field3 == 5);
    assert!(pred.eval(&state));

    let pred = p!(model.resource1.nested.variable_string == [SPValue::Unknown]);
    assert!(pred.eval(&state));

    // set state by path
    state.add_variable("model.resource2.variable_string".into(), "hello".to_spvalue());

    let pred = p!(model.resource2.variable_string == "hello");
    assert!(pred.eval(&state));
}


#[test]
fn model_and_planner() {
    #[derive(Resource)]
    struct Resource1 {
        // No initial value set here, will be SPValue::Unknown.
        #[Variable(type = "String", initial = "one", domain = "one two")]
        x: Variable,
    }

    #[derive(Resource)]
    struct Resource2 {
        // No initial value set here, will be SPValue::Unknown.
        #[Variable(type = "bool", initial = false)]
        y: Variable,
    }

    #[derive(Resource)]
    struct Model {
        #[Resource]
        r1: Resource1,
        #[Resource]
        r2: Resource2,
    }

    let m: Model = Model::new("m");

    let vars = m.get_variables();
    let state = SPState::new_from_variables(&vars);

    let mut tsm = TransitionSystemModel::default();
    tsm.vars.extend(vars);

    let t1 = Transition::new("t1".into(), p!(m.r1.x == "one"), vec![a!(m.r1.x = "two")]);
    let t2 = Transition::new("t2".into(), p!(!m.r2.y), vec![a!(m.r2.y)]);
    tsm.transitions.push(t1);
    tsm.transitions.push(t2);

    let goal = p!([m.r1.x == "two"] && [m.r2.y]);

    let result = plan(&tsm, &[(goal.clone(), None)], &state, 5);
    assert!(result.is_ok());
    assert!(result.unwrap().plan_found);
}


#[test]
fn empty_model_with_operation() {
    #[derive(Resource)]
    struct Model {
    }

    let m: Model = Model::new("m");

    let mut vars = m.get_variables();
    let (op_var, op_trans) = operation("test_op".into(),
                                       Predicate::TRUE, vec![],
                                       Predicate::TRUE, vec![],
                                       Predicate::TRUE, vec![],
                                       Predicate::TRUE, vec![]);
    vars.push(op_var.clone());

    let mut tsm = TransitionSystemModel::default();
    tsm.vars.extend(vars);
    tsm.transitions.extend(get_formal_transitions(&op_trans));
    let state = SPState::new_from_variables(&tsm.vars);
    let goal = p!(op_var == "f"); // goal is operation should be finished.

    let result = plan(&tsm, &[(goal.clone(), None)], &state, 5);
    assert!(result.is_ok());
    assert!(result.unwrap().plan_found);
}
