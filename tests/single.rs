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
    fn mutate(&mut self, operation: &mut MutateValue) {
        match operation {
            MutateValue::Increment(v) => *self += *v,
            MutateValue::Decrement(v) => *self -= *v,
        }
    }
}

#[test]
fn single_works() {
    let x = ('x', 42);
    let y = ('y', 23);

    let (mut w, r) = sevmap::Options::default().with_meta(100).construct();

    // Test insertion, meta, len, is empty
    assert_match!(r.get(&x.0), None);
    assert_eq!(*r.meta().unwrap(), 100);
    assert!(r.is_empty());
    assert_eq!(r.len(), 0);

    w.publish();

    assert_match!(r.get(&x.0), None);
    assert_eq!(*r.meta().unwrap(), 100);
    assert!(r.is_empty());
    assert_eq!(r.len(), 0);

    w.insert(x.0, x.1, 10);
    w.set_meta(1502);

    assert_match!(r.get(&x.0), None);
    assert_eq!(*r.meta().unwrap(), 100);

    w.publish();
    w.insert(y.0, y.1, 50);
    w.mutate(x.0, MutateValue::Decrement(2));

    assert_eq!(r.get(&x.0).unwrap().ref_v(), &x.1);
    assert_eq!(r.get(&x.0).unwrap().mut_v(), &10);
    assert_eq!(*r.meta().unwrap(), 1502);

    w.publish();
    w.mutate(x.0, MutateValue::Decrement(2));
    w.mutate(x.0, MutateValue::Increment(50));

    assert_eq!(r.get(&x.0).unwrap().ref_v(), &x.1);
    assert_eq!(r.get(&x.0).unwrap().mut_v(), &8);
    assert_eq!(*r.meta().unwrap(), 1502);

    w.publish();

    {
        // Test iterators
        assert_eq!(r.enter().unwrap().iter().count(), 2);
        assert_eq!(r.enter().unwrap().keys().count(), 2);
        assert_eq!(r.enter().unwrap().values().count(), 2);

        let entry = r.enter().unwrap();
        let mut iter = entry.iter();

        let first = iter.next().unwrap();
        let second = iter.next().unwrap();
        assert!(first.0 == &x.0 || first.0 == &y.0);
        assert!(second.0 == &x.0 || second.0 == &y.0);
    }

    // Test clear

    w.clear();
    assert!(!r.is_empty());
    assert_ne!(r.len(), 0);

    assert_eq!(r.get(&x.0).unwrap().ref_v(), &x.1);
    assert_eq!(r.get(&x.0).unwrap().mut_v(), &56);
    assert_eq!(*r.meta().unwrap(), 1502);

    w.publish();
    assert!(r.is_empty());
    assert_eq!(r.len(), 0);

    assert_match!(r.get(&x.0), None);
    assert_eq!(*r.meta().unwrap(), 1502);
}
