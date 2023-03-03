use criterion::{criterion_group, criterion_main, Criterion};
use sp_domain::*;
use rand::thread_rng;
use rand::seq::SliceRandom;

// A summary of the findings of these benchmarks.
// ---
// Original code with cached lookup was ~155ms.
// Changing the state to a naive hashmap increased it to ~910ms.
// Using FxHashMap instead of HashMap brought it down to ~720ms.
// Killing off the vector in SPPath brough benchmark down to 318ms.
// Minor ownership changes improved it further to 286ms.
fn bench_state_eval() {
    // Create 100 variables and 200 transitions
    let mut state = SPState::new();
    let mut trans = vec![];
    for i in 1..100 {
        let var = SPPath::from(format!("var_{i}"));
        state.add_variable(var.clone(), false.to_spvalue());

        let t1 = Transition::new("t1".into(), p!(! p: var), vec![a!(p: var)]);
        let t2 = Transition::new("t2".into(), p!(p: var), vec![a!(!p: var)]);
        trans.push(t1);
        trans.push(t2);
    }

    // shuffe transitions to trigger random lookups in the state.
    trans.shuffle(&mut thread_rng());

    for t in &mut trans {
        t.upd_state_path(&state);
    }

    // random runner for 100 000 steps
    for _ in 0..100_000 {
        for t in &trans {
            t.eval(&mut state);
        }
    }
}

fn bench_simple_state_eval() {
    // Create 100 variables and 200 transitions
    let mut state = SPState2::default();
    let mut trans = vec![];
    for i in 1..100 {
        let string = format!("var_{i}");
        let var = SPPath::from_string(&string);
        state.insert(string, false.to_spvalue());

        let t1 = Transition::new("t1", p!(! p: var), vec![a!(p: var)]);
        let t2 = Transition::new("t2", p!(p: var), vec![a!(!p: var)]);
        trans.push(t1);
        trans.push(t2);
    }

    // shuffe transitions to trigger random lookups in the state.
    trans.shuffle(&mut thread_rng());

    // random runner for 100 000 steps
    for _ in 0..100_000 {
        for t in &trans {
            t.eval2(&mut state);
        }
    }
}


pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("state_eval", |b| b.iter(|| bench_simple_state_eval()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
