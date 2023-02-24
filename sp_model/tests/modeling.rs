use sp_domain::*;
use sp_model::*;

/// A test model using nested structs and improved predicate macro.

#[derive(Resource)]
struct Resource1 {
    #[Variable(type = "String", initial = "hej", domain = "hej svejs")]
    pub field1: Variable,

    #[Variable(type = "bool", initial = true, domain = "true true")]
    field2: Variable,

    #[Variable(type = "int", initial = 5, domain = "1 2 3 4 5")]
    field3: Variable,

    #[Variable(type = "float", initial = 5.5, domain = "1.0 2.0 3.0 4.0 5.0")]
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

    let pred = px!(model.resource1.field2 == true);
    assert!(pred.eval(&state));

    let pred = px!(model.resource1.field3 == 5);
    assert!(pred.eval(&state));

    let pred = px!(model.resource1.nested.variable_string == [SPValue::Unknown]);
    assert!(pred.eval(&state));

    // set state by path
    state.add_variable(SPPath::from_string("/model/resource2/variable_string"),
                       "hello".to_spvalue());

    let pred = px!(model.resource2.variable_string == "hello");
    assert!(pred.eval(&state));
}
