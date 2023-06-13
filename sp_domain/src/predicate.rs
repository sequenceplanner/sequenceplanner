/// This file defines both predicates and actions

use super::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Predicate {
    AND(Vec<Predicate>),
    OR(Vec<Predicate>),
    XOR(Vec<Predicate>),
    NOT(Box<Predicate>),
    TRUE,
    FALSE,
    EQ(PredicateValue, PredicateValue),
    NEQ(PredicateValue, PredicateValue),
    TON(PredicateValue, PredicateValue),
    TOFF(PredicateValue, PredicateValue),
    MEMBER(PredicateValue, PredicateValue), // GT(PredicateValue, PredicateValue),
                                            // LT(PredicateValue, PredicateValue),
                                            // INDOMAIN(PredicateValue, Vec<PredicateValue>)
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Action {
    pub var: SPPath,
    pub value: Compute,
    state_path: Option<StatePath>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PredicateValue {
    SPValue(SPValue),
    SPPath(SPPath, Option<StatePath>),
}

pub trait ToPredicateValue {
    fn to_predicate_value(&self) -> PredicateValue;
}

// Just a macro helper...
pub trait ToPredicate {
    fn to_predicate(&self) -> Predicate;
}

impl ToPredicate for Predicate {
    fn to_predicate(&self) -> Predicate {
        self.clone()
    }
}

/// Used in actions to compute a new SPValue.
/// When using delay and fetching a value from another variable, the current value of that
/// variable will be taken and assigned to the action variable after the delay, and not the
/// value that the other has after the delay.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Compute {
    PredicateValue(PredicateValue),
    Predicate(Predicate), // used for boolean actions
    Function(Vec<(Predicate, PredicateValue)>),
    // TODO: AddMember och RemoveMember
    TimeStamp,
    Random(i32), // random number 0 < x < n
    Any,         // Free variable, can take on any value after this action.
                 // If we need more advanced functions we can add them here
                 //TakeNext(SPValue, Vec<SPValue>), // to be impl when needed
                 //TakeBefore(SPValue, Vec<SPValue>),
                 // Add(Box<Compute>, Box<Compute>),
                 // Sub(Box<Compute>, Box<Compute>),
                 // Join(Box<Compute>, Box<Compute>),
}

impl<'a> PredicateValue {
    pub fn sp_value(&'a self, state: &'a SPState) -> Option<&'a SPValue> {
        match self {
            PredicateValue::SPValue(x) => Some(x),
            PredicateValue::SPPath(path, sp) => {
                if sp.is_none() {
                    state.sp_value_from_path(path)
                } else if let Some(the_path) = sp {
                    state.sp_value(the_path)
                } else {
                    None
                }
            }
        }
    }

    pub fn sp_value2(&'a self, state: &'a SPState2) -> Option<&'a SPValue> {
        match self {
            PredicateValue::SPValue(x) => Some(x),
            PredicateValue::SPPath(path, _) => {
                let string = path.path.join(".");
                state.get(&string)
            }
        }
    }

    pub fn upd_state_path(&mut self, state: &SPState) {
        if let PredicateValue::SPPath(path, sp) = self {
            if sp.is_none() {
                *sp = state.state_path(path)
            } else if sp
                .clone()
                .map(|x| x.state_id != state.id())
                .unwrap_or(false)
            {
                *sp = state.state_path(path);
            }
        }
    }

    pub fn value(v: SPValue) -> Self {
        PredicateValue::SPValue(v)
    }
    pub fn path(p: SPPath) -> Self {
        PredicateValue::SPPath(p, None)
    }
}

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        self.var == other.var && self.value == other.value
    }
}

impl PartialEq for PredicateValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PredicateValue::SPValue(a), PredicateValue::SPValue(b)) => a == b,
            (PredicateValue::SPPath(a, _), PredicateValue::SPPath(b, _)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for PredicateValue {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PredicateValue::SPValue(v) => write!(fmtr, "{v}"),
            PredicateValue::SPPath(p, _) => write!(fmtr, "{p}"),
        }
    }
}

impl fmt::Display for Predicate {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = match &self {
            Predicate::AND(x) => {
                let children: Vec<_> = x.iter().map(|p| format!("{p}")).collect();
                format!("({})", children.join(" && "))
            }
            Predicate::OR(x) => {
                let children: Vec<_> = x.iter().map(|p| format!("{p}")).collect();
                format!("({})", children.join(" || "))
            }
            Predicate::XOR(_) => "TODO".into(), // remove from pred?
            Predicate::NOT(p) => format!("!({p})"),
            Predicate::TRUE => "TRUE".into(),
            Predicate::FALSE => "FALSE".into(),
            Predicate::EQ(x, y) => format!("{x} = {y}"),
            Predicate::NEQ(x, y) => format!("{x} != {y}"),
            Predicate::TON(t, d) => {
                format!("TON(t:{t} d:{d})")
            }
            Predicate::TOFF(t, d) => {
                format!("TOFF(t:{t} d:{d})")
            }
            Predicate::MEMBER(t, d) => {
                format!("is {t} a member of {d}?")
            }
        };

        write!(fmtr, "{}", &s)
    }
}

impl Default for PredicateValue {
    fn default() -> Self {
        PredicateValue::SPValue(false.to_spvalue())
    }
}

impl Predicate {
    pub fn from_string(from: &str) -> Option<Self> {
        predicate_parser::pred_parser::pred(from).ok()
    }

    pub fn upd_state_path(&mut self, state: &SPState) {
        match self {
            Predicate::AND(x) | Predicate::OR(x) | Predicate::XOR(x) => {
                x.iter_mut().for_each(|p| p.upd_state_path(state))
            }
            Predicate::NOT(x) => x.upd_state_path(state),
            Predicate::TRUE | Predicate::FALSE => {}
            Predicate::EQ(x, y)
            | Predicate::NEQ(x, y)
            | Predicate::TON(x, y)
            | Predicate::TOFF(x, y)
            | Predicate::MEMBER(x, y) => {
                x.upd_state_path(state);
                y.upd_state_path(state);
            }
        }
    }

    /// Return the supporting variables of this expression
    pub fn support(&self) -> Vec<SPPath> {
        let mut s = Vec::new();
        match &self {
            Predicate::AND(x) | Predicate::OR(x) | Predicate::XOR(x) => {
                s.extend(x.iter().flat_map(|p| p.support()))
            }
            Predicate::NOT(x) => s.extend(x.support()),
            Predicate::TRUE | Predicate::FALSE => {}
            Predicate::EQ(x, y)
            | Predicate::NEQ(x, y)
            | Predicate::TON(x, y)
            | Predicate::TOFF(x, y)
            | Predicate::MEMBER(x, y) => {
                if let PredicateValue::SPPath(p, _) = x {
                    s.push(p.clone())
                }
                if let PredicateValue::SPPath(p, _) = y {
                    s.push(p.clone())
                }
            }
        };
        s.sort();
        s.dedup();
        s
    }

    /// Recursively clean expression and keep only constants and allowed paths
    pub fn keep_only(&self, only: &[SPPath]) -> Option<Predicate> {
        match &self {
            Predicate::AND(x) => {
                let mut new: Vec<_> = x.iter().flat_map(|p| p.keep_only(only)).collect();
                new.dedup();
                if new.is_empty() {
                    None
                } else if new.len() == 1 {
                    Some(new[0].clone())
                } else {
                    Some(Predicate::AND(new))
                }
            }
            Predicate::OR(x) => {
                let mut new: Vec<_> = x.iter().flat_map(|p| p.keep_only(only)).collect();
                new.dedup();
                if new.is_empty() {
                    None
                } else if new.len() == 1 {
                    Some(new[0].clone())
                } else {
                    Some(Predicate::OR(new))
                }
            }
            Predicate::XOR(x) => {
                let mut new: Vec<_> = x.iter().flat_map(|p| p.keep_only(only)).collect();
                new.dedup();
                if new.is_empty() {
                    None
                } else if new.len() == 1 {
                    Some(new[0].clone())
                } else {
                    Some(Predicate::XOR(new))
                }
            }
            Predicate::NOT(x) => x.keep_only(only).map(|x| Predicate::NOT(Box::new(x))),
            Predicate::TRUE => Some(Predicate::TRUE),
            Predicate::FALSE => Some(Predicate::FALSE),
            Predicate::EQ(x, y)
            | Predicate::NEQ(x, y)
            | Predicate::TON(x, y)
            | Predicate::TOFF(x, y)
            | Predicate::MEMBER(x, y) => {
                let remove_x = match x {
                    PredicateValue::SPValue(_) => false,
                    PredicateValue::SPPath(p, _) => !only.contains(p),
                };
                let remove_y = match y {
                    PredicateValue::SPValue(_) => false,
                    PredicateValue::SPPath(p, _) => !only.contains(p),
                };

                if remove_x || remove_y {
                    None
                } else {
                    Some(self.clone())
                }
            }
        }
    }
}

impl Action {
    pub fn new(var: SPPath, value: Compute) -> Self {
        Action {
            var,
            value,
            state_path: None,
        }
    }

    pub fn upd_state_path(&mut self, state: &SPState) {
        match &self.state_path {
            Some(sp) if sp.state_id != state.id() => self.state_path = state.state_path(&self.var),
            None => self.state_path = state.state_path(&self.var),
            _ => {}
        }
    }

    pub fn revert_action(&self, state: &mut SPState) -> SPResult<()> {
        match &self.state_path {
            Some(sp) => state.revert_next(sp),
            None => state.revert_next_from_path(&self.var),
        }
    }

    pub fn to_predicate(&self) -> Option<Predicate> {
        match &self.value {
            Compute::PredicateValue(p) => Some(Predicate::EQ(
                PredicateValue::SPPath(self.var.clone(), None),
                p.clone(),
            )),
            _ => None,
        }
    }

    pub fn to_concrete_predicate(&self, state: &SPState) -> Option<Predicate> {
        match &self.value {
            Compute::PredicateValue(PredicateValue::SPPath(p, _)) => {
                let pv = state
                    .state_value_from_path(p)
                    .expect("no such value in the state")
                    .current_value();
                Some(Predicate::EQ(
                    PredicateValue::SPPath(self.var.clone(), None),
                    PredicateValue::SPValue(pv.clone()),
                ))
            }
            Compute::PredicateValue(p) => Some(Predicate::EQ(
                PredicateValue::SPPath(self.var.clone(), None),
                p.clone(),
            )),
            _ => None,
        }
    }

    pub fn val_to_string(&self) -> String {
        match &self.value {
            Compute::PredicateValue(PredicateValue::SPValue(v)) => v.to_string(),
            Compute::PredicateValue(PredicateValue::SPPath(p, _)) => p.to_string(),
            Compute::Predicate(p) => p.to_string(),
            Compute::Function(xs) => xs.iter().fold(String::default(), |acc, (p, v)| {
                format!("{acc}[if {p} then {v}]")
            }),
            Compute::Any => "?".to_string(),
            Compute::Random(n) => format!("rnd({n})"),
            Compute::TimeStamp => "T".to_string(),
        }
    }

    pub fn to_string_short(&self) -> String {
        format!("{} := {}", self.var.leaf(), self.val_to_string())
    }
}

impl fmt::Display for Action {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = format!("{} := {}", self.var, self.val_to_string());
        write!(fmtr, "{}", &s)
    }
}

impl Default for Predicate {
    fn default() -> Self {
        Predicate::TRUE
    }
}

impl Default for Compute {
    fn default() -> Self {
        Compute::PredicateValue(PredicateValue::default())
    }
}

/// Eval is used to evaluate a predicate (or an operation ).
pub trait EvaluatePredicate {
    fn eval(&self, state: &SPState) -> bool;
    fn eval2(&self, state: &SPState2) -> bool;
}

pub trait NextAction {
    fn next(&self, state: &mut SPState) -> SPResult<()>;
    fn next2(&self, state: &mut SPState2) -> SPResult<()>;
}

impl EvaluatePredicate for Predicate {
    fn eval(&self, state: &SPState) -> bool {
        match self {
            Predicate::AND(ps) => ps.iter().all(|p| p.eval(state)),
            Predicate::OR(ps) => ps.iter().any(|p| p.eval(state)),
            Predicate::XOR(ps) => {
                let mut c = 0;
                for p in ps.iter() {
                    if p.eval(state) {
                        c += 1;
                    }
                }
                c == 1
                // ps.iter_mut()
                //     .filter(|p| p.eval(state))  // for some reason does not filter with &mut
                //     .count()
                //     == 1
            }
            Predicate::NOT(p) => !p.eval(state),
            Predicate::TRUE => true,
            Predicate::FALSE => false,
            Predicate::EQ(lp, rp) => {
                let a = lp.sp_value(state);
                let b = rp.sp_value(state);
                if let (Some(a), Some(b)) = (a, b) {
                    a == b
                } else {
                    eprintln!(
                        "ERROR: eval in predicate EQ: path {lp} or {rp} not found in\n{state}"
                    );
                    false
                }
            }
            Predicate::NEQ(lp, rp) => {
                let a = lp.sp_value(state);
                let b = rp.sp_value(state);
                if let (Some(a), Some(b)) = (a, b) {
                    a != b
                } else {
                    eprintln!(
                        "ERROR: eval in predicate NEQ: path {lp} or {rp} not found in\n{state}"
                    );
                    false
                }
            }
            Predicate::TON(lp, rp) => {
                if let (Some(t), Some(d)) = (lp.sp_value(state), rp.sp_value(state)) {
                    if let SPValue::Time(time) = t {
                        let current_duration = time.elapsed().unwrap_or_default();
                        let delay = match d {
                            SPValue::Float32(x) => *x as i32,
                            SPValue::Int32(x) => *x,
                            _ => 0,
                        };
                        current_duration.as_millis() > delay.unsigned_abs() as u128
                    } else {
                        eprintln!("TON must point to a timestamp, and not: {t:?} i");
                        false
                    }
                } else {
                    eprintln!(
                        "ERROR: eval in predicate TON: path {lp} or {rp} not found in\n{state}"
                    );
                    false
                }
            }
            Predicate::TOFF(lp, rp) => {
                if let (Some(t), Some(d)) = (lp.sp_value(state), rp.sp_value(state)) {
                    if let SPValue::Time(time) = t {
                        let current_duration = time.elapsed().unwrap_or_default();
                        let delay = match d {
                            SPValue::Float32(x) => *x as i32,
                            SPValue::Int32(x) => *x,
                            _ => 0,
                        };
                        current_duration.as_millis() < delay.unsigned_abs() as u128
                    } else {
                        eprintln!("TON must point to a timestamp, and not: {t:?} i");
                        false
                    }
                } else {
                    eprintln!(
                        "ERROR: eval in predicate TON: path {lp} or {rp} not found in\n{state}",
                    );
                    false
                }
            }
            Predicate::MEMBER(lp, rp) => {
                if let (Some(v), Some(xs)) = (lp.sp_value(state), rp.sp_value(state)) {
                    if let SPValue::Array(_, xs) = xs {
                        xs.contains(v)
                    } else {
                        eprintln!("Member must point to an array, and not: {xs:?} i");
                        false
                    }
                } else {
                    eprintln!(
                        "ERROR: eval in predicate MEMBER: path {lp} or {rp} not found in\n{state}",
                    );
                    false
                }
            } // Predicate::GT(lp, rp) => {}
              // Predicate::LT(lp, rp) => {}
              // Predicate::INDOMAIN(value, domain) => {}
        }
    }

    fn eval2(&self, state: &SPState2) -> bool {
        match self {
            Predicate::AND(ps) => ps.iter().all(|p| p.eval2(state)),
            Predicate::OR(ps) => ps.iter().any(|p| p.eval2(state)),
            Predicate::XOR(ps) => {
                let mut c = 0;
                for p in ps.iter() {
                    if p.eval2(state) {
                        c += 1;
                    }
                }
                c == 1
                // ps.iter_mut()
                //     .filter(|p| p.eval(state))  // for some reason does not filter with &mut
                //     .count()
                //     == 1
            }
            Predicate::NOT(p) => !p.eval2(state),
            Predicate::TRUE => true,
            Predicate::FALSE => false,
            Predicate::EQ(lp, rp) => {
                let a = lp.sp_value2(state);
                let b = rp.sp_value2(state);
                if let (Some(a), Some(b)) = (a, b) {
                    a == b
                } else {
                    false
                }
            }
            Predicate::NEQ(lp, rp) => {
                let a = lp.sp_value2(state);
                let b = rp.sp_value2(state);
                if let (Some(a), Some(b)) = (a, b) {
                    a != b
                } else {
                    false
                }
            }
            Predicate::TON(lp, rp) => {
                if let (Some(t), Some(d)) = (lp.sp_value2(state), rp.sp_value2(state)) {
                    if let SPValue::Time(time) = t {
                        let current_duration = time.elapsed().unwrap_or_default();
                        let delay = match d {
                            SPValue::Float32(x) => *x as i32,
                            SPValue::Int32(x) => *x,
                            _ => 0,
                        };
                        current_duration.as_millis() > delay.unsigned_abs() as u128
                    } else {
                        eprintln!("TON must point to a timestamp, and not: {t:?} i");
                        false
                    }
                } else {
                    false
                }
            }
            Predicate::TOFF(lp, rp) => {
                if let (Some(t), Some(d)) = (lp.sp_value2(state), rp.sp_value2(state)) {
                    if let SPValue::Time(time) = t {
                        let current_duration = time.elapsed().unwrap_or_default();
                        let delay = match d {
                            SPValue::Float32(x) => *x as i32,
                            SPValue::Int32(x) => *x,
                            _ => 0,
                        };
                        current_duration.as_millis() < delay.unsigned_abs() as u128
                    } else {
                        eprintln!("TON must point to a timestamp, and not: {t:?} i");
                        false
                    }
                } else {
                    false
                }
            }
            Predicate::MEMBER(lp, rp) => {
                if let (Some(v), Some(xs)) = (lp.sp_value2(state), rp.sp_value2(state)) {
                    if let SPValue::Array(_, xs) = xs {
                        xs.contains(v)
                    } else {
                        eprintln!("Member must point to an array, and not: {xs:?} i");
                        false
                    }
                } else {
                    false
                }
            } // Predicate::GT(lp, rp) => {}
              // Predicate::LT(lp, rp) => {}
              // Predicate::INDOMAIN(value, domain) => {}
        }
    }
}

impl NextAction for Action {
    fn next(&self, state: &mut SPState) -> SPResult<()> {
        let c = match &self.value {
            Compute::PredicateValue(pv) => match pv.sp_value(state).cloned() {
                Some(x) => Some(x),
                None => {
                    eprintln!(
                        "The action PredicateValue, next did not find a value for variable: {pv:?}"
                    );
                    return Err(SPError::No(format!(
                        "The action PredicateValue, next did not find a value for variable: {pv:?}"
                    )));
                }
            },
            Compute::Predicate(p) => {
                let res = p.eval(state);
                Some(res.to_spvalue())
            }
            Compute::Function(xs) => {
                let res = xs
                    .iter()
                    .find(|(p, _)| p.eval(state))
                    .and_then(|(_, v)| v.sp_value(state));
                match res {
                    Some(x) => Some(x.clone()),
                    None => {
                        eprintln!("No predicates in the action Function was true: {self:?}");
                        return Err(SPError::No(format!(
                            "No predicates in the action Function was true: {self:?}"
                        )));
                    }
                }
            }
            Compute::Random(n) => Some(SPValue::Int32(rand::thread_rng().gen_range(0..*n))),
            Compute::TimeStamp => Some(SPValue::Time(std::time::SystemTime::now())),
            Compute::Any => None,
        };

        if let Some(c) = c {
            match &self.state_path {
                Some(sp) => state.next(sp, c),
                None => state.next_from_path(&self.var, c),
            }
        } else {
            Ok(())
        }
    }

    fn next2(&self, state: &mut SPState2) -> SPResult<()> {
        let c = match &self.value {
            Compute::PredicateValue(pv) => match pv.sp_value2(state).cloned() {
                Some(x) => Some(x),
                None => {
                    eprintln!(
                        "The action PredicateValue, next did not find a value for variable: {pv:?}"
                    );
                    return Err(SPError::No(format!(
                        "The action PredicateValue, next did not find a value for variable: {pv:?}"
                    )));
                }
            },
            Compute::Predicate(p) => {
                let res = p.eval2(state);
                Some(res.to_spvalue())
            }
            Compute::Function(xs) => {
                let res = xs
                    .iter()
                    .find(|(p, _)| p.eval2(state))
                    .and_then(|(_, v)| v.sp_value2(state));
                match res {
                    Some(x) => Some(x.clone()),
                    None => {
                        eprintln!("No predicates in the action Function was true: {self:?}");
                        return Err(SPError::No(format!(
                            "No predicates in the action Function was true: {self:?}"
                        )));
                    }
                }
            }
            Compute::Random(n) => Some(SPValue::Int32(rand::thread_rng().gen_range(0..*n))),
            Compute::TimeStamp => Some(SPValue::Time(std::time::SystemTime::now())),
            Compute::Any => None,
        };

        if let Some(c) = c {
            let string = self.var.path.join(".");
            state.insert(string, c);
            Ok(())
        } else {
            Ok(())
        }
    }
}

impl EvaluatePredicate for Action {
    fn eval(&self, state: &SPState) -> bool {
        let sp = match &self.state_path {
            Some(x) => state.state_value(x),
            None => state.state_value_from_path(&self.var),
        };
        match sp {
            Some(x) => !x.has_next(), // MD: I assume we meant to fail if we *already* had a next value for this action
            None => false, // We do not allow actions to add new state variables. But maybe this should change?
        }
    }

    fn eval2(&self, _state: &SPState2) -> bool {
        return true;
    }
}

#[macro_export]
macro_rules! p {
    // parens
    (($($inner:tt)+) ) => {{
        // println!("matched parens: {}", stringify!($($inner)+));
        p! ( $($inner)+ )
    }};
    ([$($inner:tt)+] ) => {{
        // println!("matched square parens: {}", stringify!($($inner)+));
        p! ( $($inner)+ )
    }};

    // AND: the brackets are needed because "tt" includes && which
    // leads to ambiguity without an additional delimeter
    ([$($first:tt)+] $(&& [$($rest:tt)+])+) => {{
        // println!("matched &&: {}", stringify!($($first)+));
        let first = p! ( $($first)+ );
        let mut v = vec![first];
        $(
            // println!(" && ...: {}", stringify!($($rest)+));
            let r = p!($($rest)+);
            v.push(r);
        )*
        Predicate::AND(v)
    }};

    // OR: same as and.
    ([$($first:tt)+] $(|| [$($rest:tt)+])+) => {{
        // println!("matched ||: {}", stringify!($($first)+));
        let first = p! ( $($first)+ );
        let mut v = vec![first];
        $(
            let r = p!($($rest)+);
            v.push(r);
        )*
        Predicate::OR(v)
    }};

    // implication
    ([$($x:tt)+] => [$($y:tt)+]) => {{
        // println!("matched implication: {} => {}", stringify!($($x)+), stringify!($($y)+));
        let x = p! ( $($x)+ );
        let y = p! ( $($y)+ );
        Predicate::OR(vec![Predicate::NOT(Box::new(x)), y])
    }};

    ([ $lhs:expr ] == [ $rhs:expr ]) => {{
        Predicate::EQ(
            $lhs .to_predicate_value(),
            $rhs .to_predicate_value(),
        )
    }};

    ([ $lhs:expr ] == $($rhs:tt).+) => {{
        Predicate::EQ(
            $lhs .to_predicate_value(),
            $($rhs).+ .to_predicate_value(),
        )
    }};

    ($($lhs:tt).+ == [ $rhs:expr ]) => {{
        Predicate::EQ(
            $($lhs).+ .to_predicate_value(),
            $rhs .to_predicate_value(),
        )
    }};

    ($($lhs:tt).+ == $($rhs:tt).+) => {{
        Predicate::EQ(
            $($lhs).+ .to_predicate_value(),
            $($rhs).+ .to_predicate_value(),
        )
    }};

    ([ $lhs:expr ] != [ $rhs:expr ]) => {{
        Predicate::NEQ(
            $lhs .to_predicate_value(),
            $rhs .to_predicate_value(),
        )
    }};

    ([ $lhs:expr ] != $($rhs:tt).+) => {{
        Predicate::NEQ(
            $lhs .to_predicate_value(),
            $($rhs).+ .to_predicate_value(),
        )
    }};

    ($($lhs:tt).+ != [ $rhs:expr ]) => {{
        Predicate::NEQ(
            $($lhs).+ .to_predicate_value(),
            $rhs .to_predicate_value(),
        )
    }};

    ($($lhs:tt).+ != $($rhs:tt).+) => {{
        Predicate::NEQ(
            $($lhs).+ .to_predicate_value(),
            $($rhs).+ .to_predicate_value(),
        )
    }};

    // negation
    (! $($inner:tt)+ ) => {{
        // println!("matched negation: {}", stringify!($($inner)+));
        let inner = p! ( $($inner)+ );
        Predicate::NOT(Box::new( inner ))
    }};

    ($i:expr) => {{
        $i.to_predicate()
    }};

}

pub fn binary_expr_to_action(lhs: &PredicateValue, rhs: &PredicateValue) -> Action {
    match (lhs, rhs) {
        (PredicateValue::SPPath(p, _), pv) =>
            Action::new(
                p.clone(),
                Compute::PredicateValue(pv .clone())
            ),
        _ => panic!("assignment not supported {lhs:?} = {rhs:?} (variable should be on left hand side)")
    }
}

pub fn binary_expr_negation(lhs: &PredicateValue, rhs: &PredicateValue) -> Action {
    match (lhs, rhs) {
        (PredicateValue::SPPath(p, _), pv) =>
            Action::new(
                p.clone(),
                Compute::Predicate(Predicate::NOT(Box::new(
                    Predicate::EQ(pv.clone(), PredicateValue::SPValue(true.to_spvalue()))))),
            ),
        _ => panic!("negation not supported {lhs:?} = ! {rhs:?} (variable should be on left hand side)")
    }
}

pub fn unary_expr_to_action(lhs: &PredicateValue, c: Compute) -> Action {
    match lhs {
        PredicateValue::SPPath(p, _) =>
            Action::new(
                p.clone(),
                c
            ),
        _ => panic!("assignment not supported {lhs:?} {c:?} (variable should be on left hand side)")
    }
}

#[macro_export]
macro_rules! a {
    ([ $lhs:expr ] = ! [ $rhs:expr ]) => {{
        let lhs = $lhs .to_predicate_value();
        let rhs = $rhs .to_predicate_value();
        binary_expr_negation(&lhs, &rhs)
    }};

    ([ $lhs:expr ] = [ $rhs:expr ]) => {{
        let lhs = $lhs .to_predicate_value();
        let rhs = $rhs .to_predicate_value();
        binary_expr_to_action(&lhs, &rhs)
    }};

    ([ $lhs:expr ] = ! $($rhs:tt).+) => {{
        let lhs = $lhs .to_predicate_value();
        let rhs = $($rhs).+ .to_predicate_value();
        binary_expr_negation(&lhs, &rhs)
    }};

    ([ $lhs:expr ] = $($rhs:tt).+) => {{
        let lhs = $lhs .to_predicate_value();
        let rhs = $($rhs).+ .to_predicate_value();
        binary_expr_to_action(&lhs, &rhs)
    }};

    ($($lhs:tt).+ = ! [ $rhs:expr ]) => {{
        let lhs = $($lhs).+ .to_predicate_value();
        let rhs = $rhs .to_predicate_value();
        binary_expr_negation(&lhs, &rhs)
    }};

    ($($lhs:tt).+ = [ $rhs:expr ]) => {{
        let lhs = $($lhs).+ .to_predicate_value();
        let rhs = $rhs .to_predicate_value();
        binary_expr_to_action(&lhs, &rhs)
    }};

    ($($lhs:tt).+ = ! $($rhs:tt).+) => {{
        let lhs = $($lhs).+ .to_predicate_value();
        let rhs = $($rhs).+ .to_predicate_value();
        binary_expr_negation(&lhs, &rhs)
    }};

    ($($lhs:tt).+ = $($rhs:tt).+) => {{
        let lhs = $($lhs).+ .to_predicate_value();
        let rhs = $($rhs).+ .to_predicate_value();
        binary_expr_to_action(&lhs, &rhs)
    }};


    (! [ $lhs:expr ]) => {{
        let lhs = $lhs .to_predicate_value();
        unary_expr_to_action(&lhs, Compute::PredicateValue(PredicateValue::SPValue(false.to_spvalue())))
    }};

    (! $($lhs:tt).+) => {{
        let lhs = $($lhs).+ .to_predicate_value();
        unary_expr_to_action(&lhs, Compute::PredicateValue(PredicateValue::SPValue(false.to_spvalue())))
    }};


    ([ $lhs:expr ] ?) => {{
        let lhs = $lhs .to_predicate_value();
        unary_expr_to_action(&lhs, Compute::Any)
    }};

    ($($lhs:tt).+ ?) => {{
        let lhs = $($lhs).+ .to_predicate_value();
        unary_expr_to_action(&lhs, Compute::Any)
    }};

    ([ $lhs:expr ]) => {{
        let lhs = $lhs .to_predicate_value();
        unary_expr_to_action(&lhs, Compute::PredicateValue(PredicateValue::SPValue(true.to_spvalue())))
    }};

    ($($lhs:tt).+) => {{
        let lhs = $($lhs).+ .to_predicate_value();
        unary_expr_to_action(&lhs, Compute::PredicateValue(PredicateValue::SPValue(true.to_spvalue())))
    }};
}

/// ********** TESTS ***************

#[cfg(test)]
mod sp_value_test {
    #![warn(unused_variables)]

    use super::*;

    // we want to make sure that the macro and string parser
    // have the same semantics. They probably differ a bit a
    // the moment, especially with deciding what is a path and
    // what is a value.
    #[test]
    fn macro_vs_parser() {
        let ab = SPPath::from(&["a", "b"]);
        let ac = SPPath::from(&["a", "c"]);
        let kl = SPPath::from(&["k", "l"]);

        let p1 = format!("p:{} && p:{} && p:{}", ab, ac, kl);
        assert_eq!(
            Predicate::from_string(&p1).unwrap(),
            p!([ab] && [ac] && [kl])
        );

        let p1 = format!("(!p:{}) && p:{} && p:{}", ab, ac, kl);
        assert_eq!(
            Predicate::from_string(&p1).unwrap(),
            p!([!ab] && [ac] && [kl])
        );

        let p1 = format!("(!p:{}) && p:{} || p:{}", ab, ac, kl);
        assert_eq!(
            Predicate::from_string(&p1).unwrap(),
            p!([[!ab] && [ac]] || [kl])
        );

        let p1 = format!("(!p:{}) && p:{} -> p:{}", ab, ac, kl);
        assert_eq!(
            Predicate::from_string(&p1).unwrap(),
            p!( [[ !ab] && [ac]] => [kl ])
        );

        // samve expr as above but with whitespaces interspersed
        let p1 = format!(" ( ( ! p: {} ) && p: {} ) -> ( p:{} ) ", ab, ac, kl);
        assert_eq!(
            Predicate::from_string(&p1).unwrap(),
            p!( [[ !ab] && [ac]] => [kl])
        );
    }

    #[test]
    fn eval_pred() {
        let s = state!(["a", "b"] => 2,
                       ["a", "c"] => true,
                       ["k", "l"] => true);
        let v = SPPath::from(&["a", "b"]);
        let eq = Predicate::EQ(
            PredicateValue::SPValue(2.to_spvalue()),
            PredicateValue::SPPath(v.clone(), None),
        );
        let eq2 = Predicate::EQ(
            PredicateValue::SPValue(3.to_spvalue()),
            PredicateValue::SPPath(v.clone(), None),
        );
        assert!(eq.eval(&s));
        assert!(!eq2.eval(&s));
    }

    #[test]
    fn support_pred() {
        let ab = SPPath::from(&["a", "b"]);
        let ac = SPPath::from(&["a", "c"]);
        let kl = SPPath::from(&["k", "l"]);

        let eq = Predicate::EQ(
            PredicateValue::SPValue(2.to_spvalue()),
            PredicateValue::SPPath(ac.clone(), None),
        );
        let eq2 = Predicate::NEQ(
            PredicateValue::SPValue(3.to_spvalue()),
            PredicateValue::SPPath(kl.clone(), None),
        );
        let eq3 = Predicate::EQ(
            PredicateValue::SPValue(3.to_spvalue()),
            PredicateValue::SPPath(ab.clone(), None),
        );
        let x = Predicate::AND(vec![eq, eq2]);
        let x = Predicate::OR(vec![x, eq3]);
        assert_eq!(x.support(), vec![ab.clone(), ac.clone(), kl.clone()]);
    }

    #[test]
    fn action_test() {
        let x = SPPath::from("x");
        let y = SPPath::from("x.y");

        let assign_5 = Compute::PredicateValue(PredicateValue::SPValue(5.to_spvalue()));
        let assign_true = Compute::PredicateValue(PredicateValue::SPValue(true.to_spvalue()));
        let assign_false = Compute::PredicateValue(PredicateValue::SPValue(false.to_spvalue()));
        assert_eq!(a!(x = 5), Action::new(x.clone(), assign_5.clone()));

        assert_eq!(a!("p:x" = 5), Action::new(x.clone(), assign_5.clone()));

        assert_eq!(a!(y = 5), Action::new(y.clone(), assign_5.clone()));

        assert_eq!(a!(y), Action::new(y.clone(), assign_true.clone()));

        assert_eq!(a!(!"p:x"), Action::new(x.clone(), assign_false.clone()));
    }

    #[test]
    fn pred_test() {
        let x = SPPath::from("x");
        let x_true = Predicate::EQ(
            PredicateValue::SPPath(SPPath::from("x"), None),
            PredicateValue::SPValue(true.to_spvalue()),
        );
        let not_x_true = Predicate::NOT(Box::new(x_true.clone()));
        assert_eq!(p!(x), x_true);
        assert_eq!(p!(!x), not_x_true);
        assert_eq!(p!(!(x)), not_x_true);

        let lp = SPPath::from("really/long/path");
        let lp_true = Predicate::EQ(
            PredicateValue::SPPath(lp.clone(), None),
            PredicateValue::SPValue(true.to_spvalue()),
        );
        let not_lp_true = Predicate::NOT(Box::new(lp_true.clone()));
        assert_eq!(
            p!([!lp] && [!x]),
            Predicate::AND(vec![not_lp_true.clone(), not_x_true.clone()])
        );

        assert_eq!(
            p!([!lp] && [!x] && [([x] || [lp])]),
            Predicate::AND(vec![
                not_lp_true.clone(),
                not_x_true.clone(),
                Predicate::OR(vec![x_true.clone(), lp_true.clone()])
            ])
        );

        assert_eq!(
            p!("p: x" == 5),
            Predicate::EQ(
                PredicateValue::SPPath(SPPath::from("x"), None),
                PredicateValue::SPValue(5.to_spvalue())
            )
        );
        assert_eq!(
            p!(lp == 5),
            Predicate::EQ(
                PredicateValue::SPPath(lp.clone(), None),
                PredicateValue::SPValue(5.to_spvalue())
            )
        );
    }

    #[test]
    fn test_new_predicate_macros() {
        struct Test {
            path: SPPath,
            variable: Variable,
        }
        let r = Test {
            path: SPPath::from("r.path"),
            variable: Variable::new_boolean("r.var".into()),
        };
        impl Test {
            fn fun_p() -> SPPath {
                return SPPath::from("path_from_fun");
            }
            fn fun_v() -> Variable {
                return Variable::new_boolean("r.var".into());
            }
        }

        let x = p!([r.path == r.variable]);
        assert_eq!(x, Predicate::EQ(PredicateValue::SPPath("r.path".into(), None),
                                    PredicateValue::SPPath("r.var".into(), None)));
        let x = p!([Test::fun_v()] == r.path);
        assert_eq!(x, Predicate::EQ(PredicateValue::SPPath("r.var".into(), None),
                                    PredicateValue::SPPath("r.path".into(), None)));
        let prev_x = x.clone();
        let x = p!([[r.variable] == r.path] && [prev_x.clone()]);
        assert_eq!(x,
                   Predicate::AND(vec![
                       Predicate::EQ(PredicateValue::SPPath("r.var".into(), None),
                                     PredicateValue::SPPath("r.path".into(), None)),
                       prev_x]));

        let y = p!([r.variable == [Test::fun_p()]] && [r.variable]);
        let x = p!([y] => [y]);
        assert_eq!(x, Predicate::OR(vec![Predicate::NOT(Box::new(y.clone())), y.clone()]));
    }

    #[test]
    fn test_string_path() {
        let p = SPPath::from("p.path");
        let x = p!("p:p.path" == p);
        assert_eq!(x, Predicate::EQ(
            PredicateValue::SPPath("p.path".into(), None),
            PredicateValue::SPPath("p.path".into(), None)));

        let x = p!(p != "p:p.path");
        assert_eq!(x, Predicate::NEQ(
            PredicateValue::SPPath("p.path".into(), None),
            PredicateValue::SPPath("p.path".into(), None)));

        println!("{x:?}\n");
    }

    #[test]
    fn test_new_action_macro() {
        struct Test {
            path: SPPath,
            variable: Variable,
        }
        let r = Test {
            path: SPPath::from("r.path"),
            variable: Variable::new_boolean("r.var".into()),
        };

        impl Test {
            fn fun_p() -> SPPath {
                return SPPath::from("path_from_fun");
            }
            fn fun_v() -> Variable {
                return Variable::new_boolean("r.var".into());
            }
        }
        let var = Variable::new_boolean("testvar".into());

        let p = SPPath::from("p.path");
        let value = false.to_spvalue();

        let x = a!(p = value);
        assert_eq!(x, Action::new(p.clone(),
                                  Compute::PredicateValue(
                                      PredicateValue::SPValue(false.to_spvalue()))));
        println!("{x}");

        let x = a!(r.path = [Test::fun_v()]);
        println!("{x}");

        let x = a!(r.path = "hello");
        println!("{x:?}");

        let x = a!(r.path = "p: hello");
        println!("{x:?}");

        let x = a!(r.path = 5);
        println!("{x}");

        let x = a!([Test::fun_v()] = [Test::fun_p()]);
        println!("{x}");

        let x = a!(r.variable = var);
        println!("{x}");

        let x = a!([Test::fun_v()] = var);
        println!("{x}");

        let x = a!(![Test::fun_v()]);
        println!("{x}");

        let x = a!(! "p: hello");
        println!("{x}");

        let x = a!([Test::fun_v()]);
        println!("{x}");

        let x = a!("p: hello");
        println!("{x}");

        let x = a!([Test::fun_v()] ?);
        println!("{x}");

        let x = a!("p: hello" ?);
        println!("{x}");

        let x = a!("p: hello" = !r.variable);
        println!("{x}");

        let x = a!("p: hello" = ! "p: hello");
        println!("{x}");

        let x = a!([Test::fun_v()] = ! var);
        println!("{x}");

        let x = a!([Test::fun_v()] = ! [Test::fun_v()]);
        println!("{x}");

        let x = a!(var = ! [Test::fun_v()]);
        println!("{x}");

    }
}
