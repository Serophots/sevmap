#[macro_export]
macro_rules! assert_match {
    ($x:expr, $p:pat) => {
        if let $p = $x {
        } else {
            panic!(concat!(stringify!($x), " did not match ", stringify!($p)));
        }
    };
}

#[test]
fn single_ref_works() {
    let x = ('x', 42);

    let (mut w, r) = sevmap::new();

    assert_match!(r.get(&x.0), None);
    // assert_match!(r.meta());

    w.publish();

    assert_match!(r.get(&x.0), None);
    // assert_match!(r.meta());

    w.insert(x.0, x.1, ());

    assert_match!(r.get(&x.0), None);

    w.publish();

    assert_eq!(r.get(&x.0).unwrap().ref_v(), &x.1);
    assert_eq!(r.get(&x.0).unwrap().mut_v(), &());

    w.clear();

    assert_eq!(r.get(&x.0).unwrap().ref_v(), &x.1);
    assert_eq!(r.get(&x.0).unwrap().mut_v(), &());

    w.publish();

    assert_match!(r.get(&x.0), None);
}
