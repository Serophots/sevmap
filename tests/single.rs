use sevmap::muts::Mutable;

#[macro_export]
macro_rules! assert_match {
    ($x:expr, $p:pat) => {
        if let $p = $x {
        } else {
            panic!(concat!(stringify!($x), " did not match ", stringify!($p)));
        }
    };
}

enum MutateValue {
    Increment(i32),
    Decrement(i32),
}

impl Mutable<MutateValue> for i32 {
    unsafe fn mutate(&mut self, operation: &mut MutateValue) {
        match operation {
            MutateValue::Increment(v) => *self += *v,
            MutateValue::Decrement(v) => *self -= *v,
        }
    }
}

#[test]
fn single_works() {
    let x = ('x', 42);

    let (mut w, r) = sevmap::new();

    assert_match!(r.get(&x.0), None);
    // assert_match!(r.meta());

    w.publish();

    assert_match!(r.get(&x.0), None);
    // assert_match!(r.meta());

    w.insert(x.0, x.1, 10);

    assert_match!(r.get(&x.0), None);

    w.publish();
    w.mutate(x.0, MutateValue::Decrement(2));

    assert_eq!(r.get(&x.0).unwrap().ref_v(), &x.1);
    assert_eq!(r.get(&x.0).unwrap().mut_v(), &10);

    w.publish();
    w.mutate(x.0, MutateValue::Decrement(2));
    w.mutate(x.0, MutateValue::Increment(50));

    assert_eq!(r.get(&x.0).unwrap().ref_v(), &x.1);
    assert_eq!(r.get(&x.0).unwrap().mut_v(), &8);

    w.publish();
    w.clear();

    assert_eq!(r.get(&x.0).unwrap().ref_v(), &x.1);
    assert_eq!(r.get(&x.0).unwrap().mut_v(), &56);

    w.publish();

    assert_match!(r.get(&x.0), None);
}
