use std::{
    ops::{
        Add, Sub, Mul, Div, Rem, BitAnd, BitOr, BitXor, Shl, Shr
    },
    fmt::{
        self, Display
    }
};
const U32MAX_FLOAT: f64 = 4_294_967_296_f64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fixed(i64);

#[allow(unused)]
impl Fixed {
    pub const MIN: Fixed = Fixed(i64::MIN);
    pub const MAX: Fixed = Fixed(i64::MAX);
    pub const EPSILON: Fixed = Fixed(1);
    pub fn new(x: i32) -> Self {
        Fixed((x as i64) << 32)
    }
    pub fn from_i64(x: i64) -> Self {
        Fixed(x)
    }
    pub fn from_f64(f: f64) -> Self {
        let modulo = match f % U32MAX_FLOAT {
            num if !num.is_finite() => panic!(),
            num if num.is_sign_positive() => { num },
            num if num.is_sign_negative() => { num + U32MAX_FLOAT },
            _ => panic!()
        };
        let rounded = (modulo * U32MAX_FLOAT).round() as u64 as i64;
        Fixed(rounded)
    }
    pub fn as_i64(&self) -> i64 { self.0 }
}

macro_rules! impl_easy_traits {
    ($(impl $trait:tt (fn $fnname:tt) with $op:tt),+) => {
        $(
            impl $trait for Fixed {
                type Output = Fixed;
                fn $fnname(self, rhs: Self) -> Fixed {
                    Fixed(self.0 $op rhs.0)
                }
            }
        )+
    };
}

impl_easy_traits!(
    impl Add (fn add) with +,
    impl Sub (fn sub) with -,
    impl Rem (fn rem) with %,
    impl BitAnd (fn bitand) with &,
    impl BitOr (fn bitor) with |,
    impl BitXor (fn bitxor) with ^
);

impl Mul for Fixed {
    type Output = Fixed;
    fn mul(self, rhs: Self) -> Fixed {
        let a = i128::from(self.0);
        let b = i128::from(rhs.0);
        Fixed((a*b >> 32) as i64)
    }
}

impl Div for Fixed {
    type Output = Fixed;
    fn div(self, rhs: Self) -> Fixed {
        let a = i128::from(self.0) << 32;
        let b = i128::from(rhs.0);
        Fixed((a/b) as i64)
    }
}

impl Shl for Fixed {
    type Output = Fixed;
    fn shl(self, rhs: Self) -> Fixed {
        Fixed(self.0 << (rhs.0 >> 32))
    }
}

impl Shr for Fixed {
    type Output = Fixed;
    fn shr(self, rhs: Self) -> Fixed {
        Fixed(self.0 >> (rhs.0 >> 32))
    }
}

const BILLION: f64 = 1_000_000_000.0;
impl Display for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const DECIMAL_RATIO: f64 = (1u64 << 32) as f64;

        let whole = (self.0.abs() >> 32).abs().to_string();
        let decimal = f64::from(self.0.abs() as u32) / DECIMAL_RATIO;
        let decimal = match f.precision() {
            Some(prec) => format!("{:.*}", prec, decimal),
            None => format!("{}", (decimal * BILLION).round() / BILLION)
        };

        f.pad_integral(self.0 >= 0, "", &(whole + &decimal[1..]))
    }
}
