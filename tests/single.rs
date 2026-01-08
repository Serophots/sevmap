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
fn it_works() {
    let x = ('x', 42);

    let (mut w, r) = svmap::new_single();

    assert_match!(r.get(&x.0), None);
    // assert_match!(r.meta());

    w.publish();

    assert_match!(r.get(&x.0), None);
    // assert_match!(r.meta());

    w.insert(x.0, x.1);

    assert_match!(r.get(&x.0), None);

    w.publish();

    assert_match!(r.get(&x.0), Some(_));

    w.clear();

    assert_match!(r.get(&x.0), Some(_));

    w.publish();

    assert_match!(r.get(&x.0), None);
}
