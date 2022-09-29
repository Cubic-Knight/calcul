macro_rules! make_test_functions {
    ( $(fn $fnname:tt $mode:tt for $op:tt),+ , with $args:tt ) => {
        mod t_funcs {
            use crate::fixed::Fixed;
            $(
                generate_func!(fn $mode $fnname for $op);
            )+
        }
        $(
            test_func!($fnname, $args);
        )+
    };
}

macro_rules! generate_func {
    (fn(exact) $fnname:tt for $op:tt) => {
        pub fn $fnname(a: f64, b: f64) {
            let a = Fixed::from_f64(a).as_i64();
            let b = Fixed::from_f64(b).as_i64();
            let lhs = Fixed::from_i64(a);
            let rhs = Fixed::from_i64(b);
            let expected = Fixed::from_i64(a $op b);
            assert_eq!(lhs $op rhs, expected)
        }
    };
    (fn(exact, int) $fnname:tt for $op:tt) => {
        pub fn $fnname(a: f64, b: f64) {
            let a = Fixed::from_f64(a).as_i64();
            let b = Fixed::from_f64(b).as_i64();
            let lhs = Fixed::from_i64(a);
            let rhs = Fixed::from_i64(b);
            let expected = Fixed::from_i64(a $op (b >> 32));
            assert_eq!(lhs $op rhs, expected)
        }
    };
    (fn(approx, $epsilon:tt) $fnname:tt for $op:tt) => {
        pub fn $fnname(a: f64, b: f64) {
            let lhs = Fixed::from_f64(a);
            let rhs = Fixed::from_f64(b);
            let result = (lhs $op rhs).as_i64();
            let expected = Fixed::from_f64(a $op b).as_i64();
            assert!(
                expected.abs_diff(result) < $epsilon,
                "{} {} {} => expected={}, result={}",
                a, stringify!($op), b,
                Fixed::from_i64(expected), Fixed::from_i64(result)
            )
        }
    }
}

macro_rules! test_func {
    ( $fnname:tt, [ $( ($a:expr, $b:expr) ),+ ] ) => {
        #[test]
        fn $fnname() {
            $(
                t_funcs::$fnname($a, $b);
            )+
        }
    };
}

make_test_functions!(
    fn add(exact) for +,
    fn sub(exact) for -,
    fn mul(approx, (1 << 32)) for *,
    fn div(approx, (1 << 16)) for /,
    fn rem(exact) for %,
    fn and(exact) for &,
    fn ior(exact) for |,
    fn xor(exact) for ^,
    fn shl(exact, int) for <<,
    fn shr(exact, int) for >>,
    with [
        ( 0.0, 1.0),
        (-1.0, 0.5),
        ( 96.436, -16.128),
        ( 90.055,  91.467),
        (-22.057,  48.551),
        (-36.505,  53.295),
        (-47.676, -36.603),
        (-38.731, -91.135),
        (-39.904,   8.170),
        ( 26.714,  64.207),
        ( 16.187,  44.724),
        (-98.363,  21.538),
        ( 70223.582, -20363.576),
        ( 45031.022,  77712.991),
        (-78915.655, - 6465.442),
        (-51526.795, -51516.658),
        ( 77184.568,  35352.514),
        (-11626.399, -71328.079),
        ( 95689.023, -66248.097),
        (-41206.371,  13553.372),
        (-59120.587, -21787.521),
        ( 89759.877,  66300.215)
    ]
);
