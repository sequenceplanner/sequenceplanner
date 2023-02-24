use sp_domain::*;

pub use sp_model_derive::Resource;

pub trait Resource {
    fn new(name: &str) -> Self;
    fn get_variables(&self) -> Vec<Variable>;
}

macro_rules! px {
    // parens
    (($($inner:tt)+) ) => {{
        // println!("matched parens: {}", stringify!($($inner)+));
        px! ( $($inner)+ )
    }};
    ([$($inner:tt)+] ) => {{
        // println!("matched square parens: {}", stringify!($($inner)+));
        px! ( $($inner)+ )
    }};

    // AND: the brackets are needed because "tt" includes && which
    // leads to ambiguity without an additional delimeter
    ([$($first:tt)+] $(&& [$($rest:tt)+])+) => {{
        // println!("matched &&: {}", stringify!($($first)+));
        let first = px! ( $($first)+ );
        let mut v = vec![first];
        $(
            // println!(" && ...: {}", stringify!($($rest)+));
            let r = px!($($rest)+);
            v.push(r);
        )*
        Predicate::AND(v)
    }};

    // OR: same as and.
    ([$($first:tt)+] $(|| [$($rest:tt)+])+) => {{
        // println!("matched ||: {}", stringify!($($first)+));
        let first = px! ( $($first)+ );
        let mut v = vec![first];
        $(
            let r = px!($($rest)+);
            v.push(r);
        )*
        Predicate::OR(v)
    }};

    // implication
    ([$($x:tt)+] => [$($y:tt)+]) => {{
        // println!("matched implication: {} => {}", stringify!($($x)+), stringify!($($y)+));
        let x = px! ( $($x)+ );
        let y = px! ( $($y)+ );
        Predicate::OR(vec![Predicate::NOT(Box::new(x)), y])
    }};


    ([ $lhs:expr ] == [ $rhs:expr ]) => {{
        // println!("matched [{}] == [{}]", stringify!($lhs), stringify!($rhs));
        Predicate::EQ(
            $lhs .to_predicate_value(),
            $rhs .to_predicate_value(),
        )
    }};

    ([ $lhs:expr ] == $($rhs:tt).+) => {{
        // println!("matched [{}] == {}", stringify!($lhs), stringify!($($rhs).+));
        Predicate::EQ(
            $lhs .to_predicate_value(),
            $($rhs).+ .to_predicate_value(),
        )
    }};

    ($($lhs:tt).+ == [ $rhs:expr ]) => {{
        // println!("matched {} == [{}]", stringify!($($lhs).+), stringify!($rhs));
        Predicate::EQ(
            $($lhs).+ .to_predicate_value(),
            $rhs .to_predicate_value(),
        )
    }};

    ($($lhs:tt).+ == $($rhs:tt).+) => {{
        // println!("matched {} == {}", stringify!($($lhs).+), stringify!($($rhs).+));
        Predicate::EQ(
            $($lhs).+ .to_predicate_value(),
            $($rhs).+ .to_predicate_value(),
        )
    }};

    ($i:expr) => {{
        $i.to_predicate()
    }};

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predicate_macro() {
        struct Test {
            path: SPPath,
            variable: Variable,
        }
        let r = Test {
            path: SPPath::from_string("r.path"),
            variable: Variable::new_boolean("r.var"),
        };
        let p = SPPath::from_string("p.path");

        impl Test {
            fn fun_p() -> SPPath {
                return SPPath::from_string("path_from_fun");
            }
            fn fun_v() -> Variable {
                return Variable::new_boolean("r.var");
            }
        }

        // let x = px!((r.path != true) && (r.path != true) && ((r.path != true) && (r.path != true)));
        // let x = px!(r.path != true);
        let x = px!(r.path == r.variable);
        println!("{x:?}\n");
        let x = px!([Test::fun_v()] == r.path);
        println!("{x:?}\n");
        let x = px!([[r.variable] == r.path] && [x]);
        println!("{x:?}\n");
        let x = px!([r.variable == ["hello"]] && [p == p]);
        println!("{x:?}\n");
        let x = px!([[r.variable] == [Test::fun_v()]] && [x]);
        println!("{x:?}\n");
        let x = px!(r.variable);
        println!("{x:?}\n");
        let y = px!([r.variable == [Test::fun_p()]] && [r.variable]);
        let x = px!([y] => [y]);
        println!("{x:?}\n");
    }

    #[derive(Resource)]
    struct Foo {
        #[Variable(type = "String", initial = "hej", domain = "hej svejs")]
        pub field1: Variable,

        #[Variable(type = "bool", initial = true, domain = "true true")]
        field2: Variable,

        #[Variable(type = "int", initial = 5, domain = "1 2 3 4 5")]
        field3: Variable,

        #[Variable(type = "float", initial = 5.5, domain = "1.0 2.0 3.0 4.0 5.0")]
        field4: Variable,

        #[Resource]
        nested: Nested,
    }

    #[derive(Resource)]
    struct Nested {
        #[Variable(type = "String", domain = "hej svejs")]
        variable_string: Variable,
    }

    #[derive(Resource)]
    struct Model {
        #[Resource]
        resource1: Foo,
        #[Resource]
        resource2: Nested,
    }

    #[test]
    fn test_derive() {
        let model: Model = Model::new("model");
        let vars = model.get_variables();
        let initial_state = SPState::new_from_variables(&vars);
        println!("initial_state:\n{}", initial_state);

        let pred = px!(model.resource1.field2 == true);
        assert!(pred.eval(&initial_state));

        let pred = px!(model.resource1.field3 == 5);
        assert!(pred.eval(&initial_state));

        let pred = px!(model.resource1.nested.variable_string == [SPValue::Unknown]);
        assert!(pred.eval(&initial_state));

        let pred = px!(model.resource2.variable_string == [SPValue::Unknown]);
        assert!(pred.eval(&initial_state));
    }
}
